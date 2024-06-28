use super::{VariantCommon, VariantParam, VariantResult};

use crate::defines::*;
use crate::e1000::mac::*;
use crate::e1000::nvm::*;
use crate::e1000::reg::{Reg, FWSM};
use crate::phy::PhyInfo2;
use crate::{err::*, BoardInfo, Flags, Flags2, MediaType};

#[derive(Debug, Default, Clone)]
pub(crate) struct Context {}

impl VariantCommon for Context {
    fn set_reg(&self, mac_kind: MacType, reg: &mut Reg) {
        if mac_kind == MacType::e1000_pch2lan {
            if (reg.er32(FWSM) & ICH_FWSM_FW_VALID) > 0 {
                reg.pcim2pci_arbiter_wa = true;
            }
        }
    }

    fn init_mac_params(&mut self, mac_kind: MacType, reg: Reg) -> Result<MacInfo> {
        todo!()
    }

    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo2> {
        let media_type = MediaType::media_type_copper;
        todo!()
    }

    fn init_nvm_params(&self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo> {
        todo!()
    }
}
