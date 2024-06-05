use super::board::E1000MacType;
use crate::err::{Error, Result};
/// E1000e driver.
use crate::*;
use e1000::hw::Hw;

pub struct E1000<K>
where
    K: KernelFunc,
{
    mmio: usize,
    flash: Option<usize>,
    kernel: K,
    hw: Hw,
    status: E1000Status,
}

impl<K> E1000<K>
where
    K: KernelFunc,
{
    /// new one.
    pub fn new(kfun: K, device_id: u32) -> Result<Self> {
        let board: Board = Board::from_dev_id(device_id).ok_or_else(|| {
            error!("unknown device id: {:#X}", device_id);
            return Error::Unknown("unknown device id");
        })?;

        let mmio = kfun.bar_n_remap(0);
        info!("new e1000e driver, mmio: {:#X}, board: {:?}", mmio, board);

        let mut flash = None;
        let board_info = board.get_info();
        if board_info.flags.has_flash && board_info.mac < E1000MacType::e1000_pch_spt {
            let addr = kfun.bar_n_remap(1);
            if addr > 0 {
                info!("flash: {:#X}", addr);
                flash = Some(addr);
            } else {
                error!("remap bar1 failed");
                return Err(Error::Unknown("remap bar1 failed"));
            }
        }

        let hw = Hw::new(mmio, board);

        Ok(Self {
            mmio,
            flash,
            kernel: kfun,
            hw,
            status: E1000Status::default(),
        })
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

    pub fn init(&mut self) -> Result {
        info!("init begin ...");

        info!("init end");
        Ok(())
    }

    pub fn status_mut(&mut self) -> &mut E1000Status {
        &mut self.status
    }

}

impl<K> Drop for E1000<K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {
        debug!("unmap mmio {:#X}", self.mmio);
        self.kernel.iounmap(self.mmio);
        if let Some(flash) = self.flash {
            debug!("unmap flash {:#X}", flash);
            self.kernel.iounmap(flash);
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct E1000Status {
    pub stats: E1000HWStats,

    pub itr: u32,
    pub itr_setting: u32,
    pub tx_it: u16,
    pub rx_itr: u16,
}
