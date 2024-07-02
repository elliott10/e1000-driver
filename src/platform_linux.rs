#![allow(unused, missing_docs, unreachable_pub)]
#![no_std]

#[macro_use]
mod linux;
use core::{
    alloc::Layout,
    ffi::{self, c_void},
    ptr::slice_from_raw_parts,
    time::Duration,
};

use ::alloc::collections;
pub(crate) use alloc::alloc;
pub(crate) use kernel::prelude::*;
pub(crate) use linux::print::*;
pub mod e1000;
use e1000::*;
use params::{InterruptMode, Settings};

#[derive(Clone)]
struct KFun {
    cb_bar_n_remap: E1000CbBarNremap,
    cb_iounmap: E1000CbIounmap,
    cb_udelay: E1000CbUdelay,
    cb_dma_alloc: E1000CbDmaAllocCoherent,
    cb_dma_free: E1000CbDmaFreeCoherent,
    cb_pci_read_config_word: E1000CbPciReadConfigWord,
    cb_on_xmit_completed: E1000CbOnXmitCompleted,
    cb_enable_net: E1000CbEnableNet,
    // cb_wakeup_watchdog: E1000CbWakeupWatchdog,
    cb_pcie_cap_offset: E1000CbPcieCapOffset,
    adapter: *const c_void,
}

unsafe impl Sync for KFun {}
unsafe impl Send for KFun {}
impl KernelFunc for KFun {
    fn bar_n_remap(&self, bar: usize) -> usize {
        unsafe { (self.cb_bar_n_remap)(self.adapter, bar) }
    }

    fn delay(&self, duration: Duration) {
        let secs = duration.as_secs();
        let micro = duration.subsec_micros();
        unsafe {
            if micro > 1000 {
                let n = micro / 1000;
                for _ in 0..n {
                    ((self.cb_udelay)(1000));
                }
            } else {
                ((self.cb_udelay)(micro));
            }

            let n = secs / 1_000;

            for _ in 0..n {
                ((self.cb_udelay)(1000));
            }
        }
    }

    fn pci_read_config_word(&self, where_: i32) -> u16 {
        unsafe { (self.cb_pci_read_config_word)(self.adapter, where_) }
    }

    fn pci_cap_offset(&self) -> i32 {
        unsafe { (self.cb_pcie_cap_offset)(self.adapter) }
    }

    fn dma_alloc_coherent(&self, size: usize) -> DMAInfo {
        let mut dma_addr = 0;
        let cpu_addr = unsafe { (self.cb_dma_alloc)(self.adapter, size, &mut dma_addr) };
        DMAInfo {
            dma_addr,
            cpu_addr,
            size,
        }
    }

    fn dma_free_coherent(&self, dma: DMAInfo) {
        unsafe { (self.cb_dma_free)(self.adapter, dma.cpu_addr, dma.dma_addr, dma.size) }
    }

    fn enable_net(&self) {
        unsafe { ((self.cb_enable_net)(self.adapter)) }
    }

    fn on_xmit_completed(&self, pkts: u32, bytes: u32) {
        unsafe { (self.cb_on_xmit_completed)(self.adapter, pkts, bytes) }
    }

    fn iounmap(&self, addr: usize) {
        unsafe { (self.cb_iounmap)(addr) }
    }

    // fn wakeup_watchdog(&self) {

    //     unsafe { ((self.cb_enable_net)(self.adapter)) }

    // }
}

fn e1000_mut<'a>(ptr: usize) -> &'a mut E1000 {
    unsafe { &mut *(ptr as *mut E1000) }
}

/// 通过bar获取到映射地址
/// return: 映射地址
pub type E1000CbBarNremap = unsafe extern "C" fn(pdev: *const c_void, bar: usize) -> usize;
/// 释放映射地址
pub type E1000CbIounmap = unsafe extern "C" fn(addr: usize);

pub type E1000CbUdelay = unsafe extern "C" fn(usecs: u32);

pub type E1000CbDmaAllocCoherent =
    unsafe extern "C" fn(pdev: *const c_void, size: usize, dma: &mut u64) -> usize;

pub type E1000CbDmaFreeCoherent =
    unsafe extern "C" fn(pdev: *const c_void, vaddr: usize, dma: u64, size: usize);

pub type E1000CbPciReadConfigWord = unsafe extern "C" fn(pdev: *const c_void, where_: i32) -> u16;
pub type E1000CbOnXmitCompleted = unsafe extern "C" fn(pdev: *const c_void, pkts: u32, bytes: u32);
pub type E1000CbOnRcv = unsafe extern "C" fn(pdev: *const c_void, pkt: E1000RcvPkt);

pub type E1000CbEnableNet = unsafe extern "C" fn(pdev: *const c_void);
pub type E1000CbWakeupWatchdog = unsafe extern "C" fn(pdev: *const c_void);
pub type E1000CbPcieCapOffset = unsafe extern "C" fn(pdev: *const c_void) -> i32;
#[repr(C)]
pub struct E1000Config {
    device_id: u32,
    settings: Settings,
    cb_bar_n_remap: E1000CbBarNremap,
    cb_iounmap: E1000CbIounmap,
    cb_udelay: E1000CbUdelay,
    cb_dma_alloc_coherent: E1000CbDmaAllocCoherent,
    cb_dma_free_coherent: E1000CbDmaFreeCoherent,
    cb_pci_read_config_word: E1000CbPciReadConfigWord,
    cb_on_xmit_completed: E1000CbOnXmitCompleted,
    cb_enable_net: E1000CbEnableNet,
    cb_pcie_cap_offset: E1000CbPcieCapOffset,
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
        adapter: config.adapter,
        cb_dma_alloc: config.cb_dma_alloc_coherent,
        cb_dma_free: config.cb_dma_free_coherent,
        cb_pci_read_config_word: config.cb_pci_read_config_word,
        cb_on_xmit_completed: config.cb_on_xmit_completed,
        cb_enable_net: config.cb_enable_net,
        cb_pcie_cap_offset: config.cb_pcie_cap_offset,
    };

    register_kernel(kfun);

    let e1000 = match E1000::new(config.device_id, config.settings) {
        Ok(o) => o,
        Err(_) => return 0,
    };

    unsafe {
        let layout = Layout::for_value(&e1000);
        let ptr = alloc::alloc(layout) as *mut E1000;
        ptr.write(e1000);
        ptr as usize
    }

    // let dev = Box::init(kernel::init!(Device { dev: e1000 })).unwrap();
    // let ptr = unsafe { Box::into_raw(dev) as usize };
    // ptr
}

#[no_mangle]
extern "C" fn e1000r_drop(ptr: usize) {
    unsafe {
        let dev = e1000_mut(ptr);
        dev.release();
        alloc::dealloc(ptr as _, Layout::for_value(dev));
    }
}

const __LOG_PREFIX: &[u8] = b"E1000 Rust\0";

#[no_mangle]
extern "C" fn e1000r_read_mac_addr(ptr: usize, mac: &mut [u8; 6]) {
    let dev = e1000_mut(ptr);
    let mac_new = dev.read_mac_addr_generic();
    mac.copy_from_slice(&mac_new.0);
}

#[no_mangle]
unsafe extern "C" fn e1000r_stats64<'a>(ptr: usize) -> E1000HWStats {
    let dev = e1000_mut(ptr);
    dev.get_stats64()
}

#[no_mangle]
unsafe extern "C" fn e1000r_transmit(
    ptr: usize,
    cfg: E1000XmitConfig,
    pkt: *const u8,
    len: u32,
) -> i32 {
    let dev = e1000_mut(ptr);
    let pkt = unsafe { &*slice_from_raw_parts(pkt, len as usize) };

    match dev.xmit(cfg, pkt) {
        Ok(n) => n as i32,
        Err(_) => -1,
    }
}

#[no_mangle]
unsafe extern "C" fn e1000r_irq_handle(ptr: usize, irq: i32) -> i32 {
    let dev = e1000_mut(ptr);
    rfn(|| dev.irq_handle(irq))
}
#[no_mangle]
unsafe extern "C" fn e1000r_irq_handle_msix_other(ptr: usize, irq: i32) -> i32 {
    let dev = e1000_mut(ptr);
    rfn(|| dev.irq_handle_msix_other(irq))
}
#[no_mangle]
unsafe extern "C" fn e1000r_irq_handle_msix_tx(ptr: usize, irq: i32) -> i32 {
    let dev = e1000_mut(ptr);

    dev.irq_handle_msix_tx();
    0
}

#[no_mangle]
unsafe extern "C" fn e1000r_irq_clean_tx(ptr: usize) -> bool {
    let dev = e1000_mut(ptr);
    dev.clean_tx_irq()
}
#[no_mangle]
extern "C" fn e1000r_irq_set_mode(ptr: usize, mode: InterruptMode) {
    let dev = e1000_mut(ptr);
    dev.set_interrupt_mode(mode)
}

#[no_mangle]
extern "C" fn e1000r_irq_get_mode(ptr: usize) -> InterruptMode {
    let dev = e1000_mut(ptr);
    dev.get_interrupt_mode()
}

#[no_mangle]
extern "C" fn e1000r_irq_disable(ptr: usize) {
    let dev = e1000_mut(ptr);
    dev.irq_disable();
}
#[no_mangle]
extern "C" fn e1000r_irq_enable(ptr: usize) {
    let dev = e1000_mut(ptr);
    dev.irq_enable();
}
#[no_mangle]
extern "C" fn e1000r_configure_msix(ptr: usize) {
    let dev = e1000_mut(ptr);
    dev.configure_msix();
}

#[no_mangle]
extern "C" fn e1000r_open(ptr: usize) -> i32 {
    let dev = e1000_mut(ptr);
    match dev.open() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}
#[no_mangle]
extern "C" fn e1000r_down(ptr: usize) -> i32 {
    let dev = e1000_mut(ptr);
    dev.down();

    return 0;
}
#[no_mangle]
extern "C" fn e1000_print_device_info(ptr: usize) {
    let dev = e1000_mut(ptr);

    dev.print_info();
}

#[no_mangle]
unsafe extern "C" fn e1000r_trigger_lsc(ptr: usize) {
    let dev = e1000_mut(ptr);
    dev.trigger_lsc()
}
#[no_mangle]
unsafe extern "C" fn e1000r_task_watchdog(ptr: usize) -> bool {
    let dev = e1000_mut(ptr);
    let link = dev.task_watchdog().unwrap_or(false);
    link
}

#[repr(C)]
pub struct E1000RcvPkt {
    pub data: *const u8,
    pub length: usize,
}

#[no_mangle]
unsafe extern "C" fn e1000r_clear_rx(
    ptr: usize,
    max: u32,
    on_rcv: E1000CbOnRcv,
    meta: *const c_void,
) -> usize {
    let dev = e1000_mut(ptr);
    let pkts = dev.clean_rx_irq(max);

    unsafe {
        for i in 0..pkts.len() {
            let pkt: &RcvPkt = &pkts[i];

            on_rcv(
                meta,
                E1000RcvPkt {
                    data: pkt.data.as_ptr(),
                    length: pkt.data.len(),
                },
            );
        }

        pkts.len()
    }
}
