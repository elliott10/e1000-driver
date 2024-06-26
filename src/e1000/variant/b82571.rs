use super::VariantCommon;
use crate::e1000::mac::MacInfo;
use crate::{Flags,Flags2};


#[derive(Default)]
pub(crate) struct Context {}

impl VariantCommon for Context {
    fn get_variants(&self, board_info: crate::BoardInfo) -> (MacInfo, Flags, Flags2) {
        let mut mac_info = MacInfo::default();        
        let mut flags = board_info.flags;
        let mut flags2 = board_info.flags2;



        (mac_info, flags, flags2)
    }
}
