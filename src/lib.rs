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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
