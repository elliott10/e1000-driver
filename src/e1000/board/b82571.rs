use super::{super::defines::*, NvmOverride};
use super::{BusWidth, Hw};
use crate::e1000::board::NvmType;
use crate::{e1000::regs::*, err::*, KernelFunc, MediaType};
use crate::{MacType, PhyType};
use alloc::vec::Vec;

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
        self.init_phy_params(hw)?;
        Ok(())
    }

    fn init_mac_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        hw.phy.media_type = MediaType::media_type_copper;

        /* Set mta register count */
        hw.mac.mta_reg_count = 128;
        // /* Set rar entry count */
        hw.mac.rar_entry_count = RAR_ENTRIES as _;

        hw.mac.has_fwsm = false;

        hw.mac.adaptive_ifs = true;

        let mut force_clear_smbi = true;

        match hw.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                let swsm2 = hw.er32(SWSM2);

                if (swsm2 & SWSM2_LOCK) == 0 {
                    /* Only do this for the first interface on this card */
                    hw.ew32(SWSM2, swsm2 | SWSM2_LOCK);
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

        let mut eerd = 0u32;

        for i in 0..words {
            eerd = (((offset + i) as u32) << NVM_RW_ADDR_SHIFT) + NVM_RW_REG_START;

            hw.ew32(EERD, eerd);

            hw.poll_eerd_eewr_done(NVM_POLL_READ);

            let d = hw.er32(EERD) >> NVM_RW_REG_DATA;

            data.try_push(d as u16)?;
        }

        Ok(data)
    }

    pub(crate) fn reset_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut res = hw.disable_pcie_master();
        if res.is_err() {
            error!("PCI-E Master disable polling has failed.");
        }
        debug!("Masking off all interrupts");

        hw.ew32(IMC, 0xffffffff);

        hw.ew32(RCTL, 0);

        let mut tctl = hw.er32(TCTL);

        tctl &= !TCTL_EN;

        hw.ew32(TCTL, tctl);

        hw.e1e_flush();

        hw.kernel.delay(10000_000);

        res = match hw.mac.kind {
            MacType::e1000_82573 => self.get_hw_semaphore_82573(hw),
            MacType::e1000_82574 | MacType::e1000_82583 => self.get_hw_semaphore_82574(hw),
            _ => Ok(()), // Default case - no action needed or handle error
        };

        let ctrl = hw.er32(CTRL);

        debug!("Issuing a global reset to MAC");

        hw.ew32(CTRL, ctrl | CTRL_RST);

        if res.is_ok() {
            match hw.mac.kind {
                MacType::e1000_82573 => self.put_hw_semaphore_82573(hw),
                MacType::e1000_82574 | MacType::e1000_82583 => self.put_hw_semaphore_82574(hw),
                _ => {} // Default case - no action needed or handle error
            }
        }

        if hw.nvm.kind == NvmType::FlashHw {
            hw.kernel.delay(10_000);
            let mut ctrl_ext = hw.er32(CTRL_EXT);
            ctrl_ext |= CTRL_EXT_EE_RST;
            hw.ew32(CTRL_EXT, ctrl_ext);
            hw.e1e_flush();
        }

        hw.get_auto_rd_done()?;

        // Assuming ew32 andhw.er32 are functions defined elsewhere to write and read from the given register addresses respectively.

        match hw.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                // REQ and GNT bits need to be cleared when using AUTO_RD to access the EEPROM.
                let mut eecd = hw.er32(EECD);
                eecd &= !(EECD_REQ | EECD_GNT);
                hw.ew32(EECD, eecd);
            }
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                // Delay for 25 milliseconds to allow Phy configuration to complete.
                hw.kernel.delay(25_000_000);
            }
            _ => {} // Handle other cases or do nothing
        }

        /* Clear any pending interrupt events. */
        hw.ew32(IMC, 0xffffffff);
        hw.er32(ICR);

        /* Reinitialize the 82571 serdes link state machine */
        // if (hw.phy.media_type == MediaType::media_type_internal_serdes) {
        // hw.mac.l = e1000_serdes_link_down;
        // }

        Ok(())
    }

    fn get_hw_semaphore_82573<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        Ok(())
    }
    fn get_hw_semaphore_82574<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        Ok(())
    }

    pub(crate) fn put_hw_semaphore_82571<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        let mut swsm = hw.er32(SWSM);
        swsm &= !(SWSM_SMBI | SWSM_SWESMBI);
        hw.ew32(SWSM, swsm);
    }
    fn put_hw_semaphore_82573<K: KernelFunc>(&self, hw: &mut Hw<K>) {}
    fn put_hw_semaphore_82574<K: KernelFunc>(&self, hw: &mut Hw<K>) {}

    pub(crate) fn init_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        self.init_hw_bits(hw);
        //TODO init led

        /* Disabling VLAN filtering */
        debug!("Initializing the IEEE VLAN");
        self.clear_vfta(hw);
        let mut rar_count = hw.mac.rar_entry_count;

        if self.get_laa_state(hw) {
            rar_count -= 1;
        }

        hw.init_rx_addrs(rar_count);

        /* Zero out the Multicast HASH table */
        info!("Zeroing the MTA\n");
        for i in 0..hw.mac.mta_reg_count {
            hw.write_reg_array(MTA, i as usize, 0);
        }
        Ok(())
    }

    fn init_hw_bits<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        let mut reg = hw.er32(TXDCTL(0));
        reg |= (1 << 22);

        hw.ew32(TXDCTL(0), reg);

        /* Transmit Descriptor Control 1 */
        reg = hw.er32(TXDCTL(1));
        reg |= (1 << 22);
        hw.ew32(TXDCTL(1), reg);

        /* Transmit Arbitration Control 0 */
        reg = hw.er32(TARC(0));
        reg &= !(0xF << 27); /* 30:27 */

        match hw.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                reg |= BIT(23) | BIT(24) | BIT(25) | BIT(26);
            }
            MacType::e1000_82574 | MacType::e1000_82583 => {
                reg |= BIT(26);
            }
            _ => (),
        }

        hw.ew32(TARC(0), reg);
        /* Transmit Arbitration Control 1 */
        reg = hw.er32(TARC(1));
        match hw.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                reg &= !(BIT(29) | BIT(30));
                reg |= BIT(22) | BIT(24) | BIT(25) | BIT(26);
                if (hw.er32(TCTL) & TCTL_MULR) > 0 {
                    reg &= !BIT(28);
                } else {
                    reg |= BIT(28);
                }
                hw.ew32(TARC(1), reg);
            }
            _ => (),
        }
        /* Device Control */
        match hw.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                reg = hw.er32(CTRL);
                reg &= !BIT(29);
                hw.ew32(CTRL, reg);
            }
            _ => (),
        }

        /* Extended Device Control */
        match hw.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                let mut reg = hw.er32(CTRL_EXT);
                reg &= !BIT(23);
                reg |= BIT(22);
                hw.ew32(CTRL_EXT, reg);
            }
            _ => {}
        }

        if hw.mac.kind == MacType::e1000_82571 {
            let mut reg = hw.er32(PBA_ECC);
            reg |= PBA_ECC_CORR_EN;
            hw.ew32(PBA_ECC, reg);
        }

        // Workaround for hardware errata on 82571 and 82572
        if [MacType::e1000_82571, MacType::e1000_82572].contains(&hw.mac.kind) {
            let mut reg = hw.er32(CTRL_EXT);
            reg &= !CTRL_EXT_DMA_DYN_CLK_EN;
            hw.ew32(CTRL_EXT, reg);
        }

        // Disable IPv6 extension header parsing
        if hw.mac.kind <= MacType::e1000_82573 {
            let mut reg = hw.er32(RFCTL);
            reg |= RFCTL_IPV6_EX_DIS | RFCTL_NEW_IPV6_EXT_DIS;
            hw.ew32(RFCTL, reg);
        }

        // PCI-Ex Control Registers for specific types
        match hw.mac.kind {
            MacType::e1000_82574 | MacType::e1000_82583 => {
                let mut reg = hw.er32(GCR);
                reg |= BIT(22);
                hw.ew32(GCR, reg);

                let mut reg = hw.er32(GCR2);
                reg |= 1;
                hw.ew32(GCR2, reg);
            }
            _ => (),
        }
    }

    fn clear_vfta<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        let mut vfta_value = 0;
        let mut vfta_offset = 0;
        let mut vfta_bit_in_reg = 0;
        match hw.mac.kind {
            MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                if hw.mng_cookie.vlan_id != 0 {
                    // The VFTA is a 4096b bit-field, each identifying a single VLAN ID.
                    // Determine which 32b entry (offset) into the array to set the VLAN ID (bit) of the manageability unit.
                    vfta_offset = (hw.mng_cookie.vlan_id >> VFTA_ENTRY_SHIFT) & VFTA_ENTRY_MASK;
                    vfta_bit_in_reg =
                        BIT((hw.mng_cookie.vlan_id & VFTA_ENTRY_BIT_SHIFT_MASK) as usize);
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
            hw.write_reg_array(VFTA, offset, vfta_value);
            hw.e1e_flush();
        }
    }

    /**
     *  e1000e_get_laa_state_82571 - Get locally administered address state
     *  @hw: pointer to the HW structure
     *
     *  Retrieve and return the current locally administered address state.
     **/
    fn get_laa_state<K: KernelFunc>(&self, hw: &mut Hw<K>) -> bool {
        if hw.mac.kind != MacType::e1000_82571 {
            return false;
        }

        return self.laa_is_present;
    }

    fn init_phy_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        if (hw.phy.media_type != MediaType::media_type_copper) {
            hw.phy.kind = PhyType::None;
            return Ok(());
        }

        hw.phy.addr = 1;
        hw.phy.autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT;
        hw.phy.reset_delay_us = 100;

        match hw.mac.kind {
            MacType::e1000_82571 | MacType::e1000_82572 => {
                hw.phy.kind = PhyType::Igp2;
            }
            MacType::e1000_82573 => hw.phy.kind = PhyType::M88,
            MacType::e1000_82574 | MacType::e1000_82583 => hw.phy.kind = PhyType::Bm,
            _ => return Err(Error::Unknown("Error getting PHY")),
        }

        Ok(())
    }
}
