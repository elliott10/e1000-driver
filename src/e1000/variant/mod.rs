use super::mac::MacInfo;
use super::reg::Reg;
use crate::BoardInfo;
use crate::BoardVariant;
use crate::Flags;
use crate::Flags2;

mod b82571;
mod ich8lan;

pub(crate) enum VariantKind {
    B82571(b82571::Context),
    Ich8lan(ich8lan::Context),
}

pub(crate) struct Variant {
    pub board_info: BoardInfo,
    pub kind: VariantKind,
}

pub(crate) struct VariantParam {
    pub reg: Reg,
    pub board_info: BoardInfo,
}

pub(crate) struct VariantResult {
    pub board_info: BoardInfo,
    pub mac_info: MacInfo,
    pub reg: Reg,
}

impl Variant {
    pub(crate) fn new(board_info: BoardInfo) -> Self {
        let kind = match board_info.variant {
            BoardVariant::B82571 => VariantKind::B82571(Default::default()),
            BoardVariant::Ich8lan => VariantKind::Ich8lan(Default::default()),
        };
        Self { board_info, kind }
    }

    pub(crate) fn get_variants(&self, reg: Reg) -> VariantResult {
        let param = VariantParam {
            reg,
            board_info: self.board_info.clone(),
        };
        match &self.kind {
            VariantKind::B82571(ctx) => ctx.get_variants(param),
            VariantKind::Ich8lan(ctx) => ctx.get_variants(param),
        }
    }
}

pub(crate) trait VariantCommon {
    fn get_variants(&self, param: VariantParam) -> VariantResult;
}
