use super::{VariantCommon, VariantParam, VariantResult};
use crate::e1000::mac::MacInfo;
use crate::e1000::reg::*;
use crate::phy::{self, PhyInfo2, PhySemaphore};
use crate::{defines::*, err::*, MacType};
use crate::{Flags, Flags2, MediaType};

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

    fn init_phy_params(&mut self, mac: &MacInfo, reg: Reg) -> Result<PhyInfo2> {
        let media_type = MediaType::media_type_copper;
        let mut id = 0;
        let autoneg_mask = AUTONEG_ADVERTISE_SPEED_DEFAULT;

        Ok(PhyInfo2 {
            reg,
            media_type,
            kind: todo!(),
            semaphore: PhySemaphore::B82571(Semaphore { reg }),
            addr: 1,
            id,
            reset_delay_us: 100,
            revision: todo!(),
            autoneg_advertised: todo!(),
            autoneg_mask: todo!(),
            cable_length: todo!(),
            max_cable_length: todo!(),
            min_cable_length: todo!(),
            mdix: todo!(),
            disable_polarity_correction: todo!(),
            is_mdix: todo!(),
            polarity_correction: todo!(),
            speed_downgraded: todo!(),
            autoneg_wait_to_complete: todo!(),
            ms_type: todo!(),
            original_ms_type: todo!(),
            access_shared_resource: Default::default(),
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Semaphore {
    reg: Reg,
}

impl Semaphore {
    pub(crate) fn new(reg: Reg) -> Self {
        Self { reg }
    }
}

impl phy::Semaphore for Semaphore {
    fn acquire(&self) -> crate::err::Result {
        Ok(())
    }

    fn release(&self) {}
}
