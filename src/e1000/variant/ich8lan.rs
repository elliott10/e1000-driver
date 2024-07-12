use core::time::Duration;

use super::{VariantCommon, VariantResult};
use alloc::vec::Vec;
use core::mem::size_of;

use crate::defines::*;
use crate::e1000::mac::*;
use crate::e1000::nvm::*;
use crate::e1000::reg::{self, *};
use crate::kfn;
use crate::E1000;
use crate::{defines::*, err::*, phy::*};
use crate::{err::*, BoardInfo, Flags};

const ICH_RAR_ENTRIES: u16 = 7;
const PCH2_RAR_ENTRIES: u16 = 5; /* RAR[0], SHRA[0-3] */
const PCH_LPT_RAR_ENTRIES: u16 = 12; /* RAR[0], SHRA[0-10] */

const NVM_SIZE_MULTIPLIER: u32 = 4096; // multiplier for NVMS field
const FLASH_BASE_ADDR: usize = 0xE000; // offset of NVM access regs
const CTRL_EXT_NVMVS: u32 = 0x3; // NVM valid sector

// TARC0 constants with bit masks
const TARC0_CB_MULTIQ_3_REQ: u32 = 0x30000000;
const TARC0_CB_MULTIQ_2_REQ: u32 = 0x20000000;

const PCIE_ICH8_SNOOP_ALL: u32 = PCIE_NO_SNOOP_ALL;

pub(crate) const ICH_FLASH_GFPREG: usize = 0x0000; // GFP Configuration Register
pub(crate) const ICH_FLASH_HSFSTS: usize = 0x0004; // Host Status and Control Register
pub(crate) const ICH_FLASH_HSFCTL: usize = 0x0006; // Host Control Register
pub(crate) const ICH_FLASH_FADDR: usize = 0x0008; // Flash Address Register
pub(crate) const ICH_FLASH_FDATA0: usize = 0x0010; // Flash Data Register 0
pub(crate) const ICH_FLASH_PR0: usize = 0x0074; // Protected Range Register 0
                                                // Requires up to 10 seconds when MNG might be accessing part.
pub(crate) const ICH_FLASH_READ_COMMAND_TIMEOUT: u32 = 10000000;
pub(crate) const ICH_FLASH_WRITE_COMMAND_TIMEOUT: u32 = 10000000;
pub(crate) const ICH_FLASH_ERASE_COMMAND_TIMEOUT: u32 = 10000000;

pub(crate) const ICH_FWSM_RSPCIPHY: u32 = 0x00000040; // Reset PHY on PCI Reset

const ICH8_LAN_INIT_TIMEOUT: usize = 1500;

// Mask for linear address in flash operations.
pub(crate) const ICH_FLASH_LINEAR_ADDR_MASK: u32 = 0x00FFFFFF;

// Number of times a flash cycle should be repeated.
pub(crate) const ICH_FLASH_CYCLE_REPEAT_COUNT: u32 = 10;

const HSFSTS_FLDESVALID: u16 = 1 << 14;
const HSFSTS_FLCERR: u16 = 1 << 1;
const HSFSTS_DAEL: u16 = 1 << 2;

const HSFSTS_FLCINPROG: u16 = 1 << 5;
const HSFSTS_FLCDONE: u16 = 1;

const HSFCTL_FLCGO: u16 = 1;
const HSFCTL_FLCYCLE_SHIFT: u16 = 1;
const HSFCTL_FLDBCOUNT_SHIFT: u16 = 8;

const ICH_CYCLE_READ: u16 = 0;
const ICH_CYCLE_WRITE: u16 = 2;
const ICH_CYCLE_ERASE: u16 = 3;
const ICH_NVM_SIG_WORD: usize = 0x13;
const ICH_NVM_SIG_MASK: u16 = 0xC000;
const ICH_NVM_VALID_SIG_MASK: u8 = 0xC0;
const ICH_NVM_SIG_VALUE: u8 = 0x80;

const FEXTNVM3_PHY_CFG_COUNTER_MASK: u32 = 0x0C000000;
const FEXTNVM3_PHY_CFG_COUNTER_50MSEC: u32 = 0x08000000;

const FEXTNVM_SW_CONFIG: u32 = 1;
const FEXTNVM_SW_CONFIG_ICH8M: u32 = (1 << 27); /* different on ICH8M */

const ICH8_SHADOW_RAM_WORDS: usize = 2048;

struct FlashCtrl {
    value: u16,
}

impl FlashCtrl {
    fn set_count(&mut self, count: usize) {
        let c = (count & 0b11) as u16;
        self.value &= !(0b11 << HSFCTL_FLDBCOUNT_SHIFT);
        self.value |= c << HSFCTL_FLDBCOUNT_SHIFT;
    }

    fn set_cycle(&mut self, cycle: u16) {
        let c = cycle & 0b11;

        self.value &= !(0b11 << HSFCTL_FLCYCLE_SHIFT);
        self.value |= c << HSFCTL_FLCYCLE_SHIFT;
    }
}
impl From<u16> for FlashCtrl {
    fn from(value: u16) -> Self {
        Self { value }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShadowRam {
    pub value: u16,
    pub modified: bool,
}

/* I218 PHY Ultra Low Power (ULP) states */
#[derive(Debug, Clone, Copy)]
pub(crate) enum UlpState {
    Unknown,
    Off,
    On,
}

#[derive(Debug)]
pub(crate) struct Context {
    pub kmrn_lock_loss_workaround_enabled: bool,
    pub shadow_ram: Vec<ShadowRam>,
    pub nvm_k1_enabled: bool,
    pub eee_disable: bool,
    pub eee_lp_ability: u16,
    pub ulp_state: UlpState,
}

impl Clone for Context {
    fn clone(&self) -> Self {
        let mut shadow_ram = Vec::new();
        shadow_ram.try_extend_from_slice(&self.shadow_ram);

        Self {
            kmrn_lock_loss_workaround_enabled: self.kmrn_lock_loss_workaround_enabled.clone(),
            shadow_ram,
            nvm_k1_enabled: self.nvm_k1_enabled.clone(),
            eee_disable: self.eee_disable.clone(),
            eee_lp_ability: self.eee_lp_ability.clone(),
            ulp_state: self.ulp_state.clone(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        let mut shadow_ram = Vec::try_with_capacity(ICH8_SHADOW_RAM_WORDS).unwrap();
        for _ in 0..ICH8_SHADOW_RAM_WORDS {
            shadow_ram.try_push(ShadowRam::default()).unwrap();
        }

        Self {
            kmrn_lock_loss_workaround_enabled: Default::default(),
            shadow_ram,
            nvm_k1_enabled: Default::default(),
            eee_disable: Default::default(),
            eee_lp_ability: Default::default(),
            ulp_state: UlpState::Unknown,
        }
    }
}

impl VariantCommon for Context {
    fn set_reg(&self, mac_kind: MacType, reg: &mut Reg) {
        if mac_kind == MacType::e1000_pch2lan {
            if (reg.er32(FWSM) & ICH_FWSM_FW_VALID) > 0 {
                reg.pcim2pci_arbiter_wa = true;
            }
        }
    }

    fn init_mac_params(&mut self, board: &BoardInfo, reg: Reg) -> Result<MacInfo> {
        let mut mac = MacInfo::default();
        mac.ops = board.mac_ops.clone();
        mac.kind = board.mac;

        /* Set mta register count */
        mac.mta_reg_count = 32;
        // /* Set rar entry count */
        mac.rar_entry_count = ICH_RAR_ENTRIES;
        if mac.kind == MacType::e1000_ich8lan {
            mac.rar_entry_count -= 1;
        }

        mac.has_fwsm = true;

        mac.arc_subsystem_valid = false;

        mac.adaptive_ifs = true;

        // // TODO led

        if mac.kind >= MacType::e1000_pch_lpt {
            mac.rar_entry_count = PCH_LPT_RAR_ENTRIES;
            mac.ops.setup_physical_interface = SetupPhysicalInterface::CopperLinkPchLpt;
        }

        /* Enable PCS Lock-loss workaround for ICH8 */
        // if hw.mac.kind == MacType::e1000_ich8lan{
        //     if let BoardVariant

        //     set_kmrn_lock_loss_workaround_ich8lan(hw);
        // }
        Ok(mac)
    }

    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo> {
        let media_type = MediaType::media_type_copper;
        let addr = 1;
        let reset_delay_us = 100;
        let autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT;
        let id = 0;
        let mac_kind = mac.kind;

        let mut phy = PhyInfo {
            mac_kind,
            reg,
            ops: board.phy_ops.clone(),
            media_type,
            kind: PhyType::Unknown,
            addr,
            id,
            reset_delay_us,
            revision: 0,
            autoneg_advertised: 0,
            autoneg_mask,
            cable_length: 0,
            max_cable_length: 0,
            min_cable_length: 0,
            mdix: 0,
            disable_polarity_correction: false,
            is_mdix: false,
            polarity_correction: false,
            speed_downgraded: false,
            autoneg_wait_to_complete: false,
            ms_type: Default::default(),
            original_ms_type: Default::default(),
            access_shared_resource: Default::default(),
        };

        self.init_phy_workarounds(&phy);

        phy.set_mdio_slow_mode_hv()?;

        phy.get_phy_id()?;

        phy.kind = phy.get_phy_type_from_id(phy.id);

        Ok(phy)
    }

    fn init_nvm_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo> {
        let mut nvm = NvmInfo::default();
        nvm.ops = board.nvm_ops.clone();
        nvm.reg = reg;
        let mut nvm_size = 0;

        nvm.kind = NvmType::FlashSw;

        if mac.kind >= MacType::e1000_pch_spt {
            nvm.flash_base_addr = 0;
            let strap_value = reg.er32(STRAP);
            nvm_size = (((strap_value >> 1) & 0x1F) + 1) * NVM_SIZE_MULTIPLIER;

            nvm.flash_bank_size = nvm_size / 2;

            nvm.flash_bank_size /= (size_of::<u16>() as u32);

            nvm.reg.flash = Some(reg.mmio + FLASH_BASE_ADDR);
        } else {
            if reg.flash.is_none() {
                return Err(Error::Config("Flash registers not mapped"));
            }
        }

        nvm.word_size = ICH8_SHADOW_RAM_WORDS as _;

        /* Clear shadow ram */
        for i in 0..nvm.word_size as usize {
            self.shadow_ram[i].modified = false;
            self.shadow_ram[i].value = 0xFFFF;
        }

        Ok(nvm)
    }

    fn reset_hw(&self, e1000: &E1000) -> Result {
        let mut res = e1000.reg.disable_pcie_master();
        if res.is_err() {
            error!("PCI-E Master disable polling has failed.");
        }
        info!("Masking off all interrupts");

        e1000.ew32(IMC, 0xffffffff);

        e1000.ew32(RCTL, 0);

        e1000.ew32(TCTL, TCTL_PSP);
        e1000.e1e_flush();

        kfn().delay(Duration::from_micros(10000));

        let mut ctrl = e1000.er32(CTRL);

        // if (!hw->phy.ops.check_reset_block(hw)) {
        // 	/* Full-chip reset requires MAC and PHY reset at the same
        // 	 * time to make sure the interface between MAC and the
        // 	 * external PHY is reset.
        // 	 */
        if self.check_reset_block(e1000.reg.clone()).is_ok() {
            ctrl |= CTRL_PHY_RST;
        }

        // 	/* Gate automatic PHY configuration by hardware on
        // 	 * non-managed 82579
        // 	 */
        // 	if ((hw->mac.type == e1000_pch2lan) &&
        // 	    !(er32(FWSM) & E1000_ICH_FWSM_FW_VALID))
        // 		e1000_gate_hw_phy_config_ich8lan(hw, true);
        // }
        // ret_val = e1000_acquire_swflag_ich8lan(hw);
        info!("Issuing a global reset to ich8lan");
        e1000.ew32(CTRL, (ctrl | CTRL_RST));
        /* cannot issue a flush here because it hangs the hardware */

        kfn().delay(Duration::from_micros(20_000));

        if (ctrl & CTRL_PHY_RST) > 0 {
            info!("Phy reset");
            e1000.phy.get_cfg_done()?;
            self.post_phy_reset(e1000)?;
            info!("Phy reset ok");
        }
        // /* For PCH, this write will make sure that any noise
        //  * will be detected as a CRC error and be dropped rather than show up
        //  * as a bad packet to the DMA engine.
        //  */
        // if (hw->mac.type == e1000_pchlan)
        // 	ew32(CRC_OFFSET, 0x65656565);

        e1000.ew32(IMC, 0xffffffff);
        e1000.er32(ICR);

        let mut reg = e1000.er32(KABGTXD);
        reg |= KABGTXD_BGSQLBIAS;
        e1000.ew32(KABGTXD, reg);

        Ok(())
    }

    fn init_hw(&self, e1000: &crate::E1000) -> Result {
        todo!()
    }
}

impl Context {
    fn gate_hw_phy_config(&mut self, reg: Reg, gate: bool) {
        let mut extcnf_ctrl = reg.er32(EXTCNF_CTRL);

        if (gate) {
            extcnf_ctrl |= EXTCNF_CTRL_GATE_PHY_CFG;
        } else {
            extcnf_ctrl &= !EXTCNF_CTRL_GATE_PHY_CFG;
        }

        reg.ew32(EXTCNF_CTRL, extcnf_ctrl);
    }

    fn phy_check_accessible(&mut self, phy: &PhyInfo) -> Result {
        let reg = phy.reg;
        let mut phy_lock = phy.acquire()?;
        if phy_lock.is_accessible() {
            return Ok(());
        }

        info!("PHY access is disabled");
        let mut mac_reg = reg.er32(CTRL_EXT);
        mac_reg |= CTRL_EXT_FORCE_SMBUS;
        reg.ew32(CTRL_EXT, mac_reg);
        kfn().delay(Duration::from_millis(50));
        if phy_lock.is_accessible() {
            return Ok(());
        }

        if self.check_reset_block(reg).is_err() {
            error!("Required LANPHYPC toggle blocked by ME");
            return Err(Error::Unknown("Required LANPHYPC toggle blocked by ME"));
        }
        self.toggle_lanphypc_pch_lpt(phy_lock.mac_kind, reg);

        if phy_lock.is_accessible() {
            return Ok(());
        }

        error!("LANPHYPC toggle failed");

        return Err(Error::Unknown("LANPHYPC toggle failed"));
    }
    /**
     *  post_phy_reset_ich8lan - Perform steps required after a PHY reset
     *  @hw: pointer to the HW structure
     **/
    fn post_phy_reset(&self, e1000: &E1000) -> Result {
        info!("post phy reset");

        if self.check_reset_block(e1000.reg.clone()).is_err() {
            info!("reset block");
            return Ok(());
        }

        /* Allow time for h/w to get to quiescent state after reset */
        kfn().delay(Duration::from_micros(10000));

        /* Perform any necessary post-reset workarounds */

        /* Clear the host wakeup bit after lcd reset */
        if (e1000.mac.kind >= MacType::e1000_pchlan) {
            let mut reg = e1000.e1e_rphy(BM_PORT_GEN_CFG)?;
            reg &= !BM_WUC_HOST_WU_BIT;
            e1000.e1e_wphy(BM_PORT_GEN_CFG, reg)?;
        }

        /* Configure the LCD with the extended configuration region in NVM */
        self.sw_lcd_config(e1000)?;

        /* Configure the LCD with the OEM bits in NVM */
        self.oem_bits_config(e1000, true)?;

        Ok(())
    }
    fn sw_lcd_config(&self, e1000: &E1000) -> Result {
        let mut phy_page = 0;
        let mut sw_cfg_mask = FEXTNVM_SW_CONFIG_ICH8M;

        let mut data = e1000.er32(FEXTNVM);
        if data & sw_cfg_mask == 0 {
            return Ok(());
        }

        /* Make sure HW does not configure LCD from PHY
         * extended configuration before SW configuration
         */
        data = e1000.er32(EXTCNF_CTRL);

        let mut cnf_size = e1000.er32(EXTCNF_SIZE);
        cnf_size &= EXTCNF_SIZE_EXT_PCIE_LENGTH_MASK;
        cnf_size >>= EXTCNF_SIZE_EXT_PCIE_LENGTH_SHIFT;
        if cnf_size == 0 {
            return Ok(());
        }

        let mut cnf_base_addr = data & EXTCNF_CTRL_EXT_CNF_POINTER_MASK;
        cnf_base_addr >>= EXTCNF_CTRL_EXT_CNF_POINTER_SHIFT;

        /* Configure LCD from extended configuration region. */

        /* cnf_base_addr is in DWORD */
        let mut word_addr = (cnf_base_addr << 1) as u16;

        for i in 0..cnf_size as u16 {
            let mut reg_data = e1000.read_nvm((word_addr + i * 2), 1)?[0];

            let mut reg_addr = e1000.read_nvm((word_addr + i * 2 + 1), 1)?[0];

            /* Save off the PHY page for future writes. */
            if (reg_addr as u32 == IGP01E1000_PHY_PAGE_SELECT) {
                phy_page = reg_data;
                continue;
            }

            reg_addr &= PHY_REG_MASK;
            reg_addr |= phy_page;

            e1000.e1e_wphy(reg_addr as u32, reg_data);
        }
        Ok(())
    }
    fn oem_bits_config(&self, e1000: &E1000, d0_state: bool) -> Result {
        if (e1000.mac.kind < MacType::e1000_pchlan) {
            return Ok(());
        }
        let mut mac_reg = e1000.er32(FEXTNVM);
        if (mac_reg & FEXTNVM_SW_CONFIG_ICH8M) == 0 {
            return Ok(());
        }

        mac_reg = e1000.er32(PHY_CTRL);

        let mut oem_reg = e1000.e1e_rphy(HV_OEM_BITS)?;

        oem_reg &= !(HV_OEM_BITS_GBE_DIS | HV_OEM_BITS_LPLU);

        if d0_state {
            if (mac_reg & PHY_CTRL_GBE_DISABLE) > 0 {
                oem_reg |= HV_OEM_BITS_GBE_DIS;
            }

            if (mac_reg & PHY_CTRL_D0A_LPLU) > 0 {
                oem_reg |= HV_OEM_BITS_LPLU;
            }
        } else {
            if (mac_reg & (PHY_CTRL_GBE_DISABLE | PHY_CTRL_NOND0A_GBE_DISABLE)) > 0 {
                oem_reg |= HV_OEM_BITS_GBE_DIS;
            }

            if (mac_reg & (PHY_CTRL_D0A_LPLU | PHY_CTRL_NOND0A_LPLU)) > 0 {
                oem_reg |= HV_OEM_BITS_LPLU;
            }
        }

        /* Set Restart auto-neg to activate the bits */
        if ((d0_state || (e1000.mac.kind != MacType::e1000_pchlan))
            && self.check_reset_block(e1000.reg.clone()).is_ok())
        {
            oem_reg |= HV_OEM_BITS_RESTART_AN;
        }

        e1000.e1e_wphy(HV_OEM_BITS, oem_reg)?;

        Ok(())
    }
    fn init_phy_workarounds(&mut self, phy: &PhyInfo) -> Result {
        let reg = phy.reg;
        let fwsm = reg.er32(FWSM);

        self.gate_hw_phy_config(reg, true);

        self.ulp_state = UlpState::Unknown;

        let ret = self.phy_check_accessible(phy);

        if ret.is_err() {
            if self.check_reset_block(reg).is_err() {
                error!("ME blocked access to PHY after reset");
                return Err(Error::Unknown("check_reset_block failed"));
            }

            phy.hw_reset_generic()?;

            if self.check_reset_block(reg).is_err() {
                error!("ME blocked access to PHY after reset");
                return Err(Error::Unknown("check_reset_block failed"));
            }
        }

        Ok(())
    }

    fn toggle_lanphypc_pch_lpt(&mut self, mac_kind: MacType, reg: Reg) {
        /* Set Phy Config Counter to 50msec */
        let mut mac_reg = reg.er32(FEXTNVM3);
        mac_reg &= !FEXTNVM3_PHY_CFG_COUNTER_MASK;
        mac_reg |= FEXTNVM3_PHY_CFG_COUNTER_50MSEC;
        reg.ew32(FEXTNVM3, mac_reg);

        /* Toggle LANPHYPC Value bit */
        mac_reg = reg.er32(CTRL);
        mac_reg |= CTRL_LANPHYPC_OVERRIDE;
        mac_reg &= !CTRL_LANPHYPC_VALUE;
        reg.ew32(CTRL, mac_reg);
        reg.e1e_flush();
        kfn().delay(Duration::from_micros(20));
        mac_reg &= !CTRL_LANPHYPC_OVERRIDE;
        reg.ew32(CTRL, mac_reg);
        reg.e1e_flush();

        if (mac_kind < MacType::e1000_pch_lpt) {
            kfn().delay(Duration::from_millis(50));
        } else {
            for _ in 0..20 {
                kfn().delay(Duration::from_micros(5000));
                if reg.er32(CTRL_EXT) & CTRL_EXT_LPCD > 0 {
                    break;
                }
            }

            kfn().delay(Duration::from_millis(30));
        }
    }

    /**
     *  e1000_check_reset_block_ich8lan - Check if PHY reset is blocked
     *  @hw: pointer to the HW structure
     *
     *  Checks if firmware is blocking the reset of the PHY.
     *  This is a function pointer entry point only called by
     *  reset routines.
     **/
    fn check_reset_block(&self, reg: Reg) -> Result {
        for _ in 0..30 {
            let blocked = (reg.er32(FWSM) & ICH_FWSM_RSPCIPHY) == 0;

            if !blocked {
                return Ok(());
            }
            kfn().delay(Duration::from_micros(10000));
        }

        Err(Error::Unknown("PHY RESET BLOCKED"))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PhySemaphore {
    reg: Reg,
}

impl PhySemaphore {
    pub(crate) fn new(reg: Reg) -> Self {
        Self { reg }
    }
}

impl crate::lock::SemaphoreBackend for PhySemaphore {
    fn acquire(&self) -> Result {
        let mut extcnf_ctrl = 0;
        let mut timeout = true;
        for _ in 0..100 {
            extcnf_ctrl = self.reg.er32(EXTCNF_CTRL);

            if extcnf_ctrl & EXTCNF_CTRL_SWFLAG == 0 {
                timeout = false;
                break;
            }

            kfn().delay(Duration::from_millis(1));
        }

        if timeout {
            error!("SW has already locked the resource.");
            return Err(Error::Config("SW has already locked the resource."));
        }
        extcnf_ctrl |= EXTCNF_CTRL_SWFLAG;
        self.reg.ew32(EXTCNF_CTRL, extcnf_ctrl);

        timeout = true;

        for _ in 0..20 {
            extcnf_ctrl = self.reg.er32(EXTCNF_CTRL);

            if extcnf_ctrl & EXTCNF_CTRL_SWFLAG > 0 {
                timeout = false;
                break;
            }

            kfn().delay(Duration::from_millis(1));
        }

        if timeout {
            error!(
                "Failed to acquire the semaphore, FW or HW has it: FWSM={:#x} EXTCNF_CTRL={:#x})",
                self.reg.er32(FWSM),
                extcnf_ctrl
            );
            extcnf_ctrl &= !EXTCNF_CTRL_SWFLAG;
            self.reg.ew32(EXTCNF_CTRL, extcnf_ctrl);

            return Err(Error::Config("Failed to acquire the semaphore"));
        }

        Ok(())
    }

    fn release(&self) {
        let mut extcnf_ctrl = self.reg.er32(EXTCNF_CTRL);

        if (extcnf_ctrl & EXTCNF_CTRL_SWFLAG) > 0 {
            extcnf_ctrl &= !EXTCNF_CTRL_SWFLAG;
            self.reg.ew32(EXTCNF_CTRL, extcnf_ctrl);
        } else {
            error!("Semaphore unexpectedly released by sw/fw/hw");
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct NvmSemaphore {
    reg: Reg,
}

impl NvmSemaphore {
    pub(crate) fn new(reg: Reg) -> Self {
        Self { reg }
    }
}

impl crate::lock::SemaphoreBackend for NvmSemaphore {
    fn acquire(&self) -> Result {
        Ok(())
    }

    fn release(&self) {}
}

/**
 *  lan_init_done_ich8lan - Check for PHY config completion
 *  @hw: pointer to the HW structure
 *
 *  Check the appropriate indication the MAC has finished configuring the
 *  PHY after a software reset.
 **/
pub(crate) fn lan_init_done(reg: Reg) {
    /* Wait for basic configuration completes before proceeding */
    for _ in 0..ICH8_LAN_INIT_TIMEOUT {
        let mut data = reg.er32(STATUS);
        data &= STATUS_LAN_INIT_DONE;

        kfn().delay(Duration::from_micros(100));

        if data > 0 {
            /* Clear the Init Done bit for the next init event */
            data = reg.er32(STATUS);
            data &= !STATUS_LAN_INIT_DONE;
            reg.ew32(STATUS, data);
            return;
        }
    }
    /* If basic configuration is incomplete before the above loop
     * count reaches 0, loading the configuration from NVM will
     * leave the PHY in a bad state possibly resulting in no link.
     */

    error!("LAN_INIT_DONE not set, increase timeout");
}
