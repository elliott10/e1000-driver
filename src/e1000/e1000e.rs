/// E1000e driver.

use crate::*;

pub struct E1000e{

}

impl E1000e {
    /// new one.
    pub fn new()->Self{
        Self{}
    }

    /// probe
    pub fn probe(&self){
        info!("probe! ");
    }

}