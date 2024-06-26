use super::{VariantCommon, VariantParam, VariantResult};
use crate::e1000::mac::MacInfo;
use crate::{Flags, Flags2};

#[derive(Default)]
pub(crate) struct Context {}

impl VariantCommon for Context {
    fn get_variants(&self, param: VariantParam) -> VariantResult {
        let mut mac_info = MacInfo::default();
        let mut board_info = param.board_info.clone();
        let reg = param.reg;

        VariantResult { reg, board_info }
    }
}
