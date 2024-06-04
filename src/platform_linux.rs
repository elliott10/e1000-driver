#![allow(unused, missing_docs)]
#![no_std]

#[macro_use]
mod linux;
use kernel::prelude::*;
pub(crate) use linux::print::*;
pub mod e1000;
use e1000::*;

pub struct Device {
    dev: E1000<KFun>,
}
#[derive(Clone)]
struct KFun {}

impl KernelFunc for KFun {
    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        (0, 0)
    }

    fn dma_free_coherent(&mut self, vaddr: usize, pages: usize) {}
}

type DevicePtr = *mut Device;
fn dev_mut<'a>(ptr: usize) -> &'a mut Device {
    unsafe { &mut *(ptr as DevicePtr) }
}

#[no_mangle]
extern "C" fn e1000r_new(mmio: usize, board: usize) -> usize {
    let dev = Box::init(kernel::init!(Device {
        dev: E1000::new(KFun {}, mmio, Board::from(board))
    }))
    .unwrap();
    let ptr = unsafe { Box::into_raw(dev) as usize };
    ptr
}

#[no_mangle]
extern "C" fn e1000r_drop(ptr: usize) {
    unsafe {
        let _ = Box::from_raw(ptr as DevicePtr);
    }
}

const __LOG_PREFIX: &[u8] = b"E1000 Rust\0";

#[no_mangle]
extern "C" fn e1000r_read_mac_addr(ptr: usize, mac: &mut [u8; 6]) {
    let dev = dev_mut(ptr);
    let mac_new = dev.dev.read_mac_addr_generic();
    mac.copy_from_slice(&mac_new.0);
}
