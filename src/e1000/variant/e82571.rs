use core::time::Duration;

use super::{VariantCommon, VariantResult};

use crate::e1000::mac::*;
use crate::e1000::nvm::*;
use crate::e1000::reg;
use crate::e1000::reg::*;
use crate::e1000::hw::FcMode;
use crate::e1000::kfn;
use crate::e1000::lock::{Semaphore, SemaphoreBackend};
use crate::e1000::phy::{self, PhyInfo};
use crate::e1000::E1000;
use crate::e1000::{defines::*, err::*, phy::*};
use crate::e1000::{BoardInfo, Flags};

#[derive(Default, Debug, Clone)]
pub(crate) struct Context {
    laa_is_present: bool,
    smb_counter: u32,
}

impl VariantCommon for Context {
    fn set_reg(&self, _mac_kind: MacType, _reg: &mut Reg) {}

    fn init_mac_params(&mut self, board: &BoardInfo, reg: Reg) -> Result<MacInfo> {
        let mut mac = MacInfo::default();
        mac.ops = board.mac_ops.clone();
        mac.kind = board.mac;
        /* Set mta register count */
        mac.mta_reg_count = 128;
        // /* Set rar entry count */
        mac.rar_entry_count = RAR_ENTRIES as _;

        mac.has_fwsm = false;

        mac.adaptive_ifs = true;

        let mut force_clear_smbi = true;

        match mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                let swsm2 = reg.er32(SWSM2);

                if (swsm2 & SWSM2_LOCK) == 0 {
                    /* Only do this for the first interface on this card */
                    reg.ew32(SWSM2, swsm2 | SWSM2_LOCK);
                    force_clear_smbi = true;
                } else {
                    force_clear_smbi = false;
                }
            }
            _ => {
                force_clear_smbi = true;
            }
        }

        if force_clear_smbi {
            /* Make sure SWSM.SMBI is clear */
            let swsm = reg.er32(SWSM);
            if (swsm & SWSM_SMBI > 0) {
                /* This bit should not be set on a first interface, and
                 * indicates that the bootagent or EFI code has
                 * improperly left this bit enabled
                 */
                error!("Please update your 82571 Bootagent");
            }
            reg.ew32(SWSM, swsm & (!SWSM_SMBI));
        }

        self.smb_counter = 0;
        Ok(mac)
    }

    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo> {
        let media_type = MediaType::media_type_copper;
        let mac_kind = mac.kind;
        let mut id = 0;
        let autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT;
        let reset_delay_us = 100;

        let kind = match mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => PhyType::Igp2,
            MacType::e1000_82573 => PhyType::M88,
            MacType::e1000_82574 | MacType::e1000_82583 => PhyType::Bm,
            _ => return Err(Error::Phy("Error getting PHY")),
        };

        Ok(PhyInfo {
            mac_kind,
            reg,
            ops: board.phy_ops.clone(),
            media_type,
            kind,
            addr: 1,
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
        })
    }

    fn init_nvm_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo> {
        let mut nvm = NvmInfo::default();
        nvm.ops = board.nvm_ops.clone();
        nvm.reg = reg;
        let mut eecd = reg.er32(EECD);
        nvm.opcode_bits = 8;
        nvm.delay_usec = 1;

        match nvm.nvm_override {
            NvmOverride::SpiSmall => {
                nvm.page_size = 8;
                nvm.address_bits = 8;
            }
            NvmOverride::SpiLarge => {
                nvm.page_size = 32;
                nvm.address_bits = 16;
            }
            _ => {
                nvm.page_size = if eecd & EECD_ADDR_BITS != 0 { 32 } else { 8 };
                nvm.address_bits = if eecd & EECD_ADDR_BITS != 0 { 16 } else { 8 };
            }
        }
        let mut is_set = false;
        match mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                if (eecd >> 15) & 0x3 == 0x3 {
                    nvm.kind = NvmType::FlashHw;
                    nvm.word_size = 2048;
                    // Clear the Autonomous Flash update bit due to Flash update issue.
                    eecd &= !EECD_AUPDEN;
                    reg.ew32(EECD, eecd); // Assuming ew32! is a macro for writing to a register
                    is_set = true;
                }
            }
            _ => {}
        }

        if !is_set {
            nvm.kind = NvmType::EepromSpi;
            let size = ((eecd & EECD_SIZE_EX_MASK) >> EECD_SIZE_EX_SHIFT) as u16;
            // Adding to a constant, "size" becomes the left-shift value for setting word_size.
            let mut size = size + NVM_WORD_SIZE_BASE_SHIFT;

            // EEPROM access above 16k is unsupported
            if size > 14 {
                size = 14;
            }
            nvm.word_size = 1 << size;
        }

        Ok(nvm)
    }

    fn reset_hw(&self, e1000: &E1000) -> Result {
        let reg = e1000.reg;
        let mut res = reg.disable_pcie_master();
        if res.is_err() {
            error!("PCI-E Master disable polling has failed.");
        }
        debug!("Masking off all interrupts");

        reg.ew32(IMC, 0xffffffff);

        reg.ew32(RCTL, 0);

        let mut tctl = reg.er32(TCTL);

        tctl &= !TCTL_EN;

        reg.ew32(TCTL, tctl);

        reg.e1e_flush();

        kfn().delay(Duration::from_micros(10000));

        let ctrl = reg.er32(CTRL);

        debug!("Issuing a global reset to MAC");

        reg.ew32(CTRL, ctrl | CTRL_RST);

        if e1000.nvm.kind == NvmType::FlashHw {
            kfn().delay(Duration::from_micros(10));
            let mut ctrl_ext = reg.er32(CTRL_EXT);
            ctrl_ext |= CTRL_EXT_EE_RST;
            reg.ew32(CTRL_EXT, ctrl_ext);
            reg.e1e_flush();
        }

        reg.get_auto_rd_done()?;

        // Assuming ew32 andreg.er32 are functions defined elsewhere to write and read from the given register addresses respectively.

        match &e1000.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                // REQ and GNT bits need to be cleared when using AUTO_RD to access the EEPROM.
                let mut eecd = reg.er32(EECD);
                eecd &= !(EECD_REQ | EECD_GNT);
                reg.ew32(EECD, eecd);
            }
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                // Delay for 25 milliseconds to allow Phy configuration to complete.
                kfn().delay(Duration::from_millis(25));
            }
            _ => {} // Handle other cases or do nothing
        }

        /* Clear any pending interrupt events. */
        reg.ew32(IMC, 0xffffffff);
        reg.er32(ICR);

        Ok(())
    }

    fn init_hw(&self, e1000: &E1000) -> Result {
        self.init_hw_bits(e1000);

        //TODO init led

        /* Disabling VLAN filtering */
        debug!("Initializing the IEEE VLAN");
        self.clear_vfta(e1000);
        let mut rar_count = e1000.mac.rar_entry_count;

        if self.get_laa_state(e1000) {
            rar_count -= 1;
        }

        e1000.init_rx_addrs(rar_count);

        /* Zero out the Multicast HASH table */
        info!("Zeroing the MTA");
        for i in 0..e1000.mac.mta_reg_count {
            e1000.reg.write_reg_array(MTA, i as _, 0);
        }
        Ok(())
    }
}

impl Context {
    fn init_hw_bits(&self, e1000: &E1000) {
        let mut reg = e1000.er32(TXDCTL(0));
        reg |= (1 << 22);

        e1000.ew32(TXDCTL(0), reg);

        /* Transmit Descriptor Control 1 */
        reg = e1000.er32(TXDCTL(1));
        reg |= (1 << 22);
        e1000.ew32(TXDCTL(1), reg);

        /* Transmit Arbitration Control 0 */
        reg = e1000.er32(TARC(0));
        reg &= !(0xF << 27); /* 30:27 */

        match e1000.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                reg |= BIT(23) | BIT(24) | BIT(25) | BIT(26);
            }
            MacType::e1000_82574 | MacType::e1000_82583 => {
                reg |= BIT(26);
            }
            _ => (),
        }

        e1000.ew32(TARC(0), reg);
        /* Transmit Arbitration Control 1 */
        reg = e1000.er32(TARC(1));
        match e1000.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                reg &= !(BIT(29) | BIT(30));
                reg |= BIT(22) | BIT(24) | BIT(25) | BIT(26);
                if (e1000.er32(TCTL) & TCTL_MULR) > 0 {
                    reg &= !BIT(28);
                } else {
                    reg |= BIT(28);
                }
                e1000.ew32(TARC(1), reg);
            }
            _ => (),
        }
        /* Device Control */
        match e1000.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                reg = e1000.er32(CTRL);
                reg &= !BIT(29);
                e1000.ew32(CTRL, reg);
            }
            _ => (),
        }

        /* Extended Device Control */
        match e1000.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                let mut reg = e1000.er32(CTRL_EXT);
                reg &= !BIT(23);
                reg |= BIT(22);
                e1000.ew32(CTRL_EXT, reg);
            }
            _ => {}
        }

        if e1000.mac.kind == MacType::e1000_82571 {
            let mut reg = e1000.er32(PBA_ECC);
            reg |= PBA_ECC_CORR_EN;
            e1000.ew32(PBA_ECC, reg);
        }

        // Workaround for hardware errata on 82571 and 82572
        if [MacType::e1000_82571, MacType::e1000_82572].contains(&e1000.mac.kind) {
            let mut reg = e1000.er32(CTRL_EXT);
            reg &= !CTRL_EXT_DMA_DYN_CLK_EN;
            e1000.ew32(CTRL_EXT, reg);
        }

        // Disable IPv6 extension header parsing
        if e1000.mac.kind <= MacType::e1000_82573 {
            let mut reg = e1000.er32(RFCTL);
            reg |= RFCTL_IPV6_EX_DIS | RFCTL_NEW_IPV6_EXT_DIS;
            e1000.ew32(RFCTL, reg);
        }

        // PCI-Ex Control Registers for specific types
        match e1000.mac.kind {
            MacType::e1000_82574 | MacType::e1000_82583 => {
                let mut reg = e1000.er32(GCR);
                reg |= BIT(22);
                e1000.ew32(GCR, reg);

                let mut reg = e1000.er32(GCR2);
                reg |= 1;
                e1000.ew32(GCR2, reg);
            }
            _ => (),
        }
    }
    /**
     *  e1000e_get_laa_state_82571 - Get locally administered address state
     *  @hw: pointer to the HW structure
     *
     *  Retrieve and return the current locally administered address state.
     **/
    fn get_laa_state(&self, e1000: &E1000) -> bool {
        if e1000.mac.kind != MacType::e1000_82571 {
            return false;
        }

        return self.laa_is_present;
    }
    fn clear_vfta(&self, e1000: &E1000) {
        let mut vfta_value = 0;
        let mut vfta_offset = 0;
        let mut vfta_bit_in_reg = 0;
        match e1000.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                if e1000.mng_cookie.vlan_id != 0 {
                    // The VFTA is a 4096b bit-field, each identifying a single VLAN ID.
                    // Determine which 32b entry (offset) into the array to set the VLAN ID (bit) of the manageability unit.
                    vfta_offset = (e1000.mng_cookie.vlan_id >> VFTA_ENTRY_SHIFT) & VFTA_ENTRY_MASK;
                    vfta_bit_in_reg =
                        BIT((e1000.mng_cookie.vlan_id & VFTA_ENTRY_BIT_SHIFT_MASK) as usize);
                }
            }
            _ => {}
        }

        for offset in 0..VLAN_FILTER_TBL_SIZE {
            // If the offset we want to clear is the same offset of the manageability VLAN ID,
            // then clear all bits except that of the manageability unit.
            let vfta_value = if offset as u16 == vfta_offset {
                vfta_bit_in_reg
            } else {
                0
            };
            e1000.reg.write_reg_array(VFTA, offset as _, vfta_value);
            e1000.e1e_flush();
        }
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

impl crate::e1000::lock::SemaphoreBackend for PhySemaphore {
    fn acquire(&self) -> Result {
        Ok(())
    }

    fn release(&self) {}
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

impl crate::e1000::lock::SemaphoreBackend for NvmSemaphore {
    fn acquire(&self) -> Result {
        Ok(())
    }

    fn release(&self) {}
}
