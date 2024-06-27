use crate::*;

pub(crate) const DEFAULT_TIDV: u32 = 8;
pub(crate) const DEFAULT_TADV: u32 = 32;
pub(crate) const DEFAULT_RDTR: u32 = 0;
pub(crate) const BURST_RDTR: u32 = 0x20;
pub(crate) const DEFAULT_RADV: u32 = 8;
pub(crate) const BURST_RADV: u32 = 0x20;
pub(crate) const DEFAULT_ITR: u32 = 3;

/* This is the only thing that needs to be changed to adjust the
 * maximum number of ports that the driver can manage.
 */
const MAX_NIC: usize = 32;

#[repr(C)]
#[derive(Debug, Clone, Default)]
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

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Settings {
    pub enable_msi: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Params {
    pub settings: Settings,
    pub flags: Flags3,
    pub int_mode: InterruptMode,

    /* Interrupt Throttle Rate (interrupts/sec)
     *
     * Valid Range: 100-100000 or one of: 0=off, 1=dynamic, 3=dynamic conservative
     */
    pub itr: u32,
    pub itr_settings: u32,

    pub tx_int_delay: u32,
    pub tx_abs_int_delay: u32,

    pub rx_int_delay: u32,
    pub rx_abs_int_delay: u32,

    pub card_found: usize,

    pub link_speed: u16,
    pub link_duplex: u16,
    pub max_hw_frame_size: u32,
    pub eeprom_wol: u32,
    pub wol: u32,
    pub pba: u32,
}

impl Params {
    pub fn check_options(&mut self) {
        if (self.card_found >= MAX_NIC) {
            warn!("Warning: no configuration for board #{}", self.card_found);
            warn!("Using defaults for all values");
        }

        self.tx_int_delay = DEFAULT_TIDV;
        self.tx_abs_int_delay = DEFAULT_TADV;

        self.rx_int_delay = DEFAULT_RDTR;
        self.rx_abs_int_delay = DEFAULT_RADV;

        if self.flags.dma_burst {
            self.rx_int_delay = BURST_RDTR;
            self.rx_abs_int_delay = BURST_RADV;
        }
        self.check_interrupt_throttling_rate();

        self.check_interrupt_mode();

        self.check_crc_stripping();

        self.write_protect_nvm();
    }

    fn check_interrupt_mode(&mut self) {
        if self.settings.enable_msi {
            if self.flags.has_msix {
                self.int_mode = InterruptMode::MSIX;
            } else {
                self.int_mode = InterruptMode::MSI;
            }
        } else {
            self.int_mode = InterruptMode::Legacy;
        }
    }

    fn check_interrupt_throttling_rate(&mut self) {
        {
            let opt_name = "Interrupt Throttling Rate (ints/sec)";
            self.itr = DEFAULT_ITR;
            self.itr_settings = self.itr;

            match self.itr {
                0 => {
                    debug!("{} turned off", opt_name);
                }
                1 => {
                    debug!("{} set to dynamic mode", opt_name);
                    self.itr = 20000;
                }
                2 => {
                    debug!("{} Invalid mode - setting default", opt_name);
                    self.itr_settings = DEFAULT_ITR;
                }
                3 => {
                    debug!("{} set to dynamic conservative mode", opt_name);
                    self.itr = 20000;
                }
                4 => {
                    debug!("{} set to simplified (2000-8000 ints) mode", opt_name);
                }
                _ => {
                    // Save the setting, because the dynamic bits change itr.
                    // Clear the lower two bits because they are used as control.
                    self.itr_settings &= !3;
                }
            }
        }
    }

    fn check_crc_stripping(&mut self) {
        self.flags.crc_stripping = true;
        self.flags.dflt_crc_stripping = true;
    }

    fn write_protect_nvm(&mut self) {
        if self.flags.is_ich {
            self.flags.read_only_nvm = true;
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub(crate) enum InterruptMode {
    Legacy,
    MSI,
    MSIX,
}

impl Default for InterruptMode {
    fn default() -> Self {
        Self::Legacy
    }
}
