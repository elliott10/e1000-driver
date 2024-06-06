use super::regs;
use crate::*;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

pub(crate) const E1000_ICH_FWSM_RSPCIPHY: u32 = 0x00000040; // Reset PHY on PCI Reset
pub(crate) const E1000_ICH_FWSM_FW_VALID: u32 = 0x00008000; // FW established a valid mode
pub(crate) const E1000_ICH_FWSM_PCIM2PCI: u32 = 0x01000000; // ME PCIm-to-PCI active
pub(crate) const E1000_ICH_FWSM_PCIM2PCI_COUNT: u32 = 2000;

pub(crate) const E1000_ICH_MNG_IAMT_MODE: u32 = 0x2;

#[derive(Debug)]
pub(crate) struct Hw {
    pub(crate) mmio: usize,
    pub(crate) board: BoardType,
    pub(crate) flash: Option<usize>,
    pub(crate) stats: E1000HWStats,
    pub(crate) board_info: BoardInfo,
}

impl Hw {
    pub(crate) fn new(mmio: usize, board: BoardType) -> Self {
        Hw {
            mmio: mmio,
            board,
            flash: None,
            stats: Default::default(),
            board_info: board.new_default_info(),
        }
    }

    pub(crate) fn set_flash(&mut self, flash: usize) {
        self.flash = Some(flash);
    }

    pub(crate) fn ral(&self, index: usize) -> usize {
        regs::RA + index * 8
    }

    pub(crate) fn rah(&self, index: usize) -> usize {
        regs::RA + index * 8 + 4
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub enum MediaType {
    media_type_unknown = 0,
    media_type_copper = 1,
    media_type_fiber = 2,
    media_type_internal_serdes = 3,
    num_media_types,
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum MacType {
    e1000_82571,
    e1000_82572,
    e1000_82573,
    e1000_82574,
    e1000_82583,
    e1000_80003es2lan,
    e1000_ich8lan,
    e1000_ich9lan,
    e1000_ich10lan,
    e1000_pchlan,
    e1000_pch2lan,
    e1000_pch_lpt,
    e1000_pch_spt,
    e1000_pch_cnp,
    e1000_pch_tgp,
    e1000_pch_adp,
    e1000_pch_mtp,
    e1000_pch_lnp,
    e1000_pch_ptp,
    e1000_pch_nvp,
}
const MAX_MTA_REG: usize = 128;

#[derive(Debug, Clone, Default)]
pub struct MacInfo {
    pub addr: [u8; 6],
    pub perm_addr: [u8; 6],

    pub kind: MacType,

    pub collision_delta: u32,
    pub ledctl_default: u32,
    pub ledctl_mode1: u32,
    pub ledctl_mode2: u32,
    pub mc_filter_type: u32,
    pub tx_packet_delta: u32,
    pub txcw: u32,

    pub current_ifs_val: u16,
    pub ifs_max_val: u16,
    pub ifs_min_val: u16,
    pub ifs_ratio: u16,
    pub ifs_step_size: u16,
    pub mta_reg_count: u16,

    // Maximum size of the MTA register table in all supported adapters
    // pub mta_shadow: [u32; MAX_MTA_REG],
    pub rar_entry_count: u16,

    pub forced_speed_duplex: u8,

    pub adaptive_ifs: bool,
    pub has_fwsm: bool,
    pub arc_subsystem_valid: bool,
    pub autoneg: bool,
    pub autoneg_failed: bool,
    pub get_link_status: bool,
    pub in_ifs_mode: bool,
    pub serdes_has_link: bool,
    pub tx_pkt_filtering: bool,
}




#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct E1000HWStats {
    pub crcerrs: u64,
    pub algnerrc: u64,
    pub symerrs: u64,
    pub rxerrc: u64,
    pub mpc: u64,
    pub scc: u64,
    pub ecol: u64,
    pub mcc: u64,
    pub latecol: u64,
    pub colc: u64,
    pub dc: u64,
    pub tncrs: u64,
    pub sec: u64,
    pub cexterr: u64,
    pub rlec: u64,
    pub xonrxc: u64,
    pub xontxc: u64,
    pub xoffrxc: u64,
    pub xofftxc: u64,
    pub fcruc: u64,
    pub prc64: u64,
    pub prc127: u64,
    pub prc255: u64,
    pub prc511: u64,
    pub prc1023: u64,
    pub prc1522: u64,
    pub gprc: u64,
    pub bprc: u64,
    pub mprc: u64,
    pub gptc: u64,
    pub gorc: u64,
    pub gotc: u64,
    pub rnbc: u64,
    pub ruc: u64,
    pub rfc: u64,
    pub roc: u64,
    pub rjc: u64,
    pub mgprc: u64,
    pub mgpdc: u64,
    pub mgptc: u64,
    pub tor: u64,
    pub tot: u64,
    pub tpr: u64,
    pub tpt: u64,
    pub ptc64: u64,
    pub ptc127: u64,
    pub ptc255: u64,
    pub ptc511: u64,
    pub ptc1023: u64,
    pub ptc1522: u64,
    pub mptc: u64,
    pub bptc: u64,
    pub tsctc: u64,
    pub tsctfc: u64,
    pub iac: u64,
    pub icrxptc: u64,
    pub icrxatc: u64,
    pub ictxptc: u64,
    pub ictxatc: u64,
    pub ictxqec: u64,
    pub ictxqmtc: u64,
    pub icrxdmtc: u64,
    pub icrxoc: u64,
}
#[derive(Debug, PartialEq)]
pub enum PhyType {
    Unknown,
    None,
    M88,
    Igp,
    Igp2,
    Gg82563,
    Igp3,
    Ife,
    Bm,
    I2578,
    I2577,
    I2579,
    I217,
}
impl Default for MediaType {
    fn default() -> Self {
        Self::media_type_unknown
    }
}

#[derive(Debug, Default)]
pub struct PhyInfo {
    pub type_: PhyType,

    pub addr: u32,
    pub id: u32,
    pub reset_delay_us: u32, // in microseconds
    pub revision: u32,

    pub media_type: MediaType,

    pub autoneg_advertised: u16,
    pub autoneg_mask: u16,
    pub cable_length: u16,
    pub max_cable_length: u16,
    pub min_cable_length: u16,

    pub mdix: u8,

    pub disable_polarity_correction: bool,
    pub is_mdix: bool,
    pub polarity_correction: bool,
    pub speed_downgraded: bool,
    pub autoneg_wait_to_complete: bool,
}