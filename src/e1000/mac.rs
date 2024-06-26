use alloc::vec::Vec;

use crate::MacType;

#[derive(Default)]
pub(crate) struct MacInfo {
    pub(crate) kind: MacType,
    pub(crate) addr: [u8; 6],
    pub(crate) perm_addr: [u8; 6],

    pub(crate) collision_delta: u32,
    pub(crate) ledctl_default: u32,
    pub(crate) ledctl_mode1: u32,
    pub(crate) ledctl_mode2: u32,
    pub(crate) mc_filter_type: u32,
    pub(crate) tx_packet_delta: u32,
    pub(crate) txcw: u32,

    pub(crate) current_ifs_val: u16,
    pub(crate) ifs_max_val: u16,
    pub(crate) ifs_min_val: u16,
    pub(crate) ifs_ratio: u16,
    pub(crate) ifs_step_size: u16,
    pub(crate) mta_reg_count: u16,

    // Maximum size of the MTA register table in all supported adapters
    pub(crate) mta_shadow: Vec<u32>,
    pub(crate) rar_entry_count: u16,

    pub(crate) forced_speed_duplex: u8,

    pub(crate) adaptive_ifs: bool,
    pub(crate) has_fwsm: bool,
    pub(crate) arc_subsystem_valid: bool,
    pub(crate) autoneg: bool,
    pub(crate) autoneg_failed: bool,
    pub(crate) get_link_status: bool,
    pub(crate) in_ifs_mode: bool,
    pub(crate) serdes_has_link: bool,
    pub(crate) tx_pkt_filtering: bool,
}
