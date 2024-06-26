pub(crate) const DEFAULT_TIDV: u32 = 8;
pub(crate) const DEFAULT_TADV: u32 = 32;
pub(crate) const DEFAULT_RDTR: u32 = 0;
pub(crate) const BURST_RDTR: u32 = 0x20;
pub(crate) const DEFAULT_RADV: u32 = 8;
pub(crate) const BURST_RADV: u32 = 0x20;
pub(crate) const DEFAULT_ITR: u32 = 3;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Flags3 {
    // ---- flags ----
    pub has_amt: bool,
    pub has_flash: bool,
    pub has_hw_vlan_filter: bool,
    pub has_wol: bool,
    pub has_ctrlext_on_load: bool,
    pub has_swsm_on_load: bool,
    pub has_jumbo_frames: bool,
    pub read_only_nvm: bool,
    pub is_ich: bool,
    pub has_msix: bool,
    pub has_smart_power_down: bool,
    pub is_quad_port_a: bool,
    pub is_quad_port: bool,
    pub has_hw_timestamp: bool,
    pub apme_in_wuc: bool,
    pub apme_in_ctrl3: bool,
    pub apme_check_port_b: bool,
    pub disable_fc_pause_time: bool,
    pub no_wake_ucast: bool,
    pub mng_pt_enabled: bool,
    pub reset_overwrites_laa: bool,
    pub tarc_speed_mode_bit: bool,
    pub tarc_set_bit_zero: bool,
    pub rx_needs_restart: bool,
    pub lsc_gig_speed_drop: bool,
    pub smart_power_down: bool,
    pub msi_enabled: bool,
    pub tso_force: bool,
    pub restart_now: bool,
    pub msi_test_failed: bool,
    // ---- flags ----

    // ---- flags2 ----
    pub crc_stripping: bool,
    pub has_phy_wakeup: bool,
    pub is_discarding: bool,
    pub disable_aspm_l1: bool,
    pub has_phy_stats: bool,
    pub has_eee: bool,
    pub dma_burst: bool,
    pub disable_aspm_l0s: bool,
    pub disable_aim: bool,
    pub check_phy_hang: bool,
    pub no_disable_rx: bool,
    pub pcim2pci_arbiter_wa: bool,
    pub dflt_crc_stripping: bool,
    pub check_rx_hwtstamp: bool,
    pub check_systim_overflow: bool,
    pub enable_s0ix_flows: bool,
    // ---- flags2 ----
}

pub(crate) struct Params {
    pub flags: Flags3,
}
