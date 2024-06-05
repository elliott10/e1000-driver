use e1000::board;

use super::board::{E1000MacType};
use super::defines;
use super::hw::{self, Hw};
use super::regs;
use super::ring::*;
use crate::err::{Error, Result};
use crate::*;

pub struct E1000<K>
where
    K: KernelFunc,
{
    pub(crate) flash: Option<usize>,
    kernel: K,
    pub(crate) hw: Hw,
    pub(crate) status: E1000Status,
    pub(crate) eiac_mask: u32,
    pub(crate) tx_ring: Ring<K>,
    pub(crate) rx_ring: Ring<K>,
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
        info!(
            "New E1000 rust driver, mmio: {:#X}, device: {:#X} board: {:?}",
            mmio, device_id, board
        );

        let mut flash = None;
        let board_info = board.new_default_info();
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
        let tx_ring = Ring::new(kfun.clone());
        let rx_ring = Ring::new(kfun.clone());

        Ok(Self {
            flash,
            kernel: kfun,
            hw,
            eiac_mask: 0,
            rx_ring,
            tx_ring,
            status: E1000Status::default(),
        })
    }

    ///  e1000_read_mac_addr_generic - Read device MAC address
    ///
    ///  Reads the device MAC address from the EEPROM and stores the value.
    ///  Since devices with two ports use the same EEPROM, we increment the
    /// last bit in the MAC address for the second port.
    pub fn read_mac_addr_generic(&self) -> MacAddress {
        MacAddress(self.get_ra(0))
    }

    /// board info
    pub fn board_info(&self) -> BoardInfo {
        self.hw.board_info.clone()
    }

    pub fn init(&mut self) -> Result {
        info!("Init begin ...");
        self.check_opention();
        self.init_rings();
        self.get_variants();

        info!("Init ok");
        Ok(())
    }

    pub fn status_mut(&mut self) -> &mut E1000Status {
        &mut self.status
    }

    fn check_opention(&mut self) {
        self.status.check_opention(self.board_info());
        self.crc_stripping();
        self.write_protect_nvm();
    }

    fn crc_stripping(&mut self) {
        self.hw.board_info.flags2.crc_stripping = true;
        self.hw.board_info.flags2.dflt_crc_stripping = true;
    }

    fn write_protect_nvm(&mut self) {
        if self.hw.board_info.flags.is_ich {
            self.hw.board_info.flags.read_only_nvm = true;
        }
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.hw.board_info.flags = flags;
    }

    fn init_rings(&self) {}

    /// interrupt handler
    pub fn intr(&mut self, irq: i32) -> Result {
        info!("intr {}!", irq);

        Ok(())
    }

    pub fn irq_enable(&mut self, is_msix_enabled: bool) {
        info!("Enable interrupts");

        if is_msix_enabled {
            // 当使用MSI-X中断时
            self.ew32(regs::EIAC_82574, self.eiac_mask & regs::EIAC_MASK_82574);
            self.ew32(
                regs::IMS,
                self.eiac_mask | defines::IMS_OTHER | defines::IMS_OTHER_MASK,
            );
        } else if self.hw.board_info.mac >= E1000MacType::e1000_pch_lpt {
            self.ew32(regs::IMS, defines::IMS_ENABLE_MASK | defines::IMS_ECCER);
        } else {
            self.ew32(regs::IMS, defines::IMS_ENABLE_MASK);
        }
        self.e1e_flush();
    }

    pub fn irq_disable(&mut self, is_msix_enabled: bool) {
        info!("Disable interrupts");

        self.ew32(regs::IMC, !0);

        if is_msix_enabled {
            self.ew32(regs::EIAC_82574, 0);
        }
        self.e1e_flush();
    }
    fn get_ra(&self, index: usize) -> [u8; 6] {
        let ral = self.er32(self.hw.ral(index));
        let rah = self.er32(self.hw.rah(index));
        [
            ral as u8,
            (ral >> 8) as u8,
            (ral >> 16) as u8,
            (ral >> 24) as u8,
            rah as u8,
            (rah >> 8) as u8,
        ]
    }

    fn set_ra(&self, index: usize, mac: [u8; 6]) {
        unsafe {
            let mut ral = 0;
            let mut rah = 0;

            ral = (mac[0] as u32)
                | ((mac[1] as u32) << 8)
                | ((mac[2] as u32) << 16)
                | ((mac[3] as u32) << 24);
            rah = (mac[4] as u32) | ((mac[5] as u32) << 8);

            self.ew32(self.hw.ral(index), ral);
            self.ew32(self.hw.rah(index), rah);
        }
    }
    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.hw.mmio + offset) as *const T;
            ptr.read_volatile()
        }
    }
    fn er32(&self, offset: usize) -> u32 {
        self.read(offset)
    }
    fn ew32_prepare(&self) {
        let mut i = hw::E1000_ICH_FWSM_PCIM2PCI_COUNT;

        while ((self.er32(regs::FWSM) & hw::E1000_ICH_FWSM_PCIM2PCI) != 0) && i > 0 {
            self.kernel.delay(50_000);
            i -= 1;
        }
    }
    fn ew32(&self, offset: usize, value: u32) {
        if self.hw.board_info.flags2.pcim2pci_arbiter_wa {
            self.ew32_prepare();
        }

        unsafe {
            let ptr = (self.hw.mmio + offset) as *mut u32;
            ptr.write_volatile(value);
        }
    }

    fn e1e_flush(&self) {
        self.er32(regs::STATUS);
    }
}

impl E1000Status {
    fn check_opention(&mut self, board: BoardInfo) {
        use super::params::*;
        //TODO 检查最大网卡数量

        self.tx_int_delay = DEFAULT_TIDV;
        self.tx_abs_int_delay = DEFAULT_TADV;

        self.rx_int_delay = DEFAULT_RDTR;
        self.rx_abs_int_delay = DEFAULT_RADV;

        if board.flags2.dma_burst {
            self.rx_int_delay = BURST_RDTR;
            self.rx_abs_int_delay = BURST_RADV;
        }
        /* Interrupt Throttling Rate */
        {
            let opt_name = "Interrupt Throttling Rate (ints/sec)";
            self.itr = DEFAULT_ITR;
            self.itr_setting = self.itr;

            match self.itr {
                0 => {
                    info!("{} turned off", opt_name);
                }
                1 => {
                    info!("{} set to dynamic mode", opt_name);
                    self.itr = 20000;
                }
                2 => {
                    info!("{} Invalid mode - setting default", opt_name);
                    self.itr_setting = DEFAULT_ITR;
                }
                3 => {
                    info!("{} set to dynamic conservative mode", opt_name);
                    self.itr = 20000;
                }
                4 => {
                    info!("{} set to simplified (2000-8000 ints) mode", opt_name);
                }
                _ => {
                    // Save the setting, because the dynamic bits change itr.
                    // Clear the lower two bits because they are used as control.
                    self.itr_setting &= !3;
                }
            }
        }

        //TODO Interrupt Mode
    }
}

impl<K> Drop for E1000< K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {
        debug!("unmap mmio {:#X}", self.hw.mmio);
        self.kernel.iounmap(self.hw.mmio);
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

    pub tx_int_delay: u32,
    pub tx_abs_int_delay: u32,

    pub rx_int_delay: u32,
    pub rx_abs_int_delay: u32,

    pub total_tx_bytes: u32,
    pub total_tx_packets: u32,
    pub total_rx_bytes: u32,
    pub total_rx_packets: u32,
}
