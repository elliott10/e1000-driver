use super::defines::*;
use super::params::Flags;
use super::phy::*;
use super::reg::*;

use super::err::{Error, Result};
use super::*;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use core::str;

#[repr(C)]
#[derive(Clone, Default)]
pub(crate) struct MsixEntry {
    pub vector: u32, /* Kernel uses to write allocated vector */
    pub entry: u32,  /* Driver uses to specify entry, OS writes */
}

#[derive(Default, Clone)]
#[repr(C)]
pub(crate) struct HostMngDhcpCookie {
    pub(crate) signature: u32,
    pub(crate) status: u8,
    pub(crate) reserved0: u8,
    pub(crate) vlan_id: u16,
    pub(crate) reserved1: u32,
    pub(crate) reserved2: u16,
    pub(crate) reserved3: u8,
    pub(crate) checksum: u8,
}
const MAX_MTA_REG: usize = 128;

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

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub(crate) enum BusWidth {
    Unknown = 0,
    PcieX1,
    PcieX2,
    PcieX4 = 4,
    PcieX8 = 8,
    ThirtyTwo,
    SixtyFour,
    Reserved,
}
impl Default for BusWidth {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FcInfo {
    pub high_water: u32,        // Flow control high-water mark
    pub low_water: u32,         // Flow control low-water mark
    pub pause_time: u16,        // Flow control pause timer
    pub refresh_time: u16,      // Flow control refresh timer
    pub send_xon: bool,         // Flow control send XON
    pub strict_ieee: bool,      // Strict IEEE mode
    pub current_mode: FcMode,   // FC mode in effect
    pub requested_mode: FcMode, // FC mode requested by caller
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub(crate) enum FcMode {
    None = 0,
    RxPause,
    TxPause,
    Full,
    Default = 0xFF,
}
impl Default for FcMode {
    fn default() -> Self {
        Self::Default
    }
}

pub(crate) struct BusInfo {
    pub(crate) width: BusWidth,
    pub(crate) fun: u16,
}

impl BusInfo {
    pub fn new() -> Self {
        let cap_offset = kfn().pci_cap_offset();

        let width = if cap_offset == 0 {
            BusWidth::PcieX1
        } else {
            const PCIE_LINK_STATUS: i32 = 0x12;
            const PCIE_LINK_WIDTH_MASK: u16 = 0x3F0;
            const PCIE_LINK_WIDTH_SHIFT: u16 = 4;

            let mut pcie_link_status = kfn().pci_read_config_word(cap_offset + PCIE_LINK_STATUS);
            pcie_link_status &= (PCIE_LINK_WIDTH_MASK >> PCIE_LINK_WIDTH_SHIFT);

            unsafe { *(&pcie_link_status as *const u16 as *const BusWidth) }
        };

        BusInfo { width, fun: 0 }
    }
}

pub(crate) struct HwControl {
    reg: Reg,
    flags: Flags,
}

impl HwControl {
    pub(crate) fn new(reg: Reg, flags: Flags) -> Self {
        if flags.has_swsm_on_load {
            let swsm = reg.er32(SWSM);
            reg.ew32(SWSM, swsm | SWSM_DRV_LOAD);
        } else if flags.has_ctrlext_on_load {
            let ctrl_ext = reg.er32(CTRL_EXT);
            reg.ew32(CTRL_EXT, ctrl_ext | CTRL_EXT_DRV_LOAD);
        }

        HwControl { reg, flags }
    }
}

impl Drop for HwControl {
    fn drop(&mut self) {
        if self.flags.has_swsm_on_load {
            let swsm = self.reg.er32(SWSM);
            self.reg.ew32(SWSM, swsm & !SWSM_DRV_LOAD);
        } else if self.flags.has_ctrlext_on_load {
            let ctrl_ext = self.reg.er32(CTRL_EXT);
            self.reg.ew32(CTRL_EXT, ctrl_ext & !CTRL_EXT_DRV_LOAD);
        }
    }
}
