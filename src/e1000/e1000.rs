/// E1000e driver.

use crate::*;

pub struct E1000{

}

impl E1000 {
    /// new one.
    pub fn new()->Self{
        Self{}
    }

    /// probe
    pub fn probe(&self){
        info!("probe! ");
    }

}