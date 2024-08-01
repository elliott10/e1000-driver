use core::ptr::NonNull;

use super::dev_id::*;
use super::hw::*;
use super::mac::*;
use super::nvm::*;
use super::err::*;
use super::params::Flags;
use super::phy::*;

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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum E1000GetVariants {
    B82571,
    Ich8lan,
}

#[derive(Debug, Clone)]
pub(crate) struct BoardInfo {
    pub mac: MacType,
    pub flags: Flags,
    pub pba: u32,
    pub max_hw_frame_size: u32,
    pub variant: BoardVariant,
    pub mac_ops: MacOps,
    pub phy_ops: PhyOps,
    pub nvm_ops: NvmOps,
}

const fn e82_phy_ops_bm() -> PhyOps {
    PhyOps {
        semaphore: PhySemaphore::B82571,
        read_write: PhyReadWrite::Bm2,
        get_cfg_done: PhyGetCfgDone::Generic,
    }
}
const fn ich8_phy_ops() -> PhyOps {
    PhyOps {
        semaphore: PhySemaphore::Ich8lan,
        read_write: PhyReadWrite::Hv,
        get_cfg_done: PhyGetCfgDone::Ich8lan,
    }
}

const fn spt_nvm_ops() -> NvmOps {
    NvmOps {
        semaphore: NvmSemaphore::Spt,
        read_write: NvmReadWrite::Ich8lan,
    }
}

const fn e82571_nvm_ops() -> NvmOps {
    NvmOps {
        semaphore: NvmSemaphore::E82571,
        read_write: NvmReadWrite::E82571,
    }
}

const fn e82571_mac_ops() -> MacOps {
    MacOps {
        setup_physical_interface: SetupPhysicalInterface::CopperLink82571,
    }
}

const fn ich8_mac_ops() -> MacOps {
    MacOps {
        setup_physical_interface: SetupPhysicalInterface::CopperLinkIch8lan,
    }
}

impl BoardType {
    pub(crate) fn new_default_info(&self) -> BoardInfo {
        match self {
            BoardType::board_82571 => todo!(),
            BoardType::board_82572 => todo!(),
            BoardType::board_82573 => todo!(),
            BoardType::board_82574 => BoardInfo {
                mac: MacType::e1000_82574,
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
                variant: BoardVariant::B82571,
                mac_ops: e82571_mac_ops(),
                phy_ops: e82_phy_ops_bm(),
                nvm_ops: e82571_nvm_ops(),
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
                mac: MacType::e1000_pch_adp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                variant: BoardVariant::Ich8lan,
                mac_ops: ich8_mac_ops(),
                phy_ops: ich8_phy_ops(),
                nvm_ops: spt_nvm_ops(),
            },
            BoardType::board_pch_adp => BoardInfo {
                mac: MacType::e1000_pch_adp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                variant: BoardVariant::Ich8lan,
                mac_ops: ich8_mac_ops(),
                phy_ops: ich8_phy_ops(),
                nvm_ops: spt_nvm_ops(),
            },
            BoardType::board_pch_mtp => BoardInfo {
                mac: MacType::e1000_pch_mtp,
                flags: Flags {
                    is_ich: true,
                    has_wol: true,
                    has_hw_timestamp: true,
                    has_ctrlext_on_load: true,
                    has_amt: true,
                    has_flash: true,
                    has_jumbo_frames: true,
                    apme_in_wuc: true,
                    has_phy_stats: true,
                    has_eee: true,
                    ..Default::default()
                },
                pba: 26,
                max_hw_frame_size: 9022,
                variant: BoardVariant::Ich8lan,
                mac_ops: ich8_mac_ops(),
                phy_ops: ich8_phy_ops(),
                nvm_ops: spt_nvm_ops(),
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

#[derive(Debug, Clone, Copy)]
pub enum BoardVariant {
    B82571,
    Ich8lan,
}
