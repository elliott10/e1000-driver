use ::alloc::boxed::Box;
use alloc::{alloc, Layout};
mod board;
pub(crate) mod defines;
pub(crate) mod dev_id;
mod e1000;
pub mod err;
pub(crate) mod hw;
pub(crate) mod hw_semaphore;
mod mac;
pub(crate) mod params;
pub(crate) mod phy;
mod reg;
mod regs;
pub(crate) mod ring;
mod variant;
pub use self::e1000::*;
pub use board::*;
use core::{
    fmt::{self, Display},
    ptr::null_mut,
    u16,
};
pub use hw::{E1000HWStats, MacInfo, MacType, MediaType, PhyInfo, PhyType};
pub use ring::{RxDescExt, TxDesc};
const PAGE_SIZE: usize = 4096;

static mut KERNEL: *mut Box<dyn KernelFunc2> = null_mut();

pub fn register_kernel<K: KernelFunc2 + 'static>(k: K) {
    let layout = Layout::for_value(&k);
    unsafe {
        let ptr = alloc::alloc(layout) as *mut K;
        ptr.write(k);
        let boxed: Box<dyn KernelFunc2> = Box::from_raw(ptr);

        let layout = Layout::for_value(&boxed);
        let wrap = alloc::alloc(layout) as *mut Box<dyn KernelFunc2>;
        wrap.write(boxed);

        KERNEL = wrap;
    }
}
pub(crate) fn kfn() -> &'static Box<dyn KernelFunc2> {
    unsafe { &*KERNEL }
}

pub trait KernelFunc2 {
    /// BAR remap address
    fn bar_n_remap(&self, bar: usize) -> usize;

    /// TODO linux Duration 无法编译通过
    fn delay(&self, nanos: u64);
}
pub trait KernelFunc: Send + Sync + Clone {
    /// Page size (usually 4K)
    const PAGE_SIZE: usize = PAGE_SIZE;

    // 或请求分配irq

    /// Allocate 'contiguous physical memory' for DMA;
    /// Return (cpu virtual address, dma physical address) which is page aligned.
    //fn dma_alloc_coherent(pages: usize) -> usize;
    fn dma_alloc_coherent(&self, pages: usize) -> (usize, u64);

    /// Deallocate DMA memory by virtual address
    fn dma_free_coherent(&self, vaddr: usize, dma: u64, pages: usize);

    /// BAR remap address
    fn bar_n_remap(&self, bar: usize) -> usize;

    /// iounmap address
    fn iounmap(&self, addr: usize);

    /// TODO linux Duration 无法编译通过
    fn delay(&self, nanos: u64);

    /// PCIe 相关信息
    fn pcie_info(&self) -> E1000rPcieInfo;

    /// Pcie 读取
    fn pci_read_config_word(&self, where_: i32) -> u16;

    /// mtu 大于 ETH_DATA_LEN
    fn is_mtu_bg_eth(&self) -> bool;

    fn on_xmit_completed(&self, pkts: u32, bytes: u32);

    fn enable_net(&self);
    // fn num_msix_vectors(&self)->usize;
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
