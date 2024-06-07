use b82571::DevSpec82571;
use defines::*;

use super::ich8lan::DevSpecIch8lan;
use super::regs::*;
use crate::err::{Error, Result};
use crate::*;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use core::str;

pub(crate) const E1000_ICH_FWSM_RSPCIPHY: u32 = 0x00000040; // Reset PHY on PCI Reset
pub(crate) const E1000_ICH_FWSM_FW_VALID: u32 = 0x00008000; // FW established a valid mode
pub(crate) const E1000_ICH_FWSM_PCIM2PCI: u32 = 0x01000000; // ME PCIm-to-PCI active
pub(crate) const E1000_ICH_FWSM_PCIM2PCI_COUNT: u32 = 2000;
pub(crate) const E1000_ICH_MNG_IAMT_MODE: u32 = 0x2;

pub(crate) struct Hw<K>
where
    K: KernelFunc,
{
    pub(crate) kernel: K,
    pub(crate) mmio: usize,
    pub(crate) flash: Option<usize>,
    pub(crate) phy: PhyInfo,
    pub(crate) mac: MacInfo,
    pub(crate) nvm: NvmInfo,
    pub(crate) bus: BusInfo,
    pub(crate) flags: Flags,
    pub(crate) flags2: Flags2,
    pub(crate) variant: BoardVariant,
    pub(crate) status: E1000Status,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialOrd, PartialEq)]
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

#[repr(C)]
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MsType {
    HwDefault = 0,
    ForceMaster,
    ForceSlave,
    Auto,
}
impl Default for MsType {
    fn default() -> Self {
        Self::Auto
    }
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
impl Default for PhyType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for MediaType {
    fn default() -> Self {
        Self::media_type_unknown
    }
}

#[derive(Debug, Default)]
pub struct PhyInfo {
    pub kind: PhyType,

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

    pub(crate) ms_type: MsType,
    pub(crate) original_ms_type: MsType,
}

impl PhyInfo {
    pub(crate) fn copper_options(&mut self) {
        if self.media_type == MediaType::media_type_copper {
            self.mdix = AUTO_ALL_MODES;
            self.disable_polarity_correction = false;
            self.ms_type = MsType::HwDefault;
        }
    }
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

#[derive(Debug, PartialEq)]
pub(crate) enum NvmType {
    Unknown = 0,
    None,
    EepromSpi,
    FlashHw,
    FlashSw,
}
impl Default for NvmType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default)]
pub(crate) struct NvmInfo {
    pub(crate) kind: NvmType,
    pub(crate) nvm_override: NvmOverride,

    pub(crate) flash_bank_size: u32,
    pub(crate) flash_base_addr: u32,

    pub(crate) word_size: u16,
    pub(crate) delay_usec: u16,
    pub(crate) address_bits: u16,
    pub(crate) opcode_bits: u16,
    pub(crate) page_size: u16,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NvmOverride {
    None = 0,
    SpiSmall,
    SpiLarge,
}
impl Default for NvmOverride {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub(crate) struct BusInfo {
    pub(crate) width: BusWidth,
    pub(crate) fun: u16,
}

impl<K> Hw<K>
where
    K: KernelFunc,
{
    pub(crate) fn crc_stripping(&mut self) {
        self.flags2.crc_stripping = true;
        self.flags2.dflt_crc_stripping = true;
    }

    pub(crate) fn write_protect_nvm(&mut self) {
        if self.flags.is_ich {
            self.flags.read_only_nvm = true;
        }
    }
    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.mmio + offset) as *const T;
            ptr.read_volatile()
        }
    }
    pub(crate) fn er32(&self, offset: usize) -> u32 {
        self.read(offset)
    }
    fn ew32_prepare(&self) {
        let mut i = E1000_ICH_FWSM_PCIM2PCI_COUNT;

        while ((self.er32(FWSM) & E1000_ICH_FWSM_PCIM2PCI) != 0) && i > 0 {
            self.kernel.delay(50_000);
            i -= 1;
        }
    }
    pub(crate) fn ew32(&self, offset: usize, value: u32) {
        if self.flags2.pcim2pci_arbiter_wa {
            self.ew32_prepare();
        }
        unsafe {
            let ptr = (self.mmio + offset) as *mut u32;
            ptr.write_volatile(value);
        }
    }

    pub(crate) fn e1e_flush(&self) {
        self.er32(STATUS);
    }

    pub(crate) fn ral(&self, index: usize) -> usize {
        RA + index * 8
    }

    pub(crate) fn rah(&self, index: usize) -> usize {
        RA + index * 8 + 4
    }

    pub(crate) fn poll_eerd_eewr_done(&self, ee_reg: u8) -> Result {
        let mut reg;

        for _ in 0..100000 {
            if (ee_reg == NVM_POLL_READ) {
                reg = self.er32(EERD);
            } else {
                reg = self.er32(EEWR);
            }

            if (reg & NVM_RW_REG_DONE > 0) {
                return Ok(());
            }

            self.kernel.delay(5000);
        }

        Err(Error::Nvm("poll fail"))
    }
}

