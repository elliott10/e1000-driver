use super::reg::Reg;
use crate::e1000::{err::*, mac::*, nvm::*, params::*, phy::*, BoardInfo, BoardVariant, E1000};

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

pub(crate) struct VariantResult {
    pub mac: MacInfo,
    pub phy: PhyInfo,
    pub nvm: NvmInfo,
    pub reg: Reg,
    pub flags: Flags,
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
        match &mut self.kind {
            VariantKind::B82571(ctx) => ctx.get_variants(reg, board_info),
            VariantKind::Ich8lan(ctx) => ctx.get_variants(reg, board_info),
        }
    }

    pub(crate) fn reset_hw(&self, e1000: &E1000) -> Result {
        match &self.kind {
            VariantKind::B82571(ctx) => ctx.reset_hw(e1000),
            VariantKind::Ich8lan(ctx) => ctx.reset_hw(e1000),
        }
    }
    pub(crate) fn init_hw(&self, e1000: &E1000) -> Result {
        match &self.kind {
            VariantKind::B82571(ctx) => ctx.init_hw(e1000),
            VariantKind::Ich8lan(ctx) => ctx.init_hw(e1000),
        }
    }
}

pub(crate) trait VariantCommon {
    fn init_mac_params(&mut self, board: &BoardInfo, reg: Reg) -> Result<MacInfo>;
    fn init_nvm_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<NvmInfo>;
    fn init_phy_params(&mut self, mac: &MacInfo, board: &BoardInfo, reg: Reg) -> Result<PhyInfo>;
    fn reset_hw(&self, e1000: &E1000) -> Result;
    fn init_hw(&self, e1000: &E1000) -> Result;

    fn get_variants(&mut self, mut reg: Reg, boad: &BoardInfo) -> Result<VariantResult> {
        let mut flags = boad.flags.clone();
        let mut mac: MacInfo = self.init_mac_params(boad, reg)?;
        #[cfg(not(oom_handle))]
        mac.mta_shadow.try_resize(128, 0)?;
        #[cfg(oom_handle)]
        mac.mta_shadow.resize(128, 0);
        self.set_reg(mac.kind, &mut reg);
        mac.reg = reg;
        let mut nvm = self.init_nvm_params(&mac, boad, reg)?;
        reg = nvm.reg;
        nvm.mac_kind = mac.kind;
        mac.reg = reg;
        let phy = self.init_phy_params(&mac, boad, reg)?;

        Ok(VariantResult {
            mac,
            nvm,
            phy,
            reg,
            flags,
        })
    }
    fn set_reg(&self, mac_kind: MacType, reg: &mut Reg);
}
