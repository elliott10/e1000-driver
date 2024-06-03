#![allow(unused,missing_docs)]
#![no_std]

#[macro_use]
mod linux;
pub(crate) use linux::print::*;
use kernel::prelude::*;
pub mod e1000;
use e1000::E1000e;


struct Device {
    dev: E1000e,
}

#[no_mangle]
extern "C" fn e1000_new() -> usize {
    let dev = Box::init(kernel::init!(Device { dev: E1000e::new() })).unwrap();
    let ptr = unsafe { Box::into_raw(dev) as usize };
    pr_info!("e1000_new ptr {:#X}\n", ptr);
    ptr 
}
#[no_mangle]
extern "C" fn e1000_probe(ptr: usize) {
    unsafe {
        let dev = &mut *(ptr as *mut Device);
        dev.dev.probe();
    }
}

#[no_mangle]
extern "C" fn e1000_drop(ptr: usize) {
    unsafe {
        let _ = Box::from_raw(ptr as *mut Device);
    }
}

const __LOG_PREFIX: &[u8] = b"e1000-rs\0";
