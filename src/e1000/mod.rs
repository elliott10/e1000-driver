mod e1000;
mod hw;
mod regs;
use core::fmt::{self, Display};
pub use hw::Board;
pub use self::e1000::*;

const PAGE_SIZE: usize = 4096;

pub trait KernelFunc: Clone {
    /// Page size (usually 4K)
    const PAGE_SIZE: usize = PAGE_SIZE;

    // 或请求分配irq

    /// Allocate 'contiguous physical memory' for DMA;
    /// Return (cpu virtual address, dma physical address) which is page aligned.
    //fn dma_alloc_coherent(pages: usize) -> usize;
    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize);

    /// Deallocate DMA memory by virtual address
    fn dma_free_coherent(&mut self, vaddr: usize, pages: usize);
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
