#![no_std]
#![allow(unused)]

extern crate alloc;

#[macro_use]
extern crate log;

mod utils;
pub mod e1000;
pub mod pci;

pub use volatile::Volatile;

pub trait Ext {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
