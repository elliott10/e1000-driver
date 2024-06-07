use super::defines;
use super::defines::*;
use super::hw::{self, DevSpec, Hw, HwRc, MacType};
use super::regs::*;
use super::ring::*;
use crate::err::{Error, Result};
use crate::*;
use core::alloc::Layout;
use core::borrow::Borrow;
use core::cmp::min;
use core::mem::{self, align_of_val};
use core::ptr::NonNull;
use e1000::board;
use ffi::CStr;

pub struct E1000<K>
where
    K: KernelFunc,
{
    pub(crate) eiac_mask: u32,
    pub(crate) tx_ring: Ring<K>,
    pub(crate) rx_ring: Ring<K>,
    pub(crate) hw: HwRc<K>,
    pub(crate) dev_spec: DevSpec,
}

impl<K> E1000<K>
where
    K: KernelFunc,
{
    /// new one.
    pub fn new(kfun: K, device_id: u32) -> Result<Self> {
        let board_type: BoardType = BoardType::from_dev_id(device_id).ok_or_else(|| {
            error!("unknown device id: {:#X}", device_id);
            return Error::Unknown("unknown device id");
        })?;
        let board_info = board_type.new_default_info();
        let mut hw_rc = HwRc::new(kfun.clone(), &board_info);
        hw_rc.init()?;

        let mmio = hw_rc.as_ref().mmio;

        info!(
            "New E1000 rust driver, mmio: {:#X}, device: {:#X}, board: {:?}, variant: {:?}",
            mmio, device_id, board_type, board_info.variant
        );

        let tx_ring = Ring::new(kfun.clone());
        let rx_ring = Ring::new(kfun.clone());

        let dev_spec = match board_info.variant {
            BoardVariant::B82571 => DevSpec::E82571(Default::default()),
            BoardVariant::Ich8lan => DevSpec::Ich8lan(Default::default()),
        };

        Ok(Self {
            hw: hw_rc,
            eiac_mask: 0,
            rx_ring,
            tx_ring,
            dev_spec,
        })
    }

    fn kernel(&self) -> &K {
        &self.hw.as_ref().kernel
    }

    ///  e1000_read_mac_addr_generic - Read device MAC address
    ///
    ///  Reads the device MAC address from the EEPROM and stores the value.
    ///  Since devices with two ports use the same EEPROM, we increment the
    /// last bit in the MAC address for the second port.
    pub fn read_mac_addr_generic(&mut self) -> MacAddress {
        let mac = self.get_ra(0);
        self.hw.as_mut().mac.addr.copy_from_slice(&mac);
        MacAddress(mac)
    }

    pub fn init(&mut self) -> Result {
        info!("Init begin ...");
        self.check_opention();
        self.read_mac_addr_generic();
        self.init_rings();
        self.get_variants()?;
        self.dev_spec.get_bus_info(self.hw.clone().as_mut());
        self.hw.as_mut().phy.autoneg_wait_to_complete = false;
        self.hw.as_mut().phy.copper_options();
        if self.is_enable_mng_pass_thru() {
            self.hw.as_mut().flags.mng_pt_enabled = true;
        }

        let mut binding = self.hw.clone();
        let hw2 = binding.as_mut();
        self.dev_spec.reset_hw(hw2);

        self.reset();

        info!("Init ok");
        Ok(())
    }

    pub fn status_mut(&mut self) -> &mut E1000Status {
        &mut self.hw.as_mut().status
    }

    pub fn flags(&self) -> Flags {
        self.hw.as_ref().flags.clone()
    }
    pub fn flags2(&self) -> Flags2 {
        self.hw.as_ref().flags2.clone()
    }

    fn check_opention(&mut self) {
        let flags2 = self.hw.as_ref().flags2.clone();

        self.hw.as_mut().status.check_opention(&flags2);
        self.hw.as_mut().crc_stripping();
        self.hw.as_mut().write_protect_nvm();
    }

    fn init_rings(&self) {}

    /// interrupt handler
    pub fn intr(&mut self, irq: i32) -> Result {
        info!("intr {}!", irq);

        Ok(())
    }
    pub fn set_flags(&mut self, flags: Flags) {
        self.hw.as_mut().flags = flags;
    }
    pub fn irq_enable(&mut self, is_msix_enabled: bool) {
        info!("Enable interrupts");

        if is_msix_enabled {
            // 当使用MSI-X中断时
            self.ew32(EIAC_82574, self.eiac_mask & EIAC_MASK_82574);
            self.ew32(IMS, self.eiac_mask | IMS_OTHER | IMS_OTHER_MASK);
        } else if self.hw.as_ref().mac.kind >= MacType::e1000_pch_lpt {
            self.ew32(IMS, IMS_ENABLE_MASK | IMS_ECCER);
        } else {
            self.ew32(IMS, IMS_ENABLE_MASK);
        }
        self.e1e_flush();
    }
    fn get_variants(&mut self) -> Result {
        let mut hw = self.hw.clone();
        self.dev_spec.get_variants(hw.as_mut())
    }
    /// get mac info
    pub fn mac_info(&self) -> MacInfo {
        self.hw.as_ref().mac.clone()
    }

    fn ew32(&self, offset: usize, value: u32) {
        self.hw.as_ref().ew32(offset, value);
    }

    fn er32(&self, offset: usize) -> u32 {
        self.hw.as_ref().er32(offset)
    }
    fn e1e_flush(&self) {
        self.hw.as_ref().e1e_flush();
    }

    pub fn irq_disable(&mut self, is_msix_enabled: bool) {
        info!("Disable interrupts");

        self.ew32(IMC, !0);

        if is_msix_enabled {
            self.ew32(EIAC_82574, 0);
        }
        self.e1e_flush();
    }
    fn get_ra(&self, index: usize) -> [u8; 6] {
        let ral = self.er32(self.hw.as_ref().ral(index));
        let rah = self.er32(self.hw.as_ref().rah(index));
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

            let reg = self.hw.as_ref();
            self.ew32(reg.ral(index), ral);
            self.ew32(reg.rah(index), rah);
        }
    }

    pub fn print_info(&mut self) {
        let pba_str = [0; PBANUM_LENGTH];
        let hw = self.hw.as_ref();
        let mac_kind = hw.mac.kind;
        let phy_kind = hw.phy.kind;

        info!(
            "(PCI Express:2.5GT/s:{:?}) {}M",
            hw.bus.width,
            MacAddress(hw.mac.addr)
        );

        info!(
            "Intel(R) PRO/{} Network Connection",
            if hw.phy.kind == PhyType::Ife {
                "10/100"
            } else {
                "1000"
            }
        );
        let pba = self.read_pba_string_generic().unwrap();
        let p = if pba.len() > 0 {
            let s = CStr::from_bytes_until_nul(&pba).unwrap();
            s.to_str().unwrap()
        } else {
            "unknown"
        };

        info!("MAC: {:?}, PHY: {:?}, PBA No: {}", mac_kind, phy_kind, p);
    }

    fn read_pba_string_generic(&mut self) -> Result<[u8; PBANUM_LENGTH]> {
        let hw = self.hw.as_mut();
        let mut pba_num = [0; PBANUM_LENGTH];
        let mut nvm_data = self.dev_spec.read_nvm(hw, NVM_PBA_OFFSET_0, 1)?[0];
        let mut pba_ptr = self.dev_spec.read_nvm(hw, NVM_PBA_OFFSET_1, 1)?[0];

        if (nvm_data != NVM_PBA_PTR_GUARD) {
            info!("NVM PBA number is not stored as string");
            /* extract hex string from data and pba_ptr */
            pba_num[0] = ((nvm_data >> 12) & 0xF) as u8;
            pba_num[1] = ((nvm_data >> 8) & 0xF) as u8;
            pba_num[2] = ((nvm_data >> 4) & 0xF) as u8;
            pba_num[3] = (nvm_data & 0xF) as u8;
            pba_num[4] = ((pba_ptr >> 12) & 0xF) as u8;
            pba_num[5] = ((pba_ptr >> 8) & 0xF) as u8;
            pba_num[6] = b'-';
            pba_num[7] = 0;
            pba_num[8] = ((pba_ptr >> 4) & 0xf) as u8;
            pba_num[9] = (pba_ptr & 0xF) as u8;

            /* put a null character on the end of our string */
            pba_num[10] = b'\0';

            /* switch all the data but the '-' to hex char */
            for offset in 0..10 {
                if pba_num[offset] < 0xA {
                    pba_num[offset] += b'0';
                } else if pba_num[offset] < 0x10 {
                    pba_num[offset] += b'A' - 0xA;
                }
            }
            return Ok(pba_num);
        }

        let mut length: u16 = self.dev_spec.read_nvm(hw, pba_ptr, 1)?[0];

        if length == 0xFFFF || length == 0 {
            error!("NVM PBA number section invalid length\n");
            return Err(Error::Nvm("NVM PBA number section invalid length"));
        }
        /* check if pba_num buffer is big enough */
        if (pba_num.len() < ((length as usize * 2) - 1)) {
            error!("PBA string buffer too small\n");
            return Err(Error::TryReserveError);
        }

        /* trim pba length from start of string */
        pba_ptr += 1;
        length -= 1;

        for offset in 0..length {
            nvm_data = self.dev_spec.read_nvm(hw, pba_ptr + offset, 1)?[0];
            pba_num[offset as usize * 2] = (nvm_data >> 8) as u8;
            pba_num[(offset as usize * 2) + 1] = (nvm_data & 0xFF) as u8;
        }
        pba_num[(length - 1) as usize * 2] = b'\0';

        Ok(pba_num)
    }

    fn is_enable_mng_pass_thru(&mut self) -> bool {
        // let manc = self.er32(MANC);

        // if manc & MANC_RCV_TCO_EN == 0 {
        //     return false;
        // }

        // if self.mac.has_fwsm {
        //     let fwsm = self.er32(FWSM);
        //     let factps = self.er32(FACTPS);

        //     if factps & FACTPS_MNGCG == 0
        //         && (fwsm & FWSM_MODE_MASK) == (e1000_mng_mode_pt << FWSM_MODE_SHIFT)
        //     {
        //         return true;
        //     }
        // } else if self.mac.kind == MacType::e1000_82574 || self.mac.kind == MacType::e1000_82573 {
        //     let mut data: u16 = 0;
        //     let ret_val = self.read_nvm(NVM_INIT_CONTROL2_REG, 1, &mut data);

        //     if ret_val != 0 {
        //         return false;
        //     }

        //     if factps & FACTPS_MNGCG == 0
        //         && (data & NVM_INIT_CTRL2_MNGM) == (e1000_mng_mode_pt << 13)
        //     {
        //         return true;
        //     }
        // } else if manc & MANC_SMBUS_EN != 0 && manc & MANC_ASF_EN == 0 {
        //     return true;
        // }

        false
    }

    pub fn reset(&mut self) -> Result {
        let mut _hw = self.hw.clone();
        let hw = _hw.as_mut();
        let mut pba = hw.status.pba;
        let mut tx_space = 0;
        let mut min_tx_space = 0;
        let mut min_rx_space = 0;

        hw.ew32(PBA, pba);

        if hw.status.max_hw_frame_size > (VLAN_ETH_FRAME_LEN + ETH_FCS_LEN) as u32 {
            pba = hw.er32(PBA);
            tx_space = pba >> 16;
            pba &= 0xffff;
            /* the Tx fifo also stores 16 bytes of information about the Tx
             * but don't include ethernet FCS because hardware appends it
             */
            min_tx_space = (hw.status.max_frame_size + mem::size_of::<TxDesc>() as u32
                - ETH_FCS_LEN as u32)
                * 2;
            min_tx_space = align_to(min_tx_space, 1024);
            min_tx_space >>= 10;
            /* software strips receive CRC, so leave room for it */
            min_rx_space = hw.status.max_frame_size;
            min_rx_space = align_to(min_rx_space, 1024);
            min_rx_space >>= 10;

            /* If current Tx allocation is less than the min Tx FIFO size,
             * and the min Tx FIFO size is less than the current Rx FIFO
             * allocation, take space away from current Rx allocation
             */
            if ((tx_space < min_tx_space) && ((min_tx_space - tx_space) < pba)) {
                pba -= min_tx_space - tx_space;

                /* if short on Rx space, Rx wins and must trump Tx
                 * adjustment
                 */
                if (pba < min_rx_space) {
                    pba = min_rx_space;
                }
            }

            hw.ew32(PBA, pba);
        }

        /* flow control settings
         *
         * The high water mark must be low enough to fit one full frame
         * (or the size used for early receive) above it in the Rx FIFO.
         * Set it to the lower of:
         * - 90% of the Rx FIFO size, and
         * - the full Rx FIFO size minus one full frame
         */
        if hw.flags.disable_fc_pause_time {
            hw.fc.pause_time = 0xFFFF;
        } else {
            hw.fc.pause_time = FC_PAUSE_TIME;
        }
        hw.fc.send_xon = true;
        hw.fc.current_mode = hw.fc.requested_mode;

        let is_mtu_bg_than_data = hw.kernel.is_mtu_bg_eth();

        match hw.mac.kind {
            MacType::e1000_ich9lan | MacType::e1000_ich10lan => {
                // if adapter.netdev.mtu > ETH_DATA_LEN {
                //     let pba = 14;
                //     ew32!("PBA", pba);
                //     fc.high_water = 0x2800;
                //     fc.low_water = fc.high_water - 8;
                // } else {
                //     // fallthrough to default case
                // }
            }

            MacType::e1000_pchlan => {
                /* Workaround PCH LOM adapter hangs with certain network
                 * loads.  If hangs persist, try disabling Tx flow control.
                 */
                if is_mtu_bg_than_data {
                    hw.fc.high_water = 0x3500;
                    hw.fc.low_water = 0x1500;
                } else {
                    hw.fc.high_water = 0x5000;
                    hw.fc.low_water = 0x3000;
                }
                hw.fc.refresh_time = 0x1000;
            }
            MacType::e1000_pch2lan
            | MacType::e1000_pch_lpt
            | MacType::e1000_pch_spt
            | MacType::e1000_pch_cnp
            | MacType::e1000_pch_tgp
            | MacType::e1000_pch_adp
            | MacType::e1000_pch_mtp
            | MacType::e1000_pch_lnp
            | MacType::e1000_pch_ptp
            | MacType::e1000_pch_nvp => {
                hw.fc.refresh_time = 0xFFFF;
                hw.fc.pause_time = 0xFFFF;

                if !is_mtu_bg_than_data {
                    hw.fc.high_water = 0x05C20;
                    hw.fc.low_water = 0x05048;
                } else {
                    let pba = 14;
                    hw.ew32(PBA, pba);
                    hw.fc.high_water = ((pba << 10) * 9 / 10) & FCRTH_RTH;
                    hw.fc.low_water = ((pba << 10) * 8 / 10) & FCRTL_RTL;
                }
            }
            _ => {
                let hwm = min(
                    ((pba << 10) * 9 / 10),
                    ((pba << 10) - hw.status.max_frame_size),
                );
                hw.fc.high_water = hwm & FCRTH_RTH; /* 8-byte granularity */
                hw.fc.low_water = hw.fc.high_water - 8;
            }
        }

        hw.status.tx_fifo_limit = min((hw.er32(PBA) << 10) - 96, 24 << 10);

        info!("fifo limit: {}", hw.status.tx_fifo_limit);
        /* Disable Adaptive Interrupt Moderation if 2 full packets cannot
         * fit in receive buffer.
         */
        if hw.status.itr_setting & 0x3 > 0 {
            let max_frame_size = hw.status.max_frame_size * 2;
            let pba_value = pba << 10;

            if max_frame_size > pba_value {
                if !(hw.flags2.disable_aim) {
                    info!("Interrupt Throttle Rate off");
                    hw.flags2.disable_aim = true;
                    self.e1000e_write_itr(0);
                }
            } else if hw.flags2.disable_aim {
                info!("Interrupt Throttle Rate on");
                hw.flags2.disable_aim = false;
                hw.status.itr = 20000;
                self.e1000e_write_itr(hw.status.itr);
            }
        }

        Ok(())
    }

    /**
     * e1000e_write_itr - write the ITR value to the appropriate registers
     * @adapter: address of board private structure
     * @itr: new ITR value to program
     *
     * e1000e_write_itr determines if the adapter is in MSI-X mode
     * and, if so, writes the EITR registers with the ITR value.
     * Otherwise, it writes the ITR value into the ITR register.
     **/
    fn e1000e_write_itr(&mut self, itr: u32) {
        let hw = self.hw.as_mut();
        let new_itr = if itr > 0 { 1000000000 / (itr * 256) } else { 0 };
        // if hw.msix_entryies.is_empty() {
        hw.ew32(ITR, new_itr);
        // }else{
        // for i in 0..hw.msix_entryies.len() {
        //     hw.ew32(EITR(i), new_itr);
        // }
        // }
    }
    /// open 
    pub fn open(&mut self) -> Result {



        Ok(())
    }
}

