#![allow(unused, missing_docs)]
#![no_std]

#[macro_use]
mod linux;
use core::ffi::{self, c_void};

use kernel::prelude::*;
pub(crate) use linux::print::*;
pub mod e1000;
use e1000::*;



pub struct Device {
    dev: E1000<KFun>,
}
#[derive(Clone)]
struct KFun {
    cb_bar_n_remap: E1000CbBarNremap,
    cb_iounmap: E1000CbIounmap,
    pdev: *const c_void,
}

unsafe impl Sync for KFun {}
unsafe impl Send for KFun {}

impl KernelFunc for KFun {
    fn dma_alloc_coherent(&self, pages: usize) -> (usize, usize) {
        kernel::sync::SpinLock::new

        (0, 0)
    }

    fn dma_free_coherent(&self, vaddr: usize, pages: usize) {}

    fn bar_n_remap(&self, bar: usize) -> usize {
        unsafe { (self.cb_bar_n_remap)(self.pdev, bar) }
    }
    fn iounmap(&self, addr: usize) {
        unsafe { (self.cb_iounmap)(addr) }
    }
}

type DevicePtr = *mut Device;
fn dev_mut<'a>(ptr: usize) -> &'a mut Device {
    unsafe { &mut *(ptr as DevicePtr) }
}

/// 通过bar获取到映射地址
/// return: 映射地址
pub type E1000CbBarNremap = unsafe extern "C" fn(pdev: *const c_void, bar: usize) -> usize;
/// 释放映射地址
pub type E1000CbIounmap = unsafe extern "C" fn(addr: usize);

#[repr(C)]
pub struct E1000Config {
    device_id: u32,
    cb_bar_n_remap: E1000CbBarNremap,
    cb_iounmap: E1000CbIounmap,
    pdev: *const c_void,
}

fn rfn<F>(f: F) -> i32
where
    F: FnOnce() -> core::result::Result<(), err::Error>,
{
    match f() {
        Ok(_) => 0,
        Err(e) => -1,
    }
}

#[no_mangle]
extern "C" fn e1000r_new(config: E1000Config) -> usize {
    let mut kfun = KFun {
        cb_bar_n_remap: config.cb_bar_n_remap,
        cb_iounmap: config.cb_iounmap,
        pdev: config.pdev,
    };
    let e1000 = match E1000::new(kfun, config.device_id) {
        Ok(o) => o,
        Err(_) => return 0,
    };

    let dev = Box::init(kernel::init!(Device { dev: e1000 })).unwrap();
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

#[no_mangle]
extern "C" fn e1000r_board_info(ptr: usize) -> BoardInfo {
    let dev = dev_mut(ptr);
    let board_info = dev.dev.board_info();
    board_info
}

#[no_mangle]
extern "C" fn e1000r_init(ptr: usize) -> i32 {
    let dev = dev_mut(ptr);
    rfn(|| {
        dev.dev.init()?;
        Ok(())
    })
}

#[no_mangle]
unsafe extern "C" fn e1000r_status<'a>(ptr: usize) -> &'a mut E1000Status {
    let dev = dev_mut(ptr);
    dev.dev.status_mut()
}