#[derive(Clone)]
pub(crate) struct HwRc<K>(NonNull<Hw<K>>)
where
    K: KernelFunc;

unsafe impl<K: KernelFunc> Send for HwRc<K> {}

impl<K> HwRc<K>
where
    K: KernelFunc,
{
    pub(crate) fn new(kfun: K, board: &BoardInfo) -> Self {
        unsafe {
            let mmio = kfun.bar_n_remap(0);
            let layout = Layout::new::<Hw<K>>();
            let ptr = alloc::alloc(layout) as *mut Hw<K>;
            let hw = &mut *ptr;
            hw.kernel = kfun;
            hw.mmio = mmio;
            hw.flash = None;
            hw.status = Default::default();
            hw.phy = Default::default();
            hw.mac = Default::default();
            hw.nvm = Default::default();
            hw.mac.kind = board.mac;
            hw.flags = board.flags;
            hw.flags2 = board.flags2;
            hw.variant = board.variant;
            hw.status.max_hw_frame_size = board.max_hw_frame_size;
            hw.status.pba = board.pba;

            Self(NonNull::new_unchecked(ptr))
        }
    }
    pub(crate) fn init(&mut self) -> Result {
        let hw = self.as_mut();

        if hw.flags.has_flash && hw.mac.kind < MacType::e1000_pch_spt {
            let addr = hw.kernel.bar_n_remap(1);
            if addr > 0 {
                info!("flash: {:#X}", addr);
                hw.flash = Some(addr);
            } else {
                error!("remap bar1 failed");
                return Err(Error::Unknown("remap bar1 failed"));
            }
        }
        Ok(())
    }

    pub(crate) fn drop(&mut self) {
        let base = self.as_mut();
        debug!("unmap mmio {:#X}", base.mmio);
        base.kernel.iounmap(base.mmio);
        if let Some(flash) = base.flash {
            debug!("unmap flash {:#X}", flash);
            base.kernel.iounmap(flash);
        }
        unsafe {
            let layout = Layout::new::<Hw<K>>();
            let ptr = self.0.as_ptr() as *mut u8;
            alloc::dealloc(ptr, layout);
        }
    }
}

impl<K> AsRef<Hw<K>> for HwRc<K>
where
    K: KernelFunc,
{
    fn as_ref(&self) -> &Hw<K> {
        unsafe { self.0.as_ref() }
    }
}

impl<K> AsMut<Hw<K>> for HwRc<K>
where
    K: KernelFunc,
{
    fn as_mut(&mut self) -> &mut Hw<K> {
        unsafe { self.0.as_mut() }
    }
}

pub(crate) enum DevSpec {
    E82571(DevSpec82571),
    E80003es2lan,
    Ich8lan(DevSpecIch8lan),
}
pub(crate) struct DevSpecParam<'a> {
    pub(crate) nvm: &'a mut NvmInfo,
    pub(crate) phy: &'a mut PhyInfo,
    pub(crate) mac: &'a mut MacInfo,
    pub(crate) mmio: usize,
    pub(crate) flash: &'a mut Option<usize>,
    pub(crate) strap: u32,
}

impl DevSpec {
    pub(crate) fn get_variants<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        match self {
            DevSpec::E82571(i) => i.get_variants(hw),
            DevSpec::E80003es2lan => Ok(()),
            DevSpec::Ich8lan(i) => i.get_variants(hw),
        }
    }

    pub(crate) fn get_bus_info<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        match self {
            DevSpec::E82571(i) => i.get_bus_info(hw),
            DevSpec::E80003es2lan => Ok(()),
            DevSpec::Ich8lan(i) => Ok(()),
        }
    }

    pub(crate) fn read_nvm<K: KernelFunc>(
        &mut self,
        hw: &mut Hw<K>,
        offset: usize,
        words: usize,
    ) -> Result<Vec<u16>> {
        match self {
            DevSpec::E82571(i) => i.read_nvm(hw, offset, words),
            DevSpec::E80003es2lan => todo!(),
            DevSpec::Ich8lan(i) => todo!(),
        }
    }
}
