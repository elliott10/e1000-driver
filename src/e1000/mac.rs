use super::reg::*;
use crate::{defines::*, err::*, hw::FcMode, phy::PhyInfo, MacAddress};
use alloc::vec::Vec;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum MacType {
    e1000_82571,
    e1000_82572,
    e1000_82573,
    e1000_82574,
    e1000_82583,
    e1000_80003es2lan,
    e1000_ich8lan,
    e1000_ich9lan,
    e1000_ich10lan,
    e1000_pchlan,
    e1000_pch2lan,
    e1000_pch_lpt,
    e1000_pch_spt,
    e1000_pch_cnp,
    e1000_pch_tgp,
    e1000_pch_adp,
    e1000_pch_mtp,
    e1000_pch_lnp,
    e1000_pch_ptp,
    e1000_pch_nvp,
}

impl Default for MacType {
    fn default() -> Self {
        Self::e1000_82574
    }
}

#[derive(Default)]
pub(crate) struct MacInfo {
    pub(crate) kind: MacType,
    pub(crate) ops: MacOps,
    pub reg: Reg,
    pub(crate) addr: [u8; 6],
    pub(crate) perm_addr: [u8; 6],

    pub(crate) collision_delta: u32,
    pub(crate) ledctl_default: u32,
    pub(crate) ledctl_mode1: u32,
    pub(crate) ledctl_mode2: u32,
    pub(crate) mc_filter_type: u32,
    pub(crate) tx_packet_delta: u32,
    pub(crate) txcw: u32,

    pub(crate) current_ifs_val: u16,
    pub(crate) ifs_max_val: u16,
    pub(crate) ifs_min_val: u16,
    pub(crate) ifs_ratio: u16,
    pub(crate) ifs_step_size: u16,
    pub(crate) mta_reg_count: u16,

    // Maximum size of the MTA register table in all supported adapters
    pub(crate) mta_shadow: Vec<u32>,
    pub(crate) rar_entry_count: u16,

    pub(crate) forced_speed_duplex: u8,

    pub(crate) adaptive_ifs: bool,
    pub(crate) has_fwsm: bool,
    pub(crate) arc_subsystem_valid: bool,
    pub(crate) autoneg: bool,
    pub(crate) autoneg_failed: bool,
    pub(crate) get_link_status: bool,
    pub(crate) in_ifs_mode: bool,
    pub(crate) serdes_has_link: bool,
    pub(crate) tx_pkt_filtering: bool,
}

impl MacInfo {
    pub fn read_mac_addr_generic(&mut self, reg: Reg) -> Result {
        let rar_high = reg.er32(RAH(0));
        let rar_low = reg.er32(RAL(0));

        for i in 0..RAL_MAC_ADDR_LEN {
            self.perm_addr[i] = (rar_low >> (i * 8)) as u8;
        }

        for i in 0..RAH_MAC_ADDR_LEN {
            self.perm_addr[i + 4] = (rar_high >> (i * 8)) as u8;
        }

        for i in 0..6 {
            self.addr[i] = self.perm_addr[i];
        }
        Ok(())
    }

    /**
     *  e1000e_rar_set_generic - Set receive address register
     *  @hw: pointer to the HW structure
     *  @addr: pointer to the receive address
     *  @index: receive address array register
     *
     *  Sets the receive address array register at index to the address passed
     *  in by addr.
     **/
    pub fn rar_set_pch_lpt(
        &self,
        addr: MacAddress,
        index: usize,
        reg: Reg,
        phy: &PhyInfo,
    ) -> Result {
        let index = index as u32;
        let rar_low = u32::from_le_bytes([addr.0[0], addr.0[1], addr.0[2], addr.0[3]]);
        let mut rar_high = u16::from_le_bytes([addr.0[4], addr.0[5]]) as u32;
        /* HW expects these in little endian so we reverse the byte order
         * from network order (big endian) to little endian
         */

        /* If MAC address zero, no need to set the AV bit */

        if rar_low > 0 || rar_high > 0 {
            rar_high |= RAH_AV;
        }

        if index == 0 {
            reg.ew32(RAL(index), rar_low);
            reg.e1e_flush();
            reg.ew32(RAH(index), rar_high);
            reg.e1e_flush();
            return Ok(());
        }
        if index < self.rar_entry_count as u32 {
            let mut wlock_mac = reg.er32(FWSM) & FWSM_WLOCK_MAC_MASK;
            wlock_mac >>= FWSM_WLOCK_MAC_SHIFT;

            // Check if all SHRAR registers are locked
            if wlock_mac == 1 {
                return Err(Error::Config("Failed to write receive address"));
            }

            if wlock_mac == 0 || index <= wlock_mac {
                {
                    let g = phy.acquire()?;

                    reg.ew32(SHRAL_PCH_LPT(index - 1), rar_low);
                    reg.e1e_flush();
                    reg.ew32(SHRAH_PCH_LPT(index - 1), rar_high);
                    reg.e1e_flush();

                    drop(g);
                }

                // Verify the register updates
                if reg.er32(SHRAL_PCH_LPT(index - 1)) == rar_low
                    && reg.er32(SHRAH_PCH_LPT(index - 1)) == rar_high
                {
                    return Ok(()); // Success
                }
            }
        }
        Err(Error::Config("Failed to write receive address"))
    }

