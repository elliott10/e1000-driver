use super::super::regs::*;
use super::BusWidth;
use super::DevSpec;
use super::NvmInfo;
use super::NvmType;
use super::{DevSpecParam, Hw};
use crate::defines::*;
use crate::e1000::board::FcMode;
use crate::err::*;
use crate::phy::*;
use crate::BoardVariant;
use crate::KernelFunc;
use crate::MacType;
use crate::MediaType;
use crate::PhyInfo;
use crate::PhyType;
use crate::E1000;
use alloc::vec::Vec;
use core::mem::size_of;

const E1000_ICH_RAR_ENTRIES: u16 = 7;
const E1000_PCH2_RAR_ENTRIES: u16 = 5; /* RAR[0], SHRA[0-3] */
const E1000_PCH_LPT_RAR_ENTRIES: u16 = 12; /* RAR[0], SHRA[0-10] */

const NVM_SIZE_MULTIPLIER: u32 = 4096; // multiplier for NVMS field
const E1000_FLASH_BASE_ADDR: usize = 0xE000; // offset of NVM access regs
const E1000_CTRL_EXT_NVMVS: u32 = 0x3; // NVM valid sector

// TARC0 constants with bit masks
const E1000_TARC0_CB_MULTIQ_3_REQ: u32 = 0x30000000;
const E1000_TARC0_CB_MULTIQ_2_REQ: u32 = 0x20000000;

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
    pub(crate) value: u16,
    pub(crate) modified: bool,
}
const E1000_ICH8_SHADOW_RAM_WORDS: usize = 2048;

/* I218 PHY Ultra Low Power (ULP) states */
#[derive(Debug)]
pub(crate) enum UlpState {
    Unknown,
    Off,
    On,
}

#[derive(Debug)]
pub(crate) struct DevSpecIch8lan {
    pub(crate) kmrn_lock_loss_workaround_enabled: bool,
    pub(crate) shadow_ram: Vec<ShadowRam>,
    pub(crate) nvm_k1_enabled: bool,
    pub(crate) eee_disable: bool,
    pub(crate) eee_lp_ability: u16,
    pub ulp_state: UlpState,
}

impl Default for DevSpecIch8lan {
    fn default() -> Self {
        let mut shadow_ram = Vec::try_with_capacity(E1000_ICH8_SHADOW_RAM_WORDS).unwrap();
        for _ in 0..E1000_ICH8_SHADOW_RAM_WORDS {
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

impl DevSpecIch8lan {
    pub(crate) fn get_variants<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        info!("Ich8lan init");
        self.init_mac_params(hw)?;
        self.init_nvm_params(hw)?;
        self.init_phy_params(hw)?;
        info!(
            "Phy params ok, phy {:?}, address {}",
            hw.phy.kind, hw.phy.addr
        );
        Ok(())
    }

    fn init_mac_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        hw.phy.media_type = MediaType::media_type_copper;
        /* Set mta register count */
        hw.mac.mta_reg_count = 32;
        // /* Set rar entry count */
        hw.mac.rar_entry_count = E1000_ICH_RAR_ENTRIES;
        if hw.mac.kind == MacType::e1000_ich8lan {
            hw.mac.rar_entry_count -= 1;
        }

        hw.mac.has_fwsm = true;

        hw.mac.arc_subsystem_valid = false;

        hw.mac.adaptive_ifs = true;

        // // TODO led

        if hw.mac.kind >= MacType::e1000_pch_lpt {
            hw.mac.rar_entry_count = E1000_PCH_LPT_RAR_ENTRIES;
        }

        /* Enable PCS Lock-loss workaround for ICH8 */
        // if hw.mac.kind == MacType::e1000_ich8lan{
        //     if let BoardVariant

        //     set_kmrn_lock_loss_workaround_ich8lan(hw);
        // }

        Ok(())
    }
    fn init_nvm_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let mut nvm_size = 0;

        hw.nvm.kind = NvmType::FlashSw;

        if hw.mac.kind >= MacType::e1000_pch_spt {
            hw.nvm.flash_base_addr = 0;
            let strap_value = hw.er32(STRAP);
            nvm_size = (((strap_value >> 1) & 0x1F) + 1) * NVM_SIZE_MULTIPLIER;

            hw.nvm.flash_bank_size = nvm_size / 2;

            hw.nvm.flash_bank_size /= (size_of::<u16>() as u32);

            hw.flash = Some(hw.mmio + E1000_FLASH_BASE_ADDR);
        } else {
            if hw.flash.is_none() {
                return Err(Error::Config("Flash registers not mapped"));
            }
        }

        hw.nvm.word_size = E1000_ICH8_SHADOW_RAM_WORDS as _;

        /* Clear shadow ram */
        for i in 0..hw.nvm.word_size as usize {
            self.shadow_ram[i].modified = false;
            self.shadow_ram[i].value = 0xFFFF;
        }

        Ok(())
    }

    fn init_phy_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        if (hw.phy.media_type != MediaType::media_type_copper) {
            hw.phy.kind = PhyType::None;
            return Ok(());
        }

        hw.phy.addr = 1;
        hw.phy.reset_delay_us = 100;
        hw.phy.autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT;
        hw.phy.id = 0;

        self.init_phy_workarounds(hw)?;

        self.set_mdio_slow_mode_hv(hw)?;

        hw.get_phy_id()?;

        hw.phy.kind = hw.get_phy_type_from_id(hw.phy.id);

        Ok(())
    }

    fn init_phy_workarounds<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let fwsm = hw.er32(FWSM);

        self.gate_hw_phy_config(hw, true);

        self.ulp_state = UlpState::Unknown;

        if !self.phy_is_accessible(hw) {
            info!("PHY access is disabled");
            let mut mac_reg = hw.er32(CTRL_EXT);
            mac_reg |= CTRL_EXT_FORCE_SMBUS;
            hw.ew32(CTRL_EXT, mac_reg);

            hw.udelay(50_000);

            if !self.phy_is_accessible(hw) {
                if self.check_reset_block(hw).is_err() {
                    error!("Required LANPHYPC toggle blocked by ME");
                    return Err(Error::Unknown("Required LANPHYPC toggle blocked by ME"));
                }
                self.toggle_lanphypc_pch_lpt(hw);

                if !self.phy_is_accessible(hw) {
                    error!("LANPHYPC toggle failed");
                    return Err(Error::Unknown("LANPHYPC toggle failed"));
                }
            }
        }

        self.phy_hw_reset_generic(hw)?;

        if self.check_reset_block(hw).is_err() {
            error!("ME blocked access to PHY after reset");
            return Err(Error::Unknown("check_reset_block failed"));
        }

        Ok(())
    }
    fn toggle_lanphypc_pch_lpt<K: KernelFunc>(&mut self, hw: &mut Hw<K>) {
        /* Set Phy Config Counter to 50msec */
        let mut mac_reg = hw.er32(FEXTNVM3);
        mac_reg &= !FEXTNVM3_PHY_CFG_COUNTER_MASK;
        mac_reg |= FEXTNVM3_PHY_CFG_COUNTER_50MSEC;
        hw.ew32(FEXTNVM3, mac_reg);

        /* Toggle LANPHYPC Value bit */
        mac_reg = hw.er32(CTRL);
        mac_reg |= CTRL_LANPHYPC_OVERRIDE;
        mac_reg &= !CTRL_LANPHYPC_VALUE;
        hw.ew32(CTRL, mac_reg);
        hw.e1e_flush();
        hw.udelay(20);
        mac_reg &= !CTRL_LANPHYPC_OVERRIDE;
        hw.ew32(CTRL, mac_reg);
        hw.e1e_flush();

        if (hw.mac.kind < MacType::e1000_pch_lpt) {
            hw.udelay(50_000);
        } else {
            for _ in 0..20 {
                hw.udelay(5000);
                if hw.er32(CTRL_EXT) & CTRL_EXT_LPCD > 0 {
                    break;
                }
            }

            hw.udelay(30_000);
        }
    }

    /**
     *  e1000_set_mdio_slow_mode_hv - Set slow MDIO access mode
     *  @hw:   pointer to the HW structure
     **/
    fn set_mdio_slow_mode_hv<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let mut data = hw.e1e_rphy(HV_KMRN_MODE_CTRL)?;

        data |= HV_KMRN_MDIO_SLOW;

        hw.e1e_wphy(HV_KMRN_MODE_CTRL, data)
    }

    fn phy_hw_reset_generic<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        return Ok(());

        if self.check_reset_block(hw).is_err() {
            return Ok(());
        }
        info!("PHY reset");

        let mut ctrl = hw.er32(CTRL);
        hw.ew32(CTRL, ctrl | CTRL_PHY_RST);
        hw.e1e_flush();

        hw.udelay(hw.phy.reset_delay_us);

        hw.ew32(CTRL, ctrl);
        hw.e1e_flush();

        hw.udelay(300);

        return self.get_cfg_done(hw);
    }

    /**
     *  e1000_phy_is_accessible_pchlan - Check if able to access PHY registers
     *  @hw: pointer to the HW structure
     *
     *  Test access to the PHY registers by reading the PHY ID registers.  If
     *  the PHY ID is already known (e.g. resume path) compare it with known ID,
     *  otherwise assume the read PHY ID is correct if it is valid.
     *
     *  Assumes the sw/fw/hw semaphore is already acquired.
     **/
    fn phy_is_accessible<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> bool {
        let mut phy_id = 0;
        let mut phy_reg = 0;

        for _ in 0..2 {
            phy_reg = if let Ok(phy_reg) = hw.e1e_rphy(MII_PHYSID1) {
                if phy_reg == 0xFFFF {
                    return continue;
                }
                phy_reg
            } else {
                return continue;
            };

            phy_id = (phy_reg as u32) << 16;

            phy_reg = if let Ok(phy_reg) = hw.e1e_rphy(MII_PHYSID2) {
                if phy_reg == 0xFFFF {
                    return continue;
                }
                phy_reg
            } else {
                return continue;
            };
            phy_id |= (phy_reg as u32 & PHY_REVISION_MASK);
            break;
        }

        let mut skip = false;

        if hw.phy.id > 0 {
            if (hw.phy.id == phy_id) {
                skip = true;
            }
        } else if phy_id > 0 {
            hw.phy.id = phy_id;
            hw.phy.revision = (phy_reg as u32 & !PHY_REVISION_MASK);
            skip = true;
        }

        if !skip {
            /* In case the PHY needs to be in mdio slow mode,
             * set slow mode and try to get the PHY id again.
             */
            // if (hw.mac.kind < MacType::e1000_pch_lpt) {
            // 	ret_val = e1000_set_mdio_slow_mode_hv(hw);
            // 	if (!ret_val)
            // 		ret_val = e1000e_get_phy_id(hw);
            // }
            return false;
        }

        if (hw.mac.kind >= MacType::e1000_pch_lpt) {
            /* Only unforce SMBus if ME is not active */
            if hw.er32(FWSM) & ICH_FWSM_FW_VALID == 0 {
                /* Unforce SMBus mode in PHY */
                phy_reg = match hw.e1e_rphy(CV_SMB_CTRL) {
                    Ok(d) => d,
                    Err(_) => return false,
                };
                phy_reg &= !CV_SMB_CTRL_FORCE_SMBUS;
                if hw.e1e_wphy(CV_SMB_CTRL, phy_reg).is_err() {
                    return false;
                };

                /* Unforce SMBus mode in MAC */
                let mut mac_reg = hw.er32(CTRL_EXT);
                mac_reg &= !CTRL_EXT_FORCE_SMBUS;
                hw.ew32(CTRL_EXT, mac_reg);
            }
        }

        return true;
    }

    fn gate_hw_phy_config<K: KernelFunc>(&mut self, hw: &mut Hw<K>, gate: bool) {
        if (hw.mac.kind < MacType::e1000_pch2lan) {
            return;
        }

        let mut extcnf_ctrl = hw.er32(EXTCNF_CTRL);

        if (gate) {
            extcnf_ctrl |= EXTCNF_CTRL_GATE_PHY_CFG;
        } else {
            extcnf_ctrl &= !EXTCNF_CTRL_GATE_PHY_CFG;
        }

        hw.ew32(EXTCNF_CTRL, extcnf_ctrl);
    }

    /**
     *  e1000e_set_kmrn_lock_loss_workaround_ich8lan - Set Kumeran workaround state
     *  @hw: pointer to the HW structure
     *  @state: boolean value used to set the current Kumeran workaround state
     *
     *  If ICH8, set the current Kumeran workaround state (enabled - true
     *  /disabled - false).
     **/
    fn set_kmrn_lock_loss_workaround_ich8lan<K: KernelFunc>(
        &mut self,
        base: &mut Hw<K>,
        state: bool,
    ) {
        self.kmrn_lock_loss_workaround_enabled = state;
    }

    pub(crate) fn init_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        self.init_hw_bits(hw);
        //TODO init led

        hw.init_rx_addrs(hw.mac.rar_entry_count);

        /* Zero out the Multicast HASH table */
        info!("Zeroing the MTA");
        for i in 0..hw.mac.mta_reg_count {
            hw.write_reg_array(MTA, i as usize, 0);
        }

        let ret = self.setup_link(hw);

        /* Set the transmit descriptor write-back policy for both queues */
        let mut txdctl = hw.er32(TXDCTL(0));
        txdctl = ((txdctl & !TXDCTL_WTHRESH) | TXDCTL_FULL_TX_DESC_WB);
        txdctl = ((txdctl & !TXDCTL_PTHRESH) | TXDCTL_MAX_TX_DESC_PREFETCH);
        hw.ew32(TXDCTL(0), txdctl);
        txdctl = hw.er32(TXDCTL(1));
        txdctl = ((txdctl & !TXDCTL_WTHRESH) | TXDCTL_FULL_TX_DESC_WB);
        txdctl = ((txdctl & !TXDCTL_PTHRESH) | TXDCTL_MAX_TX_DESC_PREFETCH);
        hw.ew32(TXDCTL(1), txdctl);

        /* ICH8 has opposite polarity of no_snoop bits.
         * By default, we should use snoop behavior.
         */
        let snoop = if (hw.mac.kind == MacType::e1000_ich8lan) {
            PCIE_ICH8_SNOOP_ALL
        } else {
            !(PCIE_NO_SNOOP_ALL)
        };
        self.set_pcie_no_snoop(hw, snoop);
        /* Enable workaround for packet loss issue on TGP PCH
         * Do not gate DMA clock from the modPHY block
         */
        if (hw.mac.kind >= MacType::e1000_pch_tgp) {
            let mut fflt_dbg = hw.er32(FFLT_DBG);
            fflt_dbg |= FFLT_DBG_DONT_GATE_WAKE_DMA_CLK;
            hw.ew32(FFLT_DBG, fflt_dbg);
        }

        let mut ctrl_ext = hw.er32(CTRL_EXT);
        ctrl_ext |= CTRL_EXT_RO_DIS;
        hw.ew32(CTRL_EXT, ctrl_ext);

        /* Clear all of the statistics registers (clear on read).  It is
         * important that we do this after we have tried to establish link
         * because the symbol error count will increment wildly if there
         * is no link.
         */
        self.clear_hw_cntrs(hw);

        ret
    }
    pub(crate) fn setup_link<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        /* ICH parts do not have a word in the NVM to determine
         * the default flow control setting, so we explicitly
         * set it to full.
         */
        if (hw.fc.requested_mode == FcMode::Default) {
            /* Workaround h/w hang when Tx flow control enabled */
            if (hw.mac.kind == MacType::e1000_pchlan) {
                hw.fc.requested_mode = FcMode::RxPause;
            } else {
                hw.fc.requested_mode = FcMode::Full;
            }
        }

        hw.fc.requested_mode = FcMode::None;
        /* Save off the requested flow control mode for use later.  Depending
         * on the link partner's capabilities, we may or may not use this mode.
         */
        hw.fc.current_mode = hw.fc.requested_mode;

        info!(
            "After fix-ups FlowControl is now = {:?}",
            hw.fc.current_mode
        );

        /* Continue to configure the copper link. */
        self.setup_copper_link_pch_lpt(hw)?;

        hw.ew32(FCTTV, hw.fc.pause_time as u32);
        if ((hw.phy.kind == PhyType::T82578)
            || (hw.phy.kind == PhyType::T82579)
            || (hw.phy.kind == PhyType::I217)
            || (hw.phy.kind == PhyType::T82577))
        {
            hw.ew32(FCRTV_PCH, hw.fc.refresh_time as u32);

            hw.e1e_wphy(PHY_REG(BM_PORT_CTRL_PAGE, 27), hw.fc.pause_time)?;
        }

        return hw.set_fc_watermarks();

        Ok(())
    }

    fn close_fc<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        info!("close fc");

        // No flow control
        hw.ew32(FCAH, 0);
        hw.ew32(FCAL, 0);
        hw.ew32(FCT, 0);
        hw.ew32(FCTTV, 0);
    }

    fn sw_lcd_config<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut phy_page = 0;
        let mut sw_cfg_mask = FEXTNVM_SW_CONFIG_ICH8M;

        let mut data = hw.er32(FEXTNVM);
        if data & sw_cfg_mask == 0 {
            return Ok(());
        }

        /* Make sure HW does not configure LCD from PHY
         * extended configuration before SW configuration
         */
        data = hw.er32(EXTCNF_CTRL);

        let mut cnf_size = hw.er32(EXTCNF_SIZE);
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
            let mut reg_data = self.read_nvm(hw, (word_addr + i * 2), 1)?[0];

            let mut reg_addr = self.read_nvm(hw, (word_addr + i * 2 + 1), 1)?[0];

            /* Save off the PHY page for future writes. */
            if (reg_addr as u32 == IGP01E1000_PHY_PAGE_SELECT) {
                phy_page = reg_data;
                continue;
            }

            reg_addr &= PHY_REG_MASK;
            reg_addr |= phy_page;

            hw.e1e_wphy(reg_addr as u32, reg_data);
        }
        Ok(())
    }

    fn init_hw_bits<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        /* Extended Device Control */
        let mut reg = hw.er32(CTRL_EXT);
        reg |= BIT(22);
        /* Enable PHY low-power state when MAC is at D3 w/o WoL */
        if (hw.mac.kind >= MacType::e1000_pchlan) {
            reg |= CTRL_EXT_PHYPDEN;
        }
        hw.ew32(CTRL_EXT, reg);

        /* Transmit Descriptor Control 0 */
        reg = hw.er32(TXDCTL(0));
        reg |= BIT(22);
        hw.ew32(TXDCTL(0), reg);

        /* Transmit Descriptor Control 1 */
        reg = hw.er32(TXDCTL(1));
        reg |= BIT(22);
        hw.ew32(TXDCTL(1), reg);

        /* Transmit Arbitration Control 0 */
        reg = hw.er32(TARC(0));
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg |= BIT(28) | BIT(29);
        }
        reg |= BIT(23) | BIT(24) | BIT(26) | BIT(27);
        hw.ew32(TARC(0), reg);

        /* Transmit Arbitration Control 1 */
        reg = hw.er32(TARC(1));
        if (hw.er32(TCTL) & TCTL_MULR) > 0 {
            reg &= !BIT(28);
        } else {
            reg |= BIT(28);
        }
        reg |= BIT(24) | BIT(26) | BIT(30);
        hw.ew32(TARC(1), reg);

        /* Device Status */
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg = hw.er32(STATUS);
            reg &= !BIT(31);
            hw.ew32(STATUS, reg);
        }

        /* work-around descriptor data corruption issue during nfs v2 udp
         * traffic, just disable the nfs filtering capability
         */
        reg = hw.er32(RFCTL);
        reg |= (RFCTL_NFSW_DIS | RFCTL_NFSR_DIS);

        /* Disable IPv6 extension header parsing because some malformed
         * IPv6 headers can hang the Rx.
         */
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg |= (RFCTL_IPV6_EX_DIS | RFCTL_NEW_IPV6_EXT_DIS);
        }
        hw.ew32(RFCTL, reg);

        /* Enable ECC on Lynxpoint */
        if (hw.mac.kind >= MacType::e1000_pch_lpt) {
            reg = hw.er32(PBECCSTS);
            reg |= PBECCSTS_ECC_ENABLE;
            hw.ew32(PBECCSTS, reg);

            reg = hw.er32(CTRL);
            reg |= CTRL_MEHE;
            hw.ew32(CTRL, reg);
        }
    }
    pub(crate) fn setup_copper_link_pch_lpt<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        info!("Setup CTRL SLU");
        let mut ctrl = hw.er32(CTRL);
        ctrl |= CTRL_SLU;
        ctrl &= !(CTRL_FRCSPD | CTRL_FRCDPX);
        hw.ew32(CTRL, ctrl);
        hw.e1e_flush();

        self.close_fc(hw);

        hw.print_status();
        self.copper_link_setup_82577(hw)?;

        hw.setup_copper_link()
    }

    fn copper_link_setup_82577<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut phy_data = hw.e1e_rphy(I82577_CFG_REG)?;

        /* Enable CRS on Tx. This must be set for half-duplex operation. */
        phy_data |= I82577_CFG_ASSERT_CRS_ON_TX;

        /* Enable downshift */
        phy_data |= I82577_CFG_ENABLE_DOWNSHIFT;

        hw.e1e_wphy(I82577_CFG_REG, phy_data)?;

        /* Set MDI/MDIX mode */
        phy_data = hw.e1e_rphy(I82577_PHY_CTRL_2)?;
        phy_data &= !I82577_PHY_CTRL2_MDIX_CFG_MASK;
        /* Options:
         *   0 - Auto (default)
         *   1 - MDI mode
         *   2 - MDI-X mode
         */
        match hw.phy.mdix {
            1 => {}
            2 => {
                phy_data |= I82577_PHY_CTRL2_MANUAL_MDIX;
            }
            _ => {
                phy_data |= I82577_PHY_CTRL2_AUTO_MDI_MDIX;
            }
        }
        hw.e1e_wphy(I82577_PHY_CTRL_2, phy_data)?;

        hw.set_master_slave_mode()
    }

    /**
     *  e1000e_set_pcie_no_snoop - Set PCI-express capabilities
     *  @hw: pointer to the HW structure
     *  @no_snoop: bitmap of snoop events
     *
     *  Set the PCI-express register to snoop for events enabled in 'no_snoop'.
     **/
    fn set_pcie_no_snoop<K: KernelFunc>(&self, hw: &mut Hw<K>, no_snoop: u32) {
        if (no_snoop > 0) {
            let mut gcr = hw.er32(GCR);
            gcr &= !(PCIE_NO_SNOOP_ALL);
            gcr |= no_snoop;
            hw.ew32(GCR, gcr);
        }
    }

    fn clear_hw_cntrs<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        hw.clear_hw_cntrs_base();

        hw.er32(ALGNERRC);
        hw.er32(RXERRC);
        hw.er32(TNCRS);
        hw.er32(CEXTERR);
        hw.er32(TSCTC);
        hw.er32(TSCTFC);

        hw.er32(MGTPRC);
        hw.er32(MGTPDC);
        hw.er32(MGTPTC);

        hw.er32(IAC);
        hw.er32(ICRXOC);
    }
    /**
     *  e1000_check_reset_block_ich8lan - Check if PHY reset is blocked
     *  @hw: pointer to the HW structure
     *
     *  Checks if firmware is blocking the reset of the PHY.
     *  This is a function pointer entry point only called by
     *  reset routines.
     **/
    fn check_reset_block<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        for _ in 0..30 {
            let blocked = (hw.er32(FWSM) & ICH_FWSM_RSPCIPHY) == 0;

            if !blocked {
                return Ok(());
            }
            hw.udelay(10000);
        }

        Err(Error::Unknown("PHY RESET BLOCKED"))
    }
    pub(crate) fn reset_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut res = hw.disable_pcie_master();
        if res.is_err() {
            error!("PCI-E Master disable polling has failed.");
        }
        info!("Masking off all interrupts");

        hw.ew32(IMC, 0xffffffff);

        hw.ew32(RCTL, 0);

        hw.ew32(TCTL, TCTL_PSP);
        hw.e1e_flush();

        hw.udelay(10000);

        let mut ctrl = hw.er32(CTRL);

        // if (!hw->phy.ops.check_reset_block(hw)) {
        // 	/* Full-chip reset requires MAC and PHY reset at the same
        // 	 * time to make sure the interface between MAC and the
        // 	 * external PHY is reset.
        // 	 */
        if self.check_reset_block(hw).is_ok() {
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
        hw.ew32(CTRL, (ctrl | CTRL_RST));
        /* cannot issue a flush here because it hangs the hardware */

        hw.udelay(20_000);

        if (ctrl & CTRL_PHY_RST) > 0 {
            info!("Phy reset");
            self.get_cfg_done(hw)?;
            self.post_phy_reset(hw)?;
            info!("Phy reset ok");
        }
        // /* For PCH, this write will make sure that any noise
        //  * will be detected as a CRC error and be dropped rather than show up
        //  * as a bad packet to the DMA engine.
        //  */
        // if (hw->mac.type == e1000_pchlan)
        // 	ew32(CRC_OFFSET, 0x65656565);

        hw.ew32(IMC, 0xffffffff);
        hw.er32(ICR);

        let mut reg = hw.er32(KABGTXD);
        reg |= KABGTXD_BGSQLBIAS;
        hw.ew32(KABGTXD, reg);

        Ok(())
    }
    /**
     *  e1000_post_phy_reset_ich8lan - Perform steps required after a PHY reset
     *  @hw: pointer to the HW structure
     **/
    fn post_phy_reset<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        info!("post phy reset");

        if self.check_reset_block(hw).is_err() {
            info!("reset block");
            return Ok(());
        }

        /* Allow time for h/w to get to quiescent state after reset */
        hw.udelay(10000);

        /* Perform any necessary post-reset workarounds */

        /* Clear the host wakeup bit after lcd reset */
        if (hw.mac.kind >= MacType::e1000_pchlan) {
            let mut reg = hw.e1e_rphy(BM_PORT_GEN_CFG)?;
            reg &= !BM_WUC_HOST_WU_BIT;
            hw.e1e_wphy(BM_PORT_GEN_CFG, reg)?;
        }

        /* Configure the LCD with the extended configuration region in NVM */
        self.sw_lcd_config(hw)?;

        /* Configure the LCD with the OEM bits in NVM */
        self.oem_bits_config(hw, true)?;

        Ok(())
    }

    fn oem_bits_config<K: KernelFunc>(&self, hw: &mut Hw<K>, d0_state: bool) -> Result {
        if (hw.mac.kind < MacType::e1000_pchlan) {
            return Ok(());
        }
        let mut mac_reg = hw.er32(FEXTNVM);
        if (mac_reg & FEXTNVM_SW_CONFIG_ICH8M) == 0 {
            return Ok(());
        }

        mac_reg = hw.er32(PHY_CTRL);

        let mut oem_reg = hw.e1e_rphy(HV_OEM_BITS)?;

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
        if ((d0_state || (hw.mac.kind != MacType::e1000_pchlan))
            && self.check_reset_block(hw).is_ok())
        {
            oem_reg |= HV_OEM_BITS_RESTART_AN;
        }

        hw.e1e_wphy(HV_OEM_BITS, oem_reg)?;

        Ok(())
    }

    /**
     *  e1000_lan_init_done_ich8lan - Check for PHY config completion
     *  @hw: pointer to the HW structure
     *
     *  Check the appropriate indication the MAC has finished configuring the
     *  PHY after a software reset.
     **/
    fn lan_init_done<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        /* Wait for basic configuration completes before proceeding */
        for _ in 0..ICH8_LAN_INIT_TIMEOUT {
            let mut data = hw.er32(STATUS);
            data &= STATUS_LAN_INIT_DONE;
            hw.udelay(100);

            if data > 0 {
                /* Clear the Init Done bit for the next init event */
                data = hw.er32(STATUS);
                data &= !STATUS_LAN_INIT_DONE;
                hw.ew32(STATUS, data);
                return;
            }
        }
        /* If basic configuration is incomplete before the above loop
         * count reaches 0, loading the configuration from NVM will
         * leave the PHY in a bad state possibly resulting in no link.
         */

        error!("LAN_INIT_DONE not set, increase timeout");
    }

    /**
     *  e1000_get_cfg_done_ich8lan - Read config done bit after Full or PHY reset
     *  @hw: pointer to the HW structure
     *
     *  Read appropriate register for the config done bit for completion status
     *  and configure the PHY through s/w for EEPROM-less parts.
     *
     *  NOTE: some silicon which is EEPROM-less will fail trying to read the
     *  config done bit, so only an error is logged and continues.  If we were
     *  to return with error, EEPROM-less silicon would not be able to be reset
     *  or change link.
     **/
    fn get_cfg_done<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        hw.udelay(20000);

        /* Wait for indication from h/w that it has completed basic config */
        if (hw.mac.kind >= MacType::e1000_ich10lan) {
            self.lan_init_done(hw);
        }

        info!("Clear PHY Reset Asserted bit");
        /* Clear PHY Reset Asserted bit */
        let mut status = hw.er32(STATUS);
        if (status & STATUS_PHYRA) > 0 {
            hw.ew32(STATUS, status & !STATUS_PHYRA);
        } else {
            warn!("PHY Reset Asserted not set - needs delay");
        }

        /* If EEPROM is not marked present, init the IGP 3 PHY manually */
        if (hw.mac.kind <= MacType::e1000_ich9lan) {
            // if (!(er32(EECD) & EECD_PRES) &&
            // (hw->phy.type == e1000_phy_igp_3)) {
            // e1000e_phy_init_script_igp3(hw);
            // }
            error!("hw.mac.kind <= MacType:: e1000_ich9lan");
        } else {
            self.valid_nvm_bank_detect(hw)?;
        }

        Ok(())
    }

    pub(crate) fn read_nvm<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        offset: u16,
        words: u16,
    ) -> Result<Vec<u16>> {
        if offset > hw.nvm.word_size || (words > (hw.nvm.word_size - offset)) || words == 0 {
            return Err(Error::Nvm("nvm parameter(s) out of bounds"));
        }

        let mut data: Vec<u16> = Vec::try_with_capacity(words as usize)?;
        for _ in 0..words {
            data.try_push(0).unwrap();
        }

        let res = self.valid_nvm_bank_detect(hw);
        let bank = match res {
            Ok(bank) => bank,
            Err(_) => 0,
        };

        let mut act_offset = if (bank) > 0 {
            hw.nvm.flash_bank_size
        } else {
            0
        };
        act_offset += offset as u32;

        let mut i = 0usize;
        let mut offset_to_read = 0;
        let words = words as usize;
        let offset = offset as usize;
        let act_offset = act_offset as usize;
        let mut dword = 0u32;

        while i < words {
            if (words - i == 1) {
                if (self.shadow_ram[offset + i].modified) {
                    data[i] = self.shadow_ram[offset + i].value;
                } else {
                    offset_to_read = act_offset + i - ((act_offset + i) % 2);

                    dword = self.read_flash_dword(hw, offset_to_read)?;

                    if ((act_offset + i) % 2 == 0) {
                        data[i] = (dword & 0xFFFF) as u16;
                    } else {
                        data[i] = ((dword >> 16) & 0xFFFF) as u16;
                    }
                }
            } else {
                offset_to_read = act_offset + i;
                if (!(self.shadow_ram[offset + i].modified)
                    || !(self.shadow_ram[offset + i + 1].modified))
                {
                    dword = self.read_flash_dword(hw, offset_to_read)?;
                }
                if (self.shadow_ram[offset + i].modified) {
                    data[i] = self.shadow_ram[offset + i].value;
                } else {
                    data[i] = (dword & 0xFFFF) as u16;
                }
                if (self.shadow_ram[offset + i].modified) {
                    data[i + 1] = self.shadow_ram[offset + i + 1].value;
                } else {
                    data[i + 1] = (dword >> 16 & 0xFFFF) as u16;
                }
            }

            i += 2;
        }

        Ok(data)
    }

    /**
     *  e1000_flash_cycle_init_ich8lan - Initialize flash
     *  @hw: pointer to the HW structure
     *
     *  This function does initial flash setup so that a new read/write/erase cycle
     *  can be started.
     **/
    pub(crate) fn flash_cycle_init<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;

        /* Check if the flash descriptor is valid */
        if (hsfsts & HSFSTS_FLDESVALID) == 0 {
            let msg = "Flash descriptor invalid.  SW Sequencing must be used.";
            error!("{}", msg);
            return Err(Error::Nvm(msg));
        }

        /* Clear FCERR and DAEL in hw status by writing 1 */

        hsfsts |= HSFSTS_FLCERR | HSFSTS_DAEL;

        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
        } else {
            hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
        }

        /* Either we should have a hardware SPI cycle in progress
         * bit to check against, in order to start a new cycle or
         * FDONE bit should be changed in the hardware so that it
         * is 1 after hardware reset, which can then be used as an
         * indication whether a cycle is in progress or has been
         * completed.
         */
        if (hsfsts & HSFSTS_FLCINPROG) == 0 {
            /* There is no cycle running at present,
             * so we can start a cycle.
             * Begin by setting Flash Cycle Done.
             */
            hsfsts |= HSFSTS_FLCDONE;
            if (hw.mac.kind >= MacType::e1000_pch_spt) {
                hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
            } else {
                hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
            }

            Ok(())
        } else {
            /* Otherwise poll for sometime so the current
             * cycle has a chance to end before giving up.
             */
            for _ in 0..ICH_FLASH_READ_COMMAND_TIMEOUT {
                hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCINPROG) == 0 {
                    /* Successful in waiting for previous cycle to timeout,
                     * now set the Flash Cycle Done.
                     */
                    hsfsts |= HSFSTS_FLCDONE;
                    if (hw.mac.kind >= MacType::e1000_pch_spt) {
                        hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
                    } else {
                        hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
                    }
                    break;
                }
                hw.kernel.delay(1000);
            }

            let msg = "Flash controller busy, cannot get access";
            error!("{}", msg);
            Err(Error::Nvm(msg))
        }
    }

    /**
     *  e1000_flash_cycle_ich8lan - Starts flash cycle (read/write/erase)
     *  @hw: pointer to the HW structure
     *  @timeout: maximum time to wait for completion
     *
     *  This function starts a flash cycle and waits for its completion.
     **/
    pub(crate) fn flash_cycle<K: KernelFunc>(&self, hw: &mut Hw<K>, timeout: u32) -> Result {
        let mut hsflctl: u16 = 0;
        let mut hsfsts: u16 = 0;

        /* Start a cycle by writing 1 in Flash Cycle Go in Hw Flash Control */
        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hsflctl = (hw.er32flash(ICH_FLASH_HSFSTS)? >> 16) as u16;
        } else {
            hsflctl = hw.er16flash(ICH_FLASH_HSFCTL)?;
        }
        hsflctl |= HSFCTL_FLCGO;

        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hw.ew32flash(ICH_FLASH_HSFSTS, (hsflctl as u32) << 16)?;
        } else {
            hw.ew16flash(ICH_FLASH_HSFCTL, hsflctl)?;
        }

        /* wait till FDONE bit is set to 1 */
        for i in 0..timeout {
            hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
            if hsfsts & HSFSTS_FLCDONE != 0 {
                break;
            }
            hw.kernel.delay(1000);
        }

        if (hsfsts & HSFSTS_FLCDONE) > 0 && (hsfsts & HSFSTS_FLCERR) == 0 {
            return Ok(());
        }

        return Err(Error::Nvm("timeout"));
    }

    /**
     *  e1000_read_flash_data32_ich8lan - Read dword from NVM
     *  @hw: pointer to the HW structure
     *  @offset: The offset (in bytes) of the dword to read.
     *  @data: Pointer to the dword to store the value read.
     *
     *  Reads a byte or word from the NVM using the flash access registers.
     **/
    pub(crate) fn read_flash_data32<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        offset: usize,
    ) -> Result<u32> {
        if (offset as u32 > ICH_FLASH_LINEAR_ADDR_MASK || hw.mac.kind < MacType::e1000_pch_spt) {
            return Err(Error::Nvm("not support"));
        }
        let flash_linear_addr =
            ((ICH_FLASH_LINEAR_ADDR_MASK & offset as u32) + hw.nvm.flash_base_addr);

        for _ in 0..ICH_FLASH_CYCLE_REPEAT_COUNT {
            hw.kernel.delay(1000);
            /* Steps */
            self.flash_cycle_init(hw)?;
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            let mut hsflctl: FlashCtrl = ((hw.er32flash(ICH_FLASH_HSFSTS)? >> 16) as u16).into();

            /* 0b/1b corresponds to 1 or 2 byte size, respectively. */
            hsflctl.set_count(size_of::<u32>() - 1);
            hsflctl.set_cycle(ICH_CYCLE_READ);
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            hw.ew32flash(ICH_FLASH_HSFSTS, (hsflctl.value as u32) << 16);
            hw.ew32flash(ICH_FLASH_FADDR, flash_linear_addr);

            let r = self.flash_cycle(hw, ICH_FLASH_READ_COMMAND_TIMEOUT);

            /* Check if FCERR is set to 1, if set to 1, clear it
             * and try the whole sequence a few more times, else
             * read in (shift in) the Flash Data0, the order is
             * least significant byte first msb to lsb
             */
            if (r.is_ok()) {
                return hw.er32flash(ICH_FLASH_FDATA0);
            } else {
                /* If we've gotten here, then things are probably
                 * completely hosed, but if the error condition is
                 * detected, it won't hurt to give it another try...
                 * ICH_FLASH_CYCLE_REPEAT_COUNT times.
                 */
                let hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCERR) > 0 {
                    /* Repeat for some time before giving up. */
                    continue;
                } else if (hsfsts & HSFSTS_FLCDONE) == 0 {
                    error!("Timeout error - flash cycle did not complete.");
                    break;
                }
            }
        }

        return Err(Error::Nvm("timeout"));
    }

    /**
     *  e1000_read_flash_dword_ich8lan - Read dword from flash
     *  @hw: pointer to the HW structure
     *  @offset: offset to data location
     *  @data: pointer to the location for storing the data
     *
     *  Reads the flash dword at offset into data.  Offset is converted
     *  to bytes before read.
     **/
    pub(crate) fn read_flash_dword<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        mut offset: usize,
    ) -> Result<u32> {
        /* Must convert word offset into bytes. */
        offset <<= 1;
        return self.read_flash_data32(hw, offset);
    }

    pub(crate) fn valid_nvm_bank_detect<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result<u32> {
        info!("valid_nvm_bank_detect");

        let bank1_offset = hw.nvm.flash_bank_size as usize;
        let act_offset = ICH_NVM_SIG_WORD;

        let mut bank = 0;

        /* Check bank 0 */
        let mut nvm_dword = self.read_flash_dword(hw, act_offset)?;

        let mut sig_byte = ((nvm_dword & 0xFF00) >> 8) as u8;
        if (sig_byte & ICH_NVM_VALID_SIG_MASK) == ICH_NVM_SIG_VALUE {
            return Ok(bank);
        }

        /* Check bank 1 */
        nvm_dword = self.read_flash_dword(hw, act_offset + bank1_offset)?;

        sig_byte = ((nvm_dword & 0xFF00) >> 8) as u8;
        if ((sig_byte & ICH_NVM_VALID_SIG_MASK) == ICH_NVM_SIG_VALUE) {
            bank = 1;
            return Ok(bank);
        }

        error!("ERROR: No valid NVM bank present");
        return Err(Error::Nvm("ERROR: No valid NVM bank present"));
    }

    /**
     *  e1000_read_flash_data32_ich8lan - Read dword from NVM
     *  @hw: pointer to the HW structure
     *  @offset: The offset (in bytes) of the dword to read.
     *  @data: Pointer to the dword to store the value read.
     *
     *  Reads a byte or word from the NVM using the flash access registers.
     **/
    pub(crate) fn read_flash_data<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        offset: usize,
        size: u8,
    ) -> Result<u16> {
        if (size < 1 || size > 2 || offset > ICH_FLASH_LINEAR_ADDR_MASK as usize) {
            return Err(Error::Nvm("size or offset out of range"));
        }

        let flash_linear_addr =
            ((ICH_FLASH_LINEAR_ADDR_MASK & offset as u32) + hw.nvm.flash_base_addr);

        for _ in 0..ICH_FLASH_CYCLE_REPEAT_COUNT {
            hw.kernel.delay(1000);
            /* Steps */
            self.flash_cycle_init(hw)?;
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            let mut hsflctl: FlashCtrl = hw.er16flash(ICH_FLASH_HSFSTS)?.into();

            /* 0b/1b corresponds to 1 or 2 byte size, respectively. */
            hsflctl.set_count(size as usize - 1);
            hsflctl.set_cycle(ICH_CYCLE_READ);
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            hw.ew16flash(ICH_FLASH_HSFSTS, hsflctl.value);
            hw.ew32flash(ICH_FLASH_FADDR, flash_linear_addr);

            let r = self.flash_cycle(hw, ICH_FLASH_READ_COMMAND_TIMEOUT);

            /* Check if FCERR is set to 1, if set to 1, clear it
             * and try the whole sequence a few more times, else
             * read in (shift in) the Flash Data0, the order is
             * least significant byte first msb to lsb
             */
            if (r.is_ok()) {
                let flash_data = hw.er32flash(ICH_FLASH_FDATA0)?;
                if size == 1 {
                    return Ok((flash_data & 0x000000FF) as u16);
                } else if size == 2 {
                    return Ok((flash_data & 0x0000FFFF) as u16);
                }
            } else {
                /* If we've gotten here, then things are probably
                 * completely hosed, but if the error condition is
                 * detected, it won't hurt to give it another try...
                 * ICH_FLASH_CYCLE_REPEAT_COUNT times.
                 */
                let hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCERR) > 0 {
                    /* Repeat for some time before giving up. */
                    continue;
                } else if (hsfsts & HSFSTS_FLCDONE) == 0 {
                    error!("Timeout error - flash cycle did not complete.");
                    break;
                }
            }
        }

        return Err(Error::Nvm("timeout"));
    }

    pub(crate) fn read_flash_word<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        mut offset: usize,
    ) -> Result<u16> {
        /* Must convert word offset into bytes. */
        offset <<= 1;
        return self.read_flash_data(hw, offset, 2);
    }

    pub(crate) fn get_bus_info<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let bus_info = hw.kernel.pcie_info();
        hw.bus.width = unsafe { *(&bus_info.bus_link_width as *const u16 as *const BusWidth) };

        /* ICH devices are "PCI Express"-ish.  They have
         * a configuration space, but do not contain
         * PCI Express Capability registers, so bus width
         * must be hardcoded.
         */
        if hw.bus.width == BusWidth::Unknown {
            hw.bus.width = BusWidth::PcieX1;
        }

        Ok(())
    }
}
