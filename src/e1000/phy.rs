pub(crate) const MAX_PHY_REG_ADDRESS: u32 = 0x1F; // 5 bit address bus (0-0x1F)
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
pub(crate) const BM_PORT_CTRL_PAGE: u16 = 769;
pub(crate) const BM_WUC_PAGE: u16 = 800;
pub(crate) const BM_WUC_ADDRESS_OPCODE: u8 = 0x11;
pub(crate) const BM_WUC_DATA_OPCODE: u8 = 0x12;
pub(crate) const BM_WUC_ENABLE_PAGE: u16 = BM_PORT_CTRL_PAGE;
pub(crate) const BM_WUC_ENABLE_REG: u16 = 17;

// MDI Control
pub(crate) const MDIC_REG_MASK: u32 = 0x001F0000;
pub(crate) const MDIC_REG_SHIFT: u32 = 16;
pub(crate) const MDIC_PHY_SHIFT: u32 = 21;
pub(crate) const MDIC_OP_WRITE: u32 = 0x04000000;
pub(crate) const MDIC_OP_READ: u32 = 0x08000000;
pub(crate) const MDIC_READY: u32 = 0x10000000;
pub(crate) const MDIC_ERROR: u32 = 0x40000000;
