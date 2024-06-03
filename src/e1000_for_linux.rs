#![allow(unused)]
#![no_std]

use kernel::prelude::*;

#[no_mangle]
extern "C" fn e1000_new() -> u64 {
    let d2 = 0;
    pr_info!("e1000_new ptr {:#X}\n", d2);
    d2
}
#[no_mangle]
extern "C" fn do_something(ptr: u64){
    pr_info!("do_something ptr {:#X}\n", ptr);
    unsafe{
    }
}

#[no_mangle]
extern "C" fn e1000_drop(ptr: u64) {
    unsafe {
    }
}


const __LOG_PREFIX: &[u8] = b"e1000";