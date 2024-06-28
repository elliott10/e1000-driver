use super::{VariantCommon, VariantParam, VariantResult};
use crate::e1000::mac::MacInfo;
use crate::e1000::nvm::*;
use crate::e1000::reg::*;
use crate::phy::{self, PhyInfo2};
use crate::{defines::*, err::*, MacType, PhyType};
use crate::{BoardInfo, Flags, Flags2, MediaType};

#[derive(Default, Debug, Clone)]
pub(crate) struct Context {
    laa_is_present: bool,
    smb_counter: u32,
}

impl VariantCommon for Context {
    fn set_reg(&self, _mac_kind: crate::MacType, _reg: &mut Reg) {}

    fn init_mac_params(&mut self, mac_kind: crate::MacType, reg: Reg) -> Result<MacInfo> {
        let mut mac = MacInfo::default();
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

    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo2> {
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

        Ok(PhyInfo2 {
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

    fn init_nvm_params(&self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo> {
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

impl crate::lock::Semaphore for PhySemaphore {
    fn acquire(&self) -> crate::err::Result {
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

impl crate::lock::Semaphore for NvmSemaphore {
    fn acquire(&self) -> crate::err::Result {
        Ok(())
    }

    fn release(&self) {}
}