    /**
     *  e1000e_rar_set_generic - Set receive address register
     *  @hw: pointer to the HW structure
     *  @addr: pointer to the receive address
     *  @index: receive address array register
     *
     *  Sets the receive address array register at index to the address passed
     *  in by addr.
     **/
    pub fn rar_set_generic(&self, addr: MacAddress, index: usize, reg: Reg) -> Result {
        let index = index as u32;
        let rar_low = u32::from_le_bytes([addr.0[0], addr.0[1], addr.0[2], addr.0[3]]);
        let mut rar_high = u16::from_le_bytes([addr.0[4], addr.0[5]]) as u32;
        /* HW expects these in little endian so we reverse the byte order
         * from network order (big endian) to little endian
         */

        /* If MAC address zero, no need to set the AV bit */
        if rar_low > 0 || rar_high > 0 {
            rar_high |= RAH_AV;
        }

        /* Some bridges will combine consecutive 32-bit writes into
         * a single burst write, which will malfunction on some parts.
         * The flushes avoid this.
         */
        reg.ew32(RAL(index), rar_low);
        reg.e1e_flush();
        reg.ew32(RAH(index), rar_high);
        reg.e1e_flush();

        Ok(())
    }


    /**
     *  e1000e_config_collision_dist_generic - Configure collision distance
     *  @hw: pointer to the HW structure
     *
     *  Configures the collision distance to the default value and is used
     *  during link setup.
     **/
    pub fn config_collision_dist_generic(&mut self) {
        let mut tctl = self.er32(TCTL);

        tctl &= !TCTL_COLD;
        tctl |= COLLISION_DISTANCE << COLD_SHIFT;

        self.ew32(TCTL, tctl);
        self.e1e_flush();
    }

    pub fn er32(&self, offset: u32) -> u32 {
        self.reg.er32(offset)
    }

    pub fn ew32(&self, offset: u32, value: u32) {
        self.reg.ew32(offset, value)
    }

    pub fn e1e_flush(&self) {
        self.reg.e1e_flush()
    }
    pub fn force_mac_fc(&self, fc_mode: FcMode) -> Result {
        let mut ctrl = self.er32(CTRL);

        /* Because we didn't get link via the internal auto-negotiation
         * mechanism (we either forced link or we got link via PHY
         * auto-neg), we have to manually enable/disable transmit an
         * receive flow control.
         *
         * The "Case" statement below enables/disable flow control
         * according to the "hw->fc.current_mode" parameter.
         *
         * The possible values of the "fc" parameter are:
         *      0:  Flow control is completely disabled
         *      1:  Rx flow control is enabled (we can receive pause
         *          frames but not send pause frames).
         *      2:  Tx flow control is enabled (we can send pause frames
         *          but we do not receive pause frames).
         *      3:  Both Rx and Tx flow control (symmetric) is enabled.
         *  other:  No other values should be possible at this point.
         */
        info!("hw->fc.current_mode = {:?}", fc_mode);

        match fc_mode {
            FcMode::None => ctrl &= (!(CTRL_TFCE | CTRL_RFCE)),
            FcMode::RxPause => {
                ctrl &= !CTRL_TFCE;
                ctrl |= CTRL_RFCE;
            }
            FcMode::TxPause => {
                ctrl &= !CTRL_RFCE;
                ctrl |= CTRL_TFCE;
            }
            FcMode::Full => ctrl |= (CTRL_TFCE | CTRL_RFCE),
            FcMode::Default => {
                error!("Flow control param set incorrectly");
                return Err(Error::Config("Flow control param set incorrectly"));
            }
        }

        self.ew32(CTRL, ctrl);

        return Ok(());
    }
}
#[derive(Debug, Default, Clone)]
pub(crate) struct MacOps {
    pub setup_physical_interface: SetupPhysicalInterface,
}
#[derive(Debug, Clone, Copy)]
pub(crate) enum SetupPhysicalInterface {
    CopperLink82571,
    CopperLinkIch8lan,
    CopperLinkPchLpt,
}

impl Default for SetupPhysicalInterface {
    fn default() -> Self {
        Self::CopperLink82571
    }
}
