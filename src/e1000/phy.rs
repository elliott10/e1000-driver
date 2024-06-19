// IGP01E1000 Specific Registers
pub(crate) const PHY_PORT_CONFIG: u16 = 0x10; // Port Config
pub(crate) const PHY_PORT_STATUS: u16 = 0x11; // Status
pub(crate) const PHY_PORT_CTRL: u16 = 0x12; // Control
pub(crate) const PHY_LINK_HEALTH: u16 = 0x13; // PHY Link Health
pub(crate) const PHY_POWER_MGMT_IGP02: u16 = 0x19; // Power Management
pub(crate) const PHY_PAGE_SELECT_IGP01: u16 = 0x1F; // Page Select
pub(crate) const BM_PHY_PAGE_SELECT: u16 = 22; // Page Select for BM
pub(crate) const IGP_PAGE_SHIFT: u32 = 5;
pub(crate) const PHY_REG_MASK: u16 = 0x1F;

// BM/HV Specific Registers
pub(crate) const BM_PORT_CTRL_PAGE: u32 = 769;
pub(crate) const BM_WUC_PAGE: u16 = 800;
pub(crate) const BM_WUC_ADDRESS_OPCODE: u8 = 0x11;
pub(crate) const BM_WUC_DATA_OPCODE: u8 = 0x12;
pub(crate) const BM_WUC_ENABLE_PAGE: u32 = BM_PORT_CTRL_PAGE;
pub(crate) const BM_WUC_ENABLE_REG: u16 = 17;

// MDI Control
pub(crate) const MDIC_REG_MASK: u32 = 0x001F0000;
pub(crate) const MDIC_REG_SHIFT: u32 = 16;
pub(crate) const MDIC_PHY_SHIFT: u32 = 21;
pub(crate) const MDIC_OP_WRITE: u32 = 0x04000000;
pub(crate) const MDIC_OP_READ: u32 = 0x08000000;
pub(crate) const MDIC_READY: u32 = 0x10000000;
pub(crate) const MDIC_ERROR: u32 = 0x40000000;



pub(crate) const HV_INTC_FC_PAGE_START: u32 = 768;
pub(crate) const I82578_ADDR_REG: u32 = 29;
pub(crate) const I82577_ADDR_REG: u32 = 16;
pub(crate) const I82577_CFG_REG: u32 = 22;
pub(crate) const I82577_CFG_ASSERT_CRS_ON_TX: u16 = 1 << 15;
pub(crate) const I82577_CFG_ENABLE_DOWNSHIFT: u16 = 3 << 10; // auto downshift
pub(crate) const I82577_CTRL_REG: u32 = 23;

// 82577 specific PHY registers
pub(crate) const I82577_PHY_CTRL_2: u32 = 18;
pub(crate) const I82577_PHY_LBK_CTRL: u32 = 19;
pub(crate) const I82577_PHY_STATUS_2: u32 = 26;
pub(crate) const I82577_PHY_DIAG_STATUS: u32 = 31;

// I82577 PHY Status 2
pub(crate) const I82577_PHY_STATUS2_REV_POLARITY: u16 = 0x0400;
pub(crate) const I82577_PHY_STATUS2_MDIX: u16 = 0x0800;
pub(crate) const I82577_PHY_STATUS2_SPEED_MASK: u16 = 0x0300;
pub(crate) const I82577_PHY_STATUS2_SPEED_1000MBPS: u16 = 0x0200;

// I82577 PHY Control 2
pub(crate) const I82577_PHY_CTRL2_MANUAL_MDIX: u16 = 0x0200;
pub(crate) const I82577_PHY_CTRL2_AUTO_MDI_MDIX: u16 = 0x0400;
pub(crate) const I82577_PHY_CTRL2_MDIX_CFG_MASK: u16 = 0x0600;

// I82577 PHY Diagnostics Status
pub(crate) const I82577_DSTATUS_CABLE_LENGTH: u16 = 0x03FC;
pub(crate) const I82577_DSTATUS_CABLE_LENGTH_SHIFT: u8 = 2;

pub(crate) const MAX_PHY_ADDR: u32 = 8;