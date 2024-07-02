use ::alloc::boxed::Box;
use alloc::{alloc, Layout};
mod board;
pub(crate) mod defines;
pub(crate) mod dev_id;
mod e1000;
pub mod err;
pub(crate) mod hw;
pub(crate) mod lock;
pub(crate) mod mac;
pub(crate) mod nvm;
pub(crate) mod params;
pub(crate) mod phy;
mod reg;
pub(crate) mod ring;
mod variant;
pub use self::e1000::*;
pub use board::*;
use core::{
    fmt::{self, Display},
    ptr::{null_mut, slice_from_raw_parts_mut},
    time::Duration,
    u16,
};
use err::{Error, Result};

pub use hw::E1000HWStats;
pub use params::Flags;
pub use phy::MediaType;

const PAGE_SIZE: usize = 4096;

static mut KERNEL: *mut Box<dyn KernelFunc> = null_mut();

pub fn register_kernel<K: KernelFunc + 'static>(k: K) {
    unsafe {
        if !KERNEL.is_null() {
            return;
        }
        let layout = Layout::for_value(&k);
        let ptr = alloc::alloc(layout) as *mut K;
        ptr.write(k);
        let boxed: Box<dyn KernelFunc> = Box::from_raw(ptr);

        let layout = Layout::for_value(&boxed);
        let wrap = alloc::alloc(layout) as *mut Box<dyn KernelFunc>;
        wrap.write(boxed);

        KERNEL = wrap;
    }
}
pub(crate) fn kfn() -> &'static Box<dyn KernelFunc> {
    unsafe {
        if KERNEL.is_null() {
            panic!("register_kernel not called");
        }
        &*KERNEL
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DMAInfo {
    pub dma_addr: u64,
    pub cpu_addr: usize,
    pub size: usize,
}

pub(crate) struct DMA(DMAInfo);

impl DMA {
    pub fn try_new(size: usize) -> Result<Self> {
        let info = kfn().dma_alloc_coherent(size);
        if info.cpu_addr == 0 {
            return Err(Error::TryReserveError);
        }
        unsafe {
            let data = &mut *slice_from_raw_parts_mut(info.cpu_addr as *mut u8, size);
            data.fill(0);
        }
        Ok(Self(info))
    }
    pub fn dma_addr(&self) -> u64 {
        self.0.dma_addr
    }

    pub fn cpu_addr(&self) -> usize {
        self.0.cpu_addr
    }

    pub fn size(&self) -> usize {
        self.0.size
    }
}

impl Drop for DMA {
    fn drop(&mut self) {
        kfn().dma_free_coherent(self.0)
    }
}

pub trait KernelFunc {
    /// BAR remap address
    fn bar_n_remap(&self, bar: usize) -> usize;

    /// delay
    fn delay(&self, duration: Duration);

    /// Pcie 读取
    fn pci_read_config_word(&self, where_: i32) -> u16;

    /// Pcie Capability offset
    fn pci_cap_offset(&self) -> i32;

    /// DMA 分配
    fn dma_alloc_coherent(&self, size: usize) -> DMAInfo;

    /// DMA 释放
    fn dma_free_coherent(&self, dma: DMAInfo);

    fn enable_net(&self);

    fn on_xmit_completed(&self, pkts: u32, bytes: u32);
    // fn wakeup_watchdog(&self);

    /// iounmap address
    fn iounmap(&self, addr: usize);
}

#[repr(C)]
pub struct E1000XmitConfig {
    pub timestamp: u64,
    pub segs: u32,
    pub ipv4: bool,
    pub no_fcs: bool,
    pub vlan_tag_present: bool,
}

#[repr(C)]
pub struct E1000rPcieInfo {
    pub bus_link_width: u16,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct MacAddress(pub [u8; 6]);

impl Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}
