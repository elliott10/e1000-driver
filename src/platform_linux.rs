#![allow(unused, missing_docs, unreachable_pub)]
#![no_std]

#[macro_use]
mod linux;
use core::{
    ffi::{self, c_void},
    ptr::slice_from_raw_parts,
};

use ::alloc::collections;
pub(crate) use alloc::alloc;
pub(crate) use kernel::prelude::*;
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
    cb_udelay: E1000CbUdelay,
    cb_pcie_info: E1000CbPcieInfo,
    cb_mtu_bg_eth: E1000CbMtuBgEth,
    cb_dma_alloc: E1000CbDmaAllocCoherent,
    cb_dma_free: E1000CbDmaFreeCoherent,
    cb_pci_read_config_word: E1000CbPciReadConfigWord,
    adapter: *const c_void,
}

unsafe impl Sync for KFun {}
unsafe impl Send for KFun {}

impl KernelFunc for KFun {
    fn dma_alloc_coherent(&self, pages: usize) -> (usize, u64) {
        let mut dma = 0;
        let addr = unsafe { (self.cb_dma_alloc)(self.adapter, pages, &mut dma) };
        (addr, dma)
    }

    fn dma_free_coherent(&self, vaddr: usize, dma: u64, pages: usize) {
        unsafe { (self.cb_dma_free)(self.adapter, vaddr, dma, pages) }
    }

    fn bar_n_remap(&self, bar: usize) -> usize {
        unsafe { (self.cb_bar_n_remap)(self.adapter, bar) }
    }
    fn iounmap(&self, addr: usize) {
        unsafe { (self.cb_iounmap)(addr) }
    }

    fn delay(&self, nanos: u64) {
        unsafe {
            ((self.cb_udelay)((nanos / 1000) as u32));
        }
    }

    fn pcie_info(&self) -> E1000rPcieInfo {
        unsafe { (self.cb_pcie_info)(self.adapter) }
    }

    fn is_mtu_bg_eth(&self) -> bool {
        unsafe { ((self.cb_mtu_bg_eth)(self.adapter)) }
    }

    fn pci_read_config_word(&self, where_: i32) -> u16 {
        unsafe { (self.cb_pci_read_config_word)(self.adapter, where_) }
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

pub type E1000CbUdelay = unsafe extern "C" fn(usecs: u32);

pub type E1000CbPcieInfo = unsafe extern "C" fn(pdev: *const c_void) -> E1000rPcieInfo;

pub type E1000CbMtuBgEth = unsafe extern "C" fn(pdev: *const c_void) -> bool;

pub type E1000CbDmaAllocCoherent =
    unsafe extern "C" fn(pdev: *const c_void, pages: usize, dma: &mut u64) -> usize;

pub type E1000CbDmaFreeCoherent =
    unsafe extern "C" fn(pdev: *const c_void, vaddr: usize, dma: u64, pages: usize);

pub type E1000CbPciReadConfigWord = unsafe extern "C" fn(pdev: *const c_void, where_: i32) -> u16;

#[repr(C)]
pub struct E1000Config {
    device_id: u32,
    cb_bar_n_remap: E1000CbBarNremap,
    cb_iounmap: E1000CbIounmap,
    cb_udelay: E1000CbUdelay,
    cb_pcie_info: E1000CbPcieInfo,
    cb_mtu_bg_eth: E1000CbMtuBgEth,
    cb_dma_alloc_coherent: E1000CbDmaAllocCoherent,
    cb_dma_free_coherent: E1000CbDmaFreeCoherent,
    cb_pci_read_config_word: E1000CbPciReadConfigWord,
    adapter: *const c_void,
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
        cb_udelay: config.cb_udelay,
        cb_pcie_info: config.cb_pcie_info,
        adapter: config.adapter,
        cb_mtu_bg_eth: config.cb_mtu_bg_eth,
        cb_dma_alloc: config.cb_dma_alloc_coherent,
        cb_dma_free: config.cb_dma_free_coherent,
        cb_pci_read_config_word: config.cb_pci_read_config_word,
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
extern "C" fn e1000r_init(ptr: usize) -> i32 {
    let dev = dev_mut(ptr);
    rfn(|| {
        dev.dev.init()?;
        Ok(())
    })
}

#[no_mangle]
unsafe extern "C" fn e1000r_status<'a>(ptr: usize) -> &'static mut E1000Status {
    let dev = dev_mut(ptr);
    dev.dev.status_mut()
}

#[no_mangle]
unsafe extern "C" fn e1000r_transmit(ptr: usize, pkt: *const u8, len: u32) -> i32 {
    let dev = dev_mut(ptr);
    let pkt = unsafe { &*slice_from_raw_parts(pkt , len as usize) };

    match dev.dev.transmit(pkt) {
        Ok(n) => n as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn e1000r_irq_handle(ptr: usize, irq: i32) -> i32 {
    let dev = dev_mut(ptr);
    rfn(|| dev.dev.intr(irq))
}

#[no_mangle]
unsafe extern "C" fn e1000r_set_flags(ptr: usize, flags: Flags) {
    let dev = dev_mut(ptr);
    dev.dev.set_flags(flags);
}

#[no_mangle]
extern "C" fn e1000r_irq_disable(ptr: usize, misx_enabled: bool) {
    let dev = dev_mut(ptr);
    dev.dev.irq_disable(misx_enabled);
}

#[no_mangle]
extern "C" fn e1000r_get_info_all(ptr: usize) -> e1000r_info_all {
    let dev = dev_mut(ptr);
    let e1000 = &dev.dev;

    e1000r_info_all {
        flags: e1000.flags(),
        flags2: e1000.flags2(),
        mac: e1000.mac_info(),
    }
}

#[no_mangle]
extern "C" fn e1000r_open(ptr: usize) -> i32 {
    info!("open");

    let dev = dev_mut(ptr);
    rfn(|| dev.dev.open())
}

#[no_mangle]
extern "C" fn e1000_print_device_info(ptr: usize) {
    let dev = dev_mut(ptr);
    let e1000 = &mut dev.dev;
    e1000.print_info();
}

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct e1000r_info_all {
    pub flags: Flags,
    pub flags2: Flags2,
    pub mac: MacInfo,
}
