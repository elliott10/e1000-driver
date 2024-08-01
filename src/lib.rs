#![no_std]
#![allow(unused)]
#![allow(invalid_reference_casting)]

extern crate alloc;

#[macro_use]
extern crate log;

mod utils;
pub mod e1000;

pub trait Ext {}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {

    }
}
