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
    pub(crate) board_info: BoardInfo,
    pub(crate) reg: Reg,
    pub(crate) kind: VariantKind,
}

pub(crate) struct VariantParam {
    pub(crate) reg: Reg,
    pub(crate) board_info: BoardInfo,
}

pub(crate) struct VariantResult {
    pub(crate) reg: Reg,
    pub(crate) board_info: BoardInfo,
}

impl Variant {
    pub(crate) fn new(board_info: BoardInfo, reg: Reg) -> Self {
        let kind = match board_info.variant {
            BoardVariant::B82571 => VariantKind::B82571(Default::default()),
            BoardVariant::Ich8lan => VariantKind::Ich8lan(Default::default()),
        };
        let mut s = Self {
            board_info,
            kind,
            reg,
        };

        s.get_variants();

        s
    }

    fn get_variants(&mut self) {
        let param = VariantParam {
            reg: self.reg,
            board_info: self.board_info.clone(),
        };

        let res = match &self.kind {
            VariantKind::B82571(ctx) => ctx.get_variants(param),
            VariantKind::Ich8lan(ctx) => ctx.get_variants(param),
        };

        self.reg = res.reg;
        self.board_info = res.board_info;
    }
}

pub(crate) trait VariantCommon {
    fn get_variants(&self, param: VariantParam) -> VariantResult;
}
