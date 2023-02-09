use core::sync::atomic::*;
use lazy_static::lazy_static;

extern "C" {
    fn end();
}

lazy_static! {
    static ref DMA_PADDR: AtomicUsize = AtomicUsize::new(0x81000000 as usize);
}

pub struct Kernfn;
impl e1000_driver::e1000::KernelFunc for Kernfn {
    fn dma_alloc_coherent(pages: usize) -> usize {
        let paddr = DMA_PADDR.fetch_add(0x1000 * pages, Ordering::SeqCst);
        info!("alloc DMA: paddr={:#x}, pages={}", paddr, pages);
        paddr
    }

    fn dma_free_coherent(paddr: usize, pages: usize) {
        info!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
    }
}