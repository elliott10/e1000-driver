use e1000::hw::Hw;

/// E1000e driver.
use crate::*;

pub struct E1000<K>
where
    K: KernelFunc,
{
    mmio: usize,
    kernel: K,
    hw: Hw,
    info: E1000RInfo,
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
            info: E1000RInfo::default(),
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

    /// board info
    pub fn board_info(&self) -> BoardInfo {
        self.hw.board_info()
    }

    /// set flash
    pub fn set_flash(&mut self, flash: usize) {
        info!("set flash: {:#X}", flash);
        self.hw.set_flash(flash);
    }

    pub fn init(&mut self) -> Result<(), usize> {
        info!("init begin ...");

        info!("init end");
        Ok(())
    }

    pub fn info_mut(&mut self) -> &mut E1000RInfo {
        &mut self.info
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

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct E1000RInfo {
    pub stats: E1000HWStats,

    pub itr: u32,
    pub itr_setting: u32,
    pub tx_it: u16,
    pub rx_itr: u16,
}
