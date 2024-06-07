use alloc::{alloc, Layout};
mod board;
pub(crate) mod defines;
pub(crate) mod dev_id;
mod e1000;
pub mod err;
mod hw;
pub(crate) mod params;
mod regs;
pub(crate) mod ring;
pub use self::e1000::*;
pub use board::*;
use core::fmt::{self, Display};
pub use hw::{E1000HWStats, MacInfo, MacType, MediaType, PhyInfo, PhyType};
pub use ring::{TxDesc, RxDesc};

const PAGE_SIZE: usize = 4096;

pub trait KernelFunc: Send + Sync + Clone {
    /// Page size (usually 4K)
    const PAGE_SIZE: usize = PAGE_SIZE;

    // 或请求分配irq

    /// Allocate 'contiguous physical memory' for DMA;
    /// Return (cpu virtual address, dma physical address) which is page aligned.
    //fn dma_alloc_coherent(pages: usize) -> usize;
    fn dma_alloc_coherent(&self, pages: usize) -> (usize, usize);

    /// Deallocate DMA memory by virtual address
    fn dma_free_coherent(&self, vaddr: usize, pages: usize);

    /// BAR remap address
    fn bar_n_remap(&self, bar: usize) -> usize;

    /// iounmap address
    fn iounmap(&self, addr: usize);

    /// TODO linux Duration 无法编译通过
    fn delay(&self, nanos: u64);

    /// PCIe 相关信息
    fn pcie_info(&self) -> E1000rPcieInfo;


    /// mtu 大于 ETH_DATA_LEN
    fn is_mtu_bg_eth(&self)->bool;

    // fn num_msix_vectors(&self)->usize;
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
