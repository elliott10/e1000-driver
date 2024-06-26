use super::VariantCommon;
use crate::e1000::mac::MacInfo;
use crate::{Flags,Flags2};

#[derive(Default)]
pub(crate) struct Context{

}

impl VariantCommon for Context {
    fn get_variants(&self, board_info: crate::BoardInfo) -> (MacInfo, Flags, Flags2) {
        todo!()
    }
}