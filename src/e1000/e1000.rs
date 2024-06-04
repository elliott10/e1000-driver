use e1000::hw::Hw;

/// E1000e driver.
use crate::*;

#[derive(Clone)]
pub struct E1000<K>
where
    K: KernelFunc,
{
    mmio: usize,
    kernel: K,
    hw: Hw,
}

impl<K> E1000<K>
where
    K: KernelFunc,
{
    /// new one.
    pub fn new(kfun: K, mmio: usize, board: Board) -> Self {
        info!("New E1000 mmio base: [{:#X}], board: {:?}", mmio, board);
        let hw = Hw::new(mmio, board);

        Self {
            mmio,
            kernel: kfun,
            hw,
        }
    }

    ///  e1000_read_mac_addr_generic - Read device MAC address
    ///
    ///  Reads the device MAC address from the EEPROM and stores the value.
    ///  Since devices with two ports use the same EEPROM, we increment the
    /// last bit in the MAC address for the second port.
    pub fn read_mac_addr_generic(&self) -> MacAddress {
        MacAddress(self.hw.get_ra(0))
    }
}

impl<K> Drop for E1000<K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {
        debug!("drop");
    }
}
