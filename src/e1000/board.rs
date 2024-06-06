use core::ptr::NonNull;

use super::dev_id::*;
mod b82571;
mod ich8lan;
use crate::{base::Base, err::*, KernelFunc, E1000};

const DEFAULT_JUMBO: u32 = 9234;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum BoardType {
    board_82571,
    board_82572,
    board_82573,
    board_82574,
    board_82583,
    board_80003es2lan,
    board_ich8lan,
    board_ich9lan,
    board_ich10lan,
    board_pchlan,
    board_pch2lan,
    board_pch_lpt,
    board_pch_spt,
    board_pch_cnp,
    board_pch_tgp,
    board_pch_adp,
    board_pch_mtp,
}
impl From<usize> for BoardType {
    fn from(value: usize) -> Self {
        unsafe { *(&value as *const usize as *const BoardType) }
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Flags {
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
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct Flags2 {
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
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum E1000MacType {
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

impl Default for E1000MacType {
    fn default() -> Self {
        Self::e1000_82571
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum E1000GetVariants {
    B82571,
    Ich8lan,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct BoardInfo {
    pub mac: E1000MacType,
    pub flags: Flags,
    pub flags2: Flags2,
    pub pba: u32,
    pub max_hw_frame_size: u32,
    pub get_variants: E1000GetVariants,
}

impl BoardType {
    pub fn new_default_info(&self) -> BoardInfo {
        match self {
            BoardType::board_82571 => todo!(),
            BoardType::board_82572 => todo!(),
            BoardType::board_82573 => todo!(),
            BoardType::board_82574 => BoardInfo {
                mac: E1000MacType::e1000_82574,
                flags: Flags {
                    has_hw_vlan_filter: true,
                    has_msix: true,
                    has_jumbo_frames: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    apme_in_ctrl3: true,
                    has_smart_power_down: true,
                    has_amt: true,
                    has_ctrlext_on_load: true,
                    ..Default::default()
                },
                flags2: Flags2 {
                    check_phy_hang: true,
                    disable_aspm_l0s: true,
                    disable_aspm_l1: true,
                    no_disable_rx: true,
                    dma_burst: true,
                    check_systim_overflow: true,
                    ..Default::default()
                },
                pba: 32,
                max_hw_frame_size: DEFAULT_JUMBO,
                get_variants: E1000GetVariants::B82571,
            },
            BoardType::board_82583 => todo!(),
            BoardType::board_80003es2lan => todo!(),
            BoardType::board_ich8lan => todo!(),
            BoardType::board_ich9lan => todo!(),
            BoardType::board_ich10lan => todo!(),
            BoardType::board_pchlan => todo!(),
            BoardType::board_pch2lan => todo!(),
            BoardType::board_pch_lpt => todo!(),
            BoardType::board_pch_spt => todo!(),
            BoardType::board_pch_cnp => todo!(),
            BoardType::board_pch_tgp => BoardInfo {
                mac: E1000MacType::e1000_pch_adp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,

                    ..Default::default()
                },
                flags2: Flags2 {
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                get_variants: E1000GetVariants::Ich8lan,
            },
            BoardType::board_pch_adp => BoardInfo {
                mac: E1000MacType::e1000_pch_adp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,

                    ..Default::default()
                },
                flags2: Flags2 {
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                get_variants: E1000GetVariants::Ich8lan,
            },
            BoardType::board_pch_mtp => BoardInfo {
                mac: E1000MacType::e1000_pch_mtp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,

                    ..Default::default()
                },
                flags2: Flags2 {
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                get_variants: E1000GetVariants::Ich8lan,
            },
        }
    }

    pub fn from_dev_id(id: u32) -> Option<Self> {
        use BoardType::*;
        let one = match id {
            E1000_DEV_ID_82574L => board_82574,
            E1000_DEV_ID_82574LA => board_82574,
            E1000_DEV_ID_PCH_TGP_I219_LM13 => board_pch_tgp,
            E1000_DEV_ID_PCH_TGP_I219_V13 => board_pch_tgp,
            E1000_DEV_ID_PCH_RPL_I219_LM22 => board_pch_adp,
            E1000_DEV_ID_PCH_RPL_I219_V22 => board_pch_adp,
            _ => return None,
        };

        Some(one)
    }
}

impl<K> E1000<K>
where
    K: KernelFunc,
{
    pub(crate) fn get_variants(&mut self) {
        match self.hw.board_info.get_variants {
            E1000GetVariants::B82571 => self.get_variants_82571(),
            E1000GetVariants::Ich8lan => self.get_variants_ich8lan(),
        }
    }
}

#[derive(Debug)]
pub enum BoardVariant {
    B82571,
    Ich8lan,
}
#[derive(Debug, Clone, Default)]
pub struct MacInfo {
    pub kind: E1000MacType,
}

pub struct Board {
    pub kind: BoardType,
    pub variant: BoardVariant,
    pub mac: MacInfo,
    pub flags: Flags,
    pub flags2: Flags2,
    pub pba: u32,
    pub max_hw_frame_size: u32,
}

impl Board {
    pub fn new(device_id: u32) -> Result<Self> {
        let kind = BoardType::from_dev_id(device_id).ok_or_else(|| {
            error!("device_id: {:#X} is not support", device_id);
            Error::Notsupport
        })?;

        let mut s = Self {
            kind,
            variant: BoardVariant::B82571,
            mac: Default::default(),
            flags: Default::default(),
            flags2: Default::default(),
            pba: 0,
            max_hw_frame_size: 0,
        };
        s.init();
        Ok(s)
    }

    fn init(&mut self) {
        match self.kind {
            BoardType::board_82571 => todo!(),
            BoardType::board_82572 => todo!(),
            BoardType::board_82573 => todo!(),
            BoardType::board_82574 => self.init_8754(),
            BoardType::board_82583 => todo!(),
            BoardType::board_80003es2lan => todo!(),
            BoardType::board_ich8lan => todo!(),
            BoardType::board_ich9lan => todo!(),
            BoardType::board_ich10lan => todo!(),
            BoardType::board_pchlan => self.init_pch(),
            BoardType::board_pch2lan => self.init_pch(),
            BoardType::board_pch_lpt => self.init_pch(),
            BoardType::board_pch_spt => self.init_pch(),
            BoardType::board_pch_cnp => self.init_pch(),
            BoardType::board_pch_tgp => self.init_pch(),
            BoardType::board_pch_adp => self.init_pch(),
            BoardType::board_pch_mtp => self.init_pch(),
        }
    }

    fn init_pch(&mut self) {
        self.mac.kind = E1000MacType::e1000_pch_mtp;
        self.flags = Flags {
            is_ich: true,
            has_wol: true,
            has_hw_timestamp: true,
            has_ctrlext_on_load: true,
            has_amt: true,
            has_flash: true,
            has_jumbo_frames: true,
            apme_in_wuc: true,

            ..Default::default()
        };
        self.flags2 = Flags2 {
            has_phy_stats: true,
            has_eee: true,
            ..Default::default()
        };
        self.pba = 26;
        self.max_hw_frame_size = 9022;
        self.variant = BoardVariant::Ich8lan;
    }

    fn init_8754(&mut self) {
        self.mac.kind = E1000MacType::e1000_82574;

        self.flags = Flags {
            has_hw_vlan_filter: true,
            has_msix: true,
            has_jumbo_frames: true,
            has_wol: true,
            has_hw_timestamp: true,
            apme_in_ctrl3: true,
            has_smart_power_down: true,
            has_amt: true,
            has_ctrlext_on_load: true,
            ..Default::default()
        };

        self.flags2 = Flags2 {
            check_phy_hang: true,
            disable_aspm_l0s: true,
            disable_aspm_l1: true,
            no_disable_rx: true,
            dma_burst: true,
            check_systim_overflow: true,
            ..Default::default()
        };
        self.pba = 32;
        self.max_hw_frame_size = DEFAULT_JUMBO;
        self.variant = BoardVariant::B82571;
    }
}