impl E1000Status {
    fn check_opention(&mut self, flags2: &Flags2) {
        use super::params::*;
        //TODO 检查最大网卡数量

        self.tx_int_delay = DEFAULT_TIDV;
        self.tx_abs_int_delay = DEFAULT_TADV;

        self.rx_int_delay = DEFAULT_RDTR;
        self.rx_abs_int_delay = DEFAULT_RADV;

        if flags2.dma_burst {
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

impl<K> Drop for E1000<K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {
        // debug!("unmap mmio {:#X}", self.hw.mmio);
        // self.kernel().iounmap(self.hw.mmio);
        // if let Some(flash) = self.flash {
        //     debug!("unmap flash {:#X}", flash);
        //     self.kernel().iounmap(flash);
        // }
        self.hw.drop();
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

    pub max_frame_size: u32,
    pub min_frame_size: u32,

    pub total_tx_bytes: u32,
    pub total_tx_packets: u32,
    pub total_rx_bytes: u32,
    pub total_rx_packets: u32,

    pub tx_fifo_limit: u32,

    pub link_speed: u16,
    pub link_duplex: u16,
    pub max_hw_frame_size: u32,
    pub eeprom_wol: u32,
    pub wol: u32,
    pub pba: u32,
}
fn align_to(value: u32, alignment: u32) -> u32 {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}
