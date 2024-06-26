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
    board_info: BoardInfo,
    reg: Reg,
    pub(crate) kind: VariantKind,
}

impl Variant {
    pub(crate) fn new(board_info: BoardInfo, reg: Reg) -> Self {
        let kind = match board_info.variant {
            BoardVariant::B82571 => VariantKind::B82571(Default::default()),
            BoardVariant::Ich8lan => VariantKind::Ich8lan(Default::default()),
        };
        Self {
            board_info,
            kind,
            reg,
        }
    }

    pub(crate) fn get_variants(&self) -> (MacInfo, Flags, Flags2) {
        match &self.kind {
            VariantKind::B82571(ctx) => ctx.get_variants(self.board_info.clone()),
            VariantKind::Ich8lan(ctx) => ctx.get_variants(self.board_info.clone()),
        }
    }
}

pub(crate) trait VariantCommon {
    fn get_variants(&self, board_info: BoardInfo) -> (MacInfo, Flags, Flags2);
}
