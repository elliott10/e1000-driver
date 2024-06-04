/// E1000e driver.

use crate::*;

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

#[derive(Clone)]
pub struct E1000{
    mmio: usize,
}

impl E1000 {
    /// new one.
    pub fn new(mmio: usize)->Self{
        debug!("new E1000 mmio base: [{:#X}]", mmio);
        
        Self{
            mmio
        }
    }

    /// probe
    pub fn probe(&self){
        info!("probe! ");
    }

}

impl Drop for E1000 {
    fn drop(&mut self) {
        debug!("drop");
    }
}