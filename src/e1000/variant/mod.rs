use super::mac::MacInfo;
use super::reg::Reg;
use crate::e1000::nvm::*;
use crate::params::Flags3;
use crate::phy::PhyInfo2;
use crate::BoardInfo;
use crate::BoardVariant;
use crate::Flags;
use crate::Flags2;
use crate::{err::*, MacType};

pub(crate) mod e82571;
pub(crate) mod ich8lan;

#[derive(Debug, Clone)]
pub(crate) enum VariantKind {
    B82571(e82571::Context),
    Ich8lan(ich8lan::Context),
}

#[derive(Debug, Clone)]
pub(crate) struct Variant {
    pub kind: VariantKind,
}

pub(crate) struct VariantParam {
    pub reg: Reg,
    pub board_info: BoardInfo,
}

pub(crate) struct VariantResult {
    pub mac: MacInfo,
    pub phy: PhyInfo2,
    pub reg: Reg,
    pub flags: Flags3,
}

impl Variant {
    pub(crate) fn new(board_info: &BoardInfo) -> Self {
        let kind = match board_info.variant {
            BoardVariant::B82571 => VariantKind::B82571(Default::default()),
            BoardVariant::Ich8lan => VariantKind::Ich8lan(Default::default()),
        };
        Self { kind }
    }

    pub(crate) fn get_variants(
        &mut self,
        mut reg: Reg,
        board_info: &BoardInfo,
    ) -> Result<VariantResult> {
        let param = VariantParam {
            reg,
            board_info: board_info.clone(),
        };

        match &mut self.kind {
            VariantKind::B82571(ctx) => ctx.get_variants(param, reg, board_info),
            VariantKind::Ich8lan(ctx) => ctx.get_variants(param, reg, board_info),
        }
    }
}

pub(crate) trait VariantCommon {
    fn init_mac_params(&mut self, mac_kind: MacType, reg: Reg) -> Result<MacInfo>;
    fn init_nvm_params(&self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo>;
    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo2>;

    fn get_variants(
        &mut self,
        param: VariantParam,
        mut reg: Reg,
        boad: &BoardInfo,
    ) -> Result<VariantResult> {
        let mut flags = boad.flags.clone();
        let mac = self.init_mac_params(param.board_info.mac, reg)?;
        self.set_reg(mac.kind, &mut reg);
        let nvm = self.init_nvm_params(&mac, boad, reg)?;
        let phy = self.init_phy_params(&mac, boad, reg)?;

        Ok(VariantResult {
            mac,
            phy,
            reg,
            flags,
        })
    }
    fn set_reg(&self, mac_kind: MacType, reg: &mut Reg);
}
