// remove std lib
#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(asm_sym)]

use log::LevelFilter;

extern crate alloc;
extern crate opensbi_rt;
#[macro_use]
extern crate log;

mod net;
mod dns;
mod e1000_impls;

/// rust 入口函数
/// 
/// 进行操作系统的初始化，
#[no_mangle]
pub extern "C" fn main(_hart_id: usize, _device_tree_addr: usize) {

    log::set_max_level(LevelFilter::Info);

    net::init();
}
