use super::{super::defines::*, NvmOverride};
use super::{BusWidth, Hw};
use crate::e1000::board::NvmType;
use crate::MacType;
use crate::{e1000::regs::*, err::*, KernelFunc, MediaType};

#[derive(Default)]
pub(crate) struct DevSpec82571 {
    pub(crate) laa_is_present: bool,
    pub(crate) smb_counter: u32,
}

impl DevSpec82571 {
    pub(crate) fn get_variants<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        info!("82571 variants");
        let is_port_b = hw.er32(STATUS) & STATUS_FUNC_1;

        self.init_mac_params(hw)?;
        self.init_nvm_params(hw)?;
        Ok(())
    }

    fn init_mac_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        hw.phy.media_type = MediaType::media_type_copper;

        /* Set mta register count */
        hw.mac.mta_reg_count = 128;
        // /* Set rar entry count */
        hw.mac.rar_entry_count = RAR_ENTRIES as _;

        hw.mac.has_fwsm = true;

        hw.mac.adaptive_ifs = true;

        let force_clear_smbi = true;

        if force_clear_smbi {
            /* Make sure SWSM.SMBI is clear */
            let swsm = hw.er32(SWSM);
            if (swsm & SWSM_SMBI > 0) {
                /* This bit should not be set on a first interface, and
                 * indicates that the bootagent or EFI code has
                 * improperly left this bit enabled
                 */
                error!("Please update your 82571 Bootagent");
            }
            hw.ew32(SWSM, swsm & (!SWSM_SMBI));
        }

        self.smb_counter = 0;
        Ok(())
    }
    fn init_nvm_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let mut eecd = hw.er32(EECD);
        hw.nvm.opcode_bits = 8;
        hw.nvm.delay_usec = 1;

        match hw.nvm.nvm_override {
            NvmOverride::SpiSmall => {
                hw.nvm.page_size = 8;
                hw.nvm.address_bits = 8;
            }
            NvmOverride::SpiLarge => {
                hw.nvm.page_size = 32;
                hw.nvm.address_bits = 16;
            }
            _ => {
                hw.nvm.page_size = if eecd & EECD_ADDR_BITS != 0 { 32 } else { 8 };
                hw.nvm.address_bits = if eecd & EECD_ADDR_BITS != 0 { 16 } else { 8 };
            }
        }
        let mut is_set = false;
        match hw.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                if (eecd >> 15) & 0x3 == 0x3 {
                    hw.nvm.kind = NvmType::FlashHw;
                    hw.nvm.word_size = 2048;
                    // Clear the Autonomous Flash update bit due to Flash update issue.
                    eecd &= !EECD_AUPDEN;
                    hw.ew32(EECD, eecd); // Assuming ew32! is a macro for writing to a register
                    is_set = true;
                }
            }
            _ => {}
        }

        if !is_set {
            hw.nvm.kind = NvmType::EepromSpi;
            let size = ((eecd & EECD_SIZE_EX_MASK) >> EECD_SIZE_EX_SHIFT) as u16;
            // Adding to a constant, "size" becomes the left-shift value for setting word_size.
            let mut size = size + NVM_WORD_SIZE_BASE_SHIFT;

            // EEPROM access above 16k is unsupported
            if size > 14 {
                size = 14;
            }
            hw.nvm.word_size = 1 << size;
        }

        Ok(())
    }

    pub(crate) fn get_bus_info<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let bus_info = hw.kernel.pcie_info();
        hw.bus.width = unsafe { *(&bus_info.bus_link_width as *const u16 as *const BusWidth) };
        info!("Bus link width {:?}", hw.bus.width);
        Ok(())
    }
}
