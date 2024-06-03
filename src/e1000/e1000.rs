/// E1000e driver.

use crate::*;

#[derive(Clone)]
pub struct E1000{
    mmio: usize,
}

impl E1000 {
    /// new one.
    pub fn new(mmio: usize)->Self{
        Self{
            mmio
        }
    }

    /// probe
    pub fn probe(&self){
        info!("probe! ");
    }

}