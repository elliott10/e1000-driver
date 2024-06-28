use super::defines::*;
use super::hw::{self, *};
use super::reg::*;
use super::ring::*;
use crate::err::{Error, Result};
use crate::mac::*;
use crate::nvm::*;
use crate::phy::*;
use crate::*;
use core::alloc::Layout;
use core::cmp::min;
use core::mem::{self, align_of_val, size_of};
use core::ptr::NonNull;
use e1000::variant::{self, Variant};
use ffi::CStr;
use params::{Flags3, InterruptMode, Params, Settings};

static mut CARD_FOUND: usize = 0;

const TX_RING_SIZE: usize = 256;
const RX_RING_SIZE: usize = 256;

pub struct E1000 {
    pub(crate)reg: Reg,
    pub(crate) params: Params,
    pub(crate) mac: MacInfo,
    pub(crate) phy: PhyInfo2,
    pub(crate) nvm: NvmInfo,
    pub(crate) bus: BusInfo,
    pub(crate) fc: FcInfo,
    pub(crate) eiac_mask: u32,
    rx_ring: Ring<RxDescExt>,
    tx_ring: Ring<TxDesc>,
    hw_stats: E1000HWStats,
    is_link_up: bool,
    variant: Variant,
}

impl E1000 {
    /// new one.
    pub fn new(device_id: u32, settings: Settings) -> Result<Self> {
        let board_type: BoardType = BoardType::from_dev_id(device_id).ok_or_else(|| {
            error!("unknown device id: {:#X}", device_id);
            return Error::Unknown("unknown device id");
        })?;
        let mtu = if settings.mtu > 0 { settings.mtu } else { 1500 };
        let board_info = board_type.new_default_info();
        let card_found = unsafe { CARD_FOUND } + 1;

        let mut reg = Reg::new(&board_info)?;

        let mut variant = Variant::new(&board_info);

        let res = variant.get_variants(reg, &board_info)?;
        let mac = res.mac;
        reg = res.reg;
        let phy = res.phy;
        let nvm = res.nvm;
        let mut flags = res.flags;

        let mut params = Params {
            settings,
            flags,
            card_found,
            pba: board_info.pba,
            max_hw_frame_size: board_info.max_hw_frame_size,
            mtu,
            max_frame_size: (mtu + VLAN_ETH_DATA_LEN + ETH_FCS_LEN) as _,
            min_frame_size: (ETH_ZLEN + ETH_FCS_LEN) as _,
            ..Default::default()
        };

        params.check_options();

        let bus = BusInfo::new();
        let fc = FcInfo::default();

        let tx_ring = Ring::new(2048)?;
        let rx_ring = Ring::new(2048)?;

        let mut e1000 = Self {
            reg,
            params,
            mac,
            fc,
            phy,
            nvm,
            bus,
            eiac_mask: 0,
            is_link_up: false,
            rx_ring,
            tx_ring,
            variant,
            hw_stats: Default::default(),
        };
        e1000.irq_disable();

        unsafe { CARD_FOUND = card_found };

        e1000.mac.read_mac_addr_generic(reg);
        e1000.phy.autoneg_wait_to_complete = false;

        

        e1000.phy.copper_options();

        /* Initialize link parameters. User can change them with ethtool */
        e1000.mac.autoneg = true;
        e1000.fc.requested_mode = FcMode::Default;
        e1000.fc.current_mode = FcMode::Default;
        e1000.phy.autoneg_advertised = 0x2f;

        //TODO ptp_init

        e1000.reset()?;

        if e1000.params.flags.has_amt{
            e1000.get_hw_control();
        }

        info!(
            "New E1000 rust driver, mmio: {:#X}, flash: {:?} bus: {:?}",
            reg.mmio, reg.flash, e1000.bus.width
        );

        info!(
            "device: {:#X}, board: {:?}, variant: {:?}, itr: {:?}",
            device_id, board_type, board_info.variant, e1000.params.int_mode
        );
        info!(
            "tx frame size: {}, rx frame size {}",
            e1000.tx_ring.buff_len, e1000.rx_ring.buff_len
        );
        Ok(e1000)
    }

    ///  e1000_read_mac_addr_generic - Read device MAC address
    ///
    ///  Reads the device MAC address from the EEPROM and stores the value.
    ///  Since devices with two ports use the same EEPROM, we increment the
    /// last bit in the MAC address for the second port.
    pub fn read_mac_addr_generic(&mut self) -> MacAddress {
        self.mac.read_mac_addr_generic(self.reg);
        MacAddress(self.mac.addr)
    }

    fn eeprom_checks(&mut self) -> Result {
        if (self.mac.kind != MacType::e1000_82573) {
            return Ok(());
        }
        // TODO
        // ret_val = self.read_nvm(NVM_INIT_CONTROL2_REG, 1);
        // le16_to_cpus(&buf);
        // if (!ret_val && (!(buf & BIT(0)))) {
        // 	/* Deep Smart Power Down (DSPD) */
        // 	dev_warn(&adapter->pdev->dev,
        // 		 "Warning: detected DSPD enabled in EEPROM\n");
        // }

        Ok(())
    }

    pub fn flags(&self) -> Flags3 {
        self.params.flags.clone()
    }

    pub fn irq_enable(&mut self) {
        match &self.params.int_mode {
            InterruptMode::MSIX => {
                self.ew32(EIAC_82574, self.eiac_mask & EIAC_MASK_82574);
                self.ew32(IMS, self.eiac_mask | IMS_OTHER | IMS_OTHER_MASK);
            }
            _ => {
                if self.mac.kind >= MacType::e1000_pch_lpt {
                    self.ew32(IMS, IMS_ENABLE_MASK | IMS_ECCER);
                } else {
                    self.ew32(IMS, IMS_ENABLE_MASK);
                }
            }
        }

        self.e1e_flush();
    }

    /**
     * reset - bring the hardware into a known good state
     *
     * This function boots the hardware and enables some settings that
     * require a configuration cycle of the hardware - those cannot be
     * set/changed during runtime. After reset the device needs to be
     * properly configured for Rx, Tx etc.
     */
    fn reset(&mut self) -> Result {
        let mut pba = self.params.pba;
        let mut tx_space = 0;
        let mut min_tx_space = 0;
        let mut min_rx_space = 0;

        self.ew32(PBA, pba);

        if self.params.max_hw_frame_size > (VLAN_ETH_FRAME_LEN + ETH_FCS_LEN) as u32 {
            pba = self.er32(PBA);
            tx_space = pba.wrapping_shr(16);
            pba &= 0xffff;
            /* the Tx fifo also stores 16 bytes of information about the Tx
             * but don't include ethernet FCS because hardware appends it
             */
            min_tx_space = (self.params.max_frame_size + mem::size_of::<TxDesc>() as u32
                - ETH_FCS_LEN as u32)
                * 2;
            min_tx_space = align_to(min_tx_space, 1024);
            min_tx_space >>= 10;
            /* software strips receive CRC, so leave room for it */
            min_rx_space = self.params.max_frame_size;
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

            self.ew32(PBA, pba);
        }

        /* flow control settings
         *
         * The high water mark must be low enough to fit one full frame
         * (or the size used for early receive) above it in the Rx FIFO.
         * Set it to the lower of:
         * - 90% of the Rx FIFO size, and
         * - the full Rx FIFO size minus one full frame
         */
        if self.params.flags.disable_fc_pause_time {
            self.fc.pause_time = 0xFFFF;
        } else {
            self.fc.pause_time = FC_PAUSE_TIME;
        }
        self.fc.send_xon = true;
        self.fc.current_mode = self.fc.requested_mode;

        let is_mtu_bg_than_data = self.params.mtu > ETH_DATA_LEN;

        match self.mac.kind {
            MacType::e1000_pchlan => {
                /* Workaround PCH LOM adapter hangs with certain network
                 * loads.  If hangs persist, try disabling Tx flow control.
                 */
                if is_mtu_bg_than_data {
                    self.fc.high_water = 0x3500;
                    self.fc.low_water = 0x1500;
                } else {
                    self.fc.high_water = 0x5000;
                    self.fc.low_water = 0x3000;
                }
                self.fc.refresh_time = 0x1000;
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
                self.fc.refresh_time = 0xFFFF;
                self.fc.pause_time = 0xFFFF;

                if is_mtu_bg_than_data {
                    let pba = 14;
                    self.ew32(PBA, pba);
                    self.fc.high_water = ((pba << 10) * 9 / 10) & FCRTH_RTH;
                    self.fc.low_water = ((pba << 10) * 8 / 10) & FCRTL_RTL;
                } else {
                    self.fc.high_water = 0x05C20;
                    self.fc.low_water = 0x05048;
                }
            }
            _ => {
                if let MacType::e1000_ich9lan | MacType::e1000_ich10lan = self.mac.kind {
                    if self.params.mtu > ETH_DATA_LEN {
                        let pba = 14;
                        self.ew32(PBA, pba);
                        self.fc.high_water = 0x2800;
                        self.fc.low_water = self.fc.high_water - 8;
                    }
                }

                let hwm = min(
                    ((pba << 10) * 9 / 10),
                    ((pba << 10) - self.params.max_frame_size),
                );
                self.fc.high_water = hwm & FCRTH_RTH; /* 8-byte granularity */
                self.fc.low_water = self.fc.high_water - 8;
            }
        }

        pba = self.er32(PBA);
        let mut limit = pba.wrapping_shr(16) << 10;
        limit = limit.wrapping_sub(96);
        limit = limit.min(24 << 10);

        self.params.tx_fifo_limit = limit;

        info!("pba: {}, fifo limit: {:#X}", pba, self.params.tx_fifo_limit);

        /* Disable Adaptive Interrupt Moderation if 2 full packets cannot
         * fit in receive buffer.
         */
        if self.params.itr_settings & 0x3 > 0 {
            let max_frame_size = self.params.max_frame_size * 2;
            let pba_value = pba << 10;

            if max_frame_size > pba_value {
                if !self.params.flags.disable_aim {
                    info!("Interrupt Throttle Rate off");
                    self.params.flags.disable_aim = true;
                }
            } else if self.params.flags.disable_aim {
                info!("Interrupt Throttle Rate on");
                self.params.flags.disable_aim = false;
                self.params.itr = 20000;
                self.write_itr(self.params.itr);
            }
        }
        if self.mac.kind >= MacType::e1000_pch_spt {
            self.flush_desc_ring();
        }

        self.ew32(WUC, 0);

        Ok(())
    }

    fn reset_hw(&self)->Result{
        match self.variant {
            
        }
    }
    /**
     * get_hw_control - get control of the h/w from f/w
     *
     * get_hw_control sets {CTRL_EXT|SWSM}:DRV_LOAD bit.
     * For ASF and Pass Through versions of f/w this means that
     * the driver is loaded. For AMT version (only with 82573)
     * of the f/w this means that the network i/f is open.
     **/
    fn get_hw_control(&mut self) {
        if self.params.flags.has_swsm_on_load {
            let swsm = self.er32(SWSM);
            self.ew32(SWSM, swsm | SWSM_DRV_LOAD);
        } else if self.params.flags.has_ctrlext_on_load {
            let ctrl_ext = self.er32(CTRL_EXT);
            self.ew32(CTRL_EXT, ctrl_ext | CTRL_EXT_DRV_LOAD);
        }
    }
    /**
     * release_hw_control - release control of the h/w to f/w
     *
     * release_hw_control resets {CTRL_EXT|SWSM}:DRV_LOAD bit.
     * For ASF and Pass Through versions of f/w this means that the
     * driver is no longer loaded. For AMT version (only with 82573) i
     * of the f/w this means that the network i/f is closed.
     *
     **/
    fn release_hw_control(&mut self) {
        if self.params.flags.has_swsm_on_load {
            let swsm = self.er32(SWSM);
            self.ew32(SWSM, swsm & !SWSM_DRV_LOAD);
        } else if self.params.flags.has_ctrlext_on_load {
            let ctrl_ext = self.er32(CTRL_EXT);
            self.ew32(CTRL_EXT, ctrl_ext & !CTRL_EXT_DRV_LOAD);
        }
    }

    /**
     * flush_tx_ring - remove all descriptors from the tx_ring
     *
     * We want to clear all pending descriptors from the TX ring.
     * zeroing happens when the HW reads the regs. We  assign the ring itself as
     * the data of the next descriptor. We don't care about the data we are about
     * to reset the HW.
     */
    fn flush_tx_ring(&mut self) {
        let txd_lower = TXD_CMD_IFCS;
        let size = 512;

        let tctl = self.er32(TCTL);
        self.ew32(TCTL, tctl | TCTL_EN);
        let tdt = self.er32(TDT(0));
        if tdt != self.tx_ring.next_to_use as u32 {
            panic!("tdt {} != next_to_use {}", tdt, self.tx_ring.next_to_use);
        }

        {
            let index = self.tx_ring.next_to_use;
            self.tx_ring[index].desc_mut().status = 0;
        }

        /* flush descriptors to memory before notifying the HW */
        fence();
        self.tx_ring.next_to_use += 1;
        if (self.tx_ring.next_to_use == self.tx_ring.len()) {
            self.tx_ring.next_to_use = 0;
        }
        self.ew32(TDT(0), self.tx_ring.next_to_use as u32);

        kfn().sleep(Duration::from_nanos(200));
    }

    /**
     * flush_tx_ring - remove all descriptors from the tx_ring
     *
     * We want to clear all pending descriptors from the TX ring.
     * zeroing happens when the HW reads the regs. We  assign the ring itself as
     * the data of the next descriptor. We don't care about the data we are about
     * to reset the HW.
     */
    fn flush_rx_ring(&mut self) {
        let rctl = self.er32(RCTL);

        self.ew32(RCTL, rctl & !RCTL_EN);

        self.e1e_flush();

        kfn().sleep(Duration::from_nanos(100));

        let mut rxdctl = self.er32(RXDCTL(0));
        /* zero the lower 14 bits (prefetch and host thresholds) */
        rxdctl &= 0xffffc000;

        /* update thresholds: prefetch threshold to 31, host threshold to 1
         * and make sure the granularity is "descriptors" and not "cache lines"
         */
        rxdctl |= (0x1F | (1 << 8) | RXDCTL_THRESH_UNIT_DESC);

        self.ew32(RXDCTL(0), rxdctl);

        /* momentarily enable the RX ring for the changes to take effect */
        self.ew32(RCTL, rctl | RCTL_EN);
        self.e1e_flush();

        kfn().sleep(Duration::from_nanos(100));

        self.ew32(RCTL, rctl & (!RCTL_EN));
    }

    /**
     * flush_desc_rings - remove all descriptors from the descriptor rings
     * @adapter: board private structure
     *
     * In i219, the descriptor rings must be emptied before resetting the HW
     * or before changing the device state to D3 during runtime (runtime PM).
     *
     * Failure to do this will cause the HW to enter a unit hang state which can
     * only be released by PCI reset on the device
     *
     */

    fn flush_desc_ring(&mut self) {
        /* First, disable MULR fix in FEXTNVM11 */
        let mut fext_nvm11 = self.er32(FEXTNVM11);
        fext_nvm11 |= FEXTNVM11_DISABLE_MULR_FIX;
        self.ew32(FEXTNVM11, fext_nvm11);
        /* do nothing if we're not in faulty state, or if the queue is empty */
        let tdlen = self.er32(TDLEN(0));
        let mut hang_state = kfn().pci_read_config_word(PCICFG_DESC_RING_STATUS);
        if ((hang_state & FLUSH_DESC_REQUIRED) == 0 || tdlen == 0) {
            return;
        }
        self.flush_tx_ring();

        /* recheck, maybe the fault is caused by the rx ring */
        hang_state = kfn().pci_read_config_word(PCICFG_DESC_RING_STATUS);

        if (hang_state & FLUSH_DESC_REQUIRED) > 0 {
            self.flush_rx_ring();
        }
    }

    /**
     * write_itr - write the ITR value to the appropriate registers
     * @itr: new ITR value to program
     *
     * write_itr determines if the adapter is in MSI-X mode
     * and, if so, writes the EITR registers with the ITR value.
     * Otherwise, it writes the ITR value into the ITR register.
     **/
    fn write_itr(&mut self, itr: u32) {
        let new_itr = if itr > 0 { 1000000000 / (itr * 256) } else { 0 };

        if self.params.int_mode == InterruptMode::MSIX {
            for vector in 0..3 {
                self.write_reg(EITR_82574(vector), new_itr);
            }
        } else {
            self.ew32(ITR, new_itr);
        }
    }

    fn write_reg(&self, offset: u32, value: u32) {
        unsafe {
            let ptr = (self.reg.mmio + offset as usize) as *mut u32;
            ptr.write_volatile(value);
        }
    }

    fn ew32(&self, offset: u32, value: u32) {
        self.reg.ew32(offset, value);
    }

    fn er32(&self, offset: u32) -> u32 {
        self.reg.er32(offset)
    }
    fn e1e_flush(&self) {
        self.reg.e1e_flush();
    }
    pub fn set_interrupt_mode(&mut self, mode: InterruptMode) {
        self.params.int_mode = mode;
    }

    pub fn get_interrupt_mode(&self) -> InterruptMode {
        self.params.int_mode.clone()
    }

    pub fn irq_disable(&mut self) {
        self.ew32(IMC, !0);
        if self.params.int_mode == InterruptMode::MSIX {
            self.ew32(EIAC_82574, 0);
        }
        self.e1e_flush();
    }

    pub fn rar_set(&self, index: usize, mac: MacAddress) -> Result {
        let res = if self.mac.kind >= MacType::e1000_pch_lpt {
            self.mac.rar_set_pch_lpt(mac, index, self.reg, &self.phy)
        } else {
            self.mac.rar_set_generic(mac, index, self.reg)
        };
        if res.is_err() {
            error!("Failed to write receive address {}", index);
        }
        res
    }

    pub fn print_info(&mut self) {
        let pba_str = [0; PBANUM_LENGTH];
        let mac_kind = self.mac.kind;
        let phy_kind = self.phy.kind;

        info!(
            "(PCI Express:2.5GT/s:{:?}) {}",
            self.bus.width,
            MacAddress(self.mac.addr)
        );

        info!(
            "Intel(R) PRO/{} Network Connection",
            if phy_kind == PhyType::Ife {
                "10/100"
            } else {
                "1000"
            }
        );
        // info!("MAC: {:?}, PHY: {:?}", mac_kind, phy_kind);
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
        let mut pba_num = [0; PBANUM_LENGTH];
        let mut nvm_data = self.nvm.read_nvm(NVM_PBA_OFFSET_0, 1)?[0];
        let mut pba_ptr = self.nvm.read_nvm(NVM_PBA_OFFSET_1, 1)?[0];

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

        let mut length: u16 = self.nvm.read_nvm(pba_ptr, 1)?[0];

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
            nvm_data = self.nvm.read_nvm(pba_ptr + offset, 1)?[0];
            pba_num[offset as usize * 2] = (nvm_data >> 8) as u8;
            pba_num[(offset as usize * 2) + 1] = (nvm_data & 0xFF) as u8;
        }
        pba_num[(length - 1) as usize * 2] = b'\0';

        Ok(pba_num)
    }

    pub fn xmit(&mut self, config: E1000XmitConfig, packet: &[u8]) -> Result<u32> {
        // let mut i = self.tx_ring.next_to_use;

        // // info!("transmit [{}] len {}", i, packet.len());
        // let n = {
        //     let n = self.tx_ring.one_copy_data_from(i, packet);

        //     let mut tx_desc = &mut self.tx_ring.as_mut()[i];
        //     tx_desc.lower = (TXD_CMD_EOP | TXD_CMD_IFCS | TXD_CMD_RS | (n as u32 & 0xffff)).to_le();
        //     tx_desc.status = 0;
        //     n
        // };
        // i += 1;
        // if i >= self.tx_ring.size {
        //     i = 0;
        // }
        // self.tx_ring.next_to_use = i;
        // self.ew32(TDT(0), i as u32);

        // Ok(n as u32)
        Ok(0)
    }

    pub fn clean_rx(&mut self, max: u32) -> Vec<RcvPkt> {
        let mut out = Vec::new();

        // let mut i = self.rx_ring.next_to_clean;
        // let mut pkts_compl = 0;
        // let mut bytes_compl = 0;
        // let mut tail = i;

        // for _ in 0..max {
        //     // info!(
        //     //     "clean_rx: max {}, index: {}, head {}",
        //     //     max,
        //     //     i,
        //     //     self.er32(RDH(0))
        //     // );

        //     let desc = self.rx_ring.mut_volatile(i);
        //     if (desc.status & RXD_STAT_DD as u8) == 0 {
        //         break;
        //     }
        //     self.rx_ring.as_mut()[i].status = 0;
        //     let len = desc.length as usize;
        //     let mbuf = unsafe { &*slice_from_raw_parts(self.rx_ring.mbufs[i] as *mut u8, len) };
        //     let mut data = Vec::try_with_capacity(len).unwrap();
        //     data.try_extend_from_slice(mbuf);
        //     out.try_push(RcvPkt { data }).unwrap();

        //     tail = i;
        //     // self.ew32(RDT(0), i as u32);
        //     i += 1;
        //     if i >= self.rx_ring.size {
        //         i = 0;
        //     }
        // }

        // if self.rx_ring.next_to_clean != i {
        //     self.rx_ring.next_to_clean = i;
        //     self.ew32(RDT(0), tail as u32);
        // }

        out
    }

    /// open
    pub fn open(&mut self) -> Result {
        Ok(())
    }

    fn clean_uc_addr(&mut self) {
        for i in 1..self.mac.rar_entry_count as u32 {
            self.ew32(RAL(i), 0);
            self.ew32(RAH(i), 0)
        }
    }

    fn update_mc_addr_list_generic(&mut self) {
        for i in 0..self.mac.mta_reg_count as u32 {
            self.reg.write_reg_array(MTA, i, 0);
        }
    }

    fn enable_receives(&mut self) {
        info!("enable rx");
        let rctl = self.er32(RCTL);
        self.ew32(RCTL, rctl | RCTL_EN);
    }
    fn enable_transmits(&mut self) {
        info!("enable tx");
        let mut tctl = self.er32(TCTL);
        tctl |= TCTL_EN;
        self.ew32(TCTL, tctl);
    }

    fn set_rx_mode(&mut self) {
        /* Check for Promiscuous and All Multicast modes */
        let mut rctl = self.er32(RCTL);

        /* clear the affected bits */
        rctl &= !(RCTL_UPE | RCTL_MPE);

        rctl |= RCTL_UPE | RCTL_LPE | RCTL_BAM | RCTL_SECRC | RCTL_BSEX;

        // rctl |= (
        //     RCTL_EN // enable receiver
        //     |  RCTL_BAM
        //     // enable broadcast
        //     //| RCTL_SZ_2048   // 2048-byte rx buffers
        //     //| RCTL_SECRC  // strip CRC
        // ) & !(0b11 << 10);

        self.ew32(RCTL, rctl);
        // TODO
    }

    fn init_manageability_pt(&mut self) {
        if !self.params.flags.mng_pt_enabled {
            return;
        }
        info!("Enabling management packets to the host");
        let mut manc = self.er32(MANC);

        /* enable receiving management packets to the host. this will probably
         * generate destination unreachable messages from the host OS, but
         * the packets will be handled on SMBUS
         */
        manc |= MANC_EN_MNG2HOST;
        let mut manc2h = self.er32(MANC2H);

        match self.mac.kind {
            // Default case handling
            _ => {
                manc2h |= (MANC2H_PORT_623 | MANC2H_PORT_664);
            }
            MacType::e1000_82574 | MacType::e1000_82583 => {
                let mut mdef: u32;
                let mut j: u32 = 0;

                // Check existing filters and enable if they match IPMI ports.
                for i in 0..8 {
                    mdef = self.er32(MDEF(i));
                    if mdef & !(MANC2H_PORT_623 | MANC2H_PORT_664) != 0 {
                        continue;
                    }
                    if mdef != 0 {
                        manc2h |= 1 << i;
                    }
                    j |= mdef;
                }

                if j != (MANC2H_PORT_623 | MANC2H_PORT_664) {
                    // Attempt to create a new filter if none matched.
                    for i in 0..8 {
                        if self.er32(MDEF(i)) == 0 {
                            self.ew32(MDEF(i), MANC2H_PORT_623 | MANC2H_PORT_664);
                            manc2h |= 1 << 1; // Enabling in MANC2H, assuming BIT(1) was a typo for enabling a different bit.
                            j += 1;
                            break;
                        }
                    }

                    if j == 0 {
                        warn!("Unable to create IPMI pass-through filter");
                    }
                }
            }
        }

        // Write the updated values back to the hardware registers.
        self.ew32(MANC2H, manc2h);
        self.ew32(MANC, manc);
    }

    fn ctrl_slu(&mut self) {
        info!("Setup CTRL SLU");
        let mut ctrl = self.er32(CTRL);
        ctrl |= CTRL_SLU;
        self.ew32(CTRL, ctrl);
        self.e1e_flush();
    }

    fn e1e_rphy(&mut self, offset: u32) -> Result<u16> {
        self.phy.e1e_rphy(offset)
    }

    fn e1e_wphy(&mut self, offset: u32, value: u16) -> Result {
        self.phy.e1e_wphy(offset, value)
    }

    pub fn get_stats64(&mut self) -> E1000HWStats {
        self.hw_stats.crcerrs += self.er32(CRCERRS) as u64;
        self.hw_stats.gprc += self.er32(GPRC) as u64;
        self.hw_stats.gorc += self.er32(GORCL) as u64;
        self.er32(GORCH); /* Clear gorc */
        self.hw_stats.bprc += self.er32(BPRC) as u64;
        self.hw_stats.mprc += self.er32(MPRC) as u64;
        self.hw_stats.roc += self.er32(ROC) as u64;

        self.hw_stats.mpc += self.er32(MPC) as u64;

        /* Half-duplex statistics */
        if (self.params.link_duplex == HALF_DUPLEX) {
            if (self.params.flags.has_phy_stats) {
                // e1000e_update_phy_stats(adapter);
            } else {
                self.hw_stats.scc += self.er32(SCC) as u64;
                self.hw_stats.ecol += self.er32(ECOL) as u64;
                self.hw_stats.mcc += self.er32(MCC) as u64;
                self.hw_stats.latecol += self.er32(LATECOL) as u64;
                self.hw_stats.dc += self.er32(DC) as u64;

                // hw->mac.collision_delta = self.er32(COLC);

                if ((self.mac.kind != MacType::e1000_82574)
                    && (self.mac.kind != MacType::e1000_82583))
                {
                    self.hw_stats.tncrs += self.er32(TNCRS) as u64;
                }
            }
            self.hw_stats.colc += self.mac.collision_delta as u64;
        }

        self.hw_stats.xonrxc += self.er32(XONRXC) as u64;
        self.hw_stats.xontxc += self.er32(XONTXC) as u64;
        self.hw_stats.xoffrxc += self.er32(XOFFRXC) as u64;
        self.hw_stats.xofftxc += self.er32(XOFFTXC) as u64;
        self.hw_stats.gptc += self.er32(GPTC) as u64;
        self.hw_stats.gotc += self.er32(GOTCL) as u64;
        self.er32(GOTCH); /* Clear gotc */
        self.hw_stats.rnbc += self.er32(RNBC) as u64;
        self.hw_stats.ruc += self.er32(RUC) as u64;

        self.hw_stats.mptc += self.er32(MPTC) as u64;
        self.hw_stats.bptc += self.er32(BPTC) as u64;

        /* used for adaptive IFS */

        self.mac.tx_packet_delta = self.er32(TPT);
        self.hw_stats.tpt += self.mac.tx_packet_delta as u64;

        self.hw_stats.algnerrc += self.er32(ALGNERRC) as u64;
        self.hw_stats.rxerrc += self.er32(RXERRC) as u64;
        self.hw_stats.cexterr += self.er32(CEXTERR) as u64;
        self.hw_stats.tsctc += self.er32(TSCTC) as u64;
        self.hw_stats.tsctfc += self.er32(TSCTFC) as u64;

        self.hw_stats.clone()
    }

    /**
     *  e1000e_check_downshift - Checks whether a downshift in speed occurred
     *  @hw: pointer to the HW structure
     *
     *  Success returns 0, Failure returns 1
     *
     *  A downshift is detected by querying the PHY link health.
     **/
    fn check_downshift(&mut self) -> Result {
        let mut offset = 0;
        let mut mask = 0u16;

        match self.phy.kind {
            PhyType::M88 | PhyType::Gg82563 | PhyType::Bm | PhyType::T82578 => {
                offset = M88E1000_PHY_SPEC_STATUS;
                mask = M88E1000_PSSR_DOWNSHIFT;
            }
            PhyType::Igp2 | PhyType::Igp3 => {
                offset = IGP01E1000_PHY_LINK_HEALTH;
                mask = IGP01E1000_PLHR_SS_DOWNGRADE;
            }
            _ => {
                self.phy.speed_downgraded = false;
                return Ok(());
            }
        }

        let phy_data = self.e1e_rphy(offset)?;

        self.phy.speed_downgraded = (phy_data & mask) != 0;

        return Ok(());
    }

    /**
     *  e1000e_update_adaptive - Update Adaptive Interframe Spacing
     *  @hw: pointer to the HW structure
     *
     *  Update the Adaptive Interframe Spacing Throttle value based on the
     *  time between transmitted packets and time between collisions.
     **/
    fn update_adaptive(&mut self) {
        if (!self.mac.adaptive_ifs) {
            error!("Not in Adaptive IFS mode!\n");
            return;
        }

        if ((self.mac.collision_delta * self.mac.ifs_ratio as u32) > self.mac.tx_packet_delta) {
            if (self.mac.tx_packet_delta > MIN_NUM_XMITS) {
                self.mac.in_ifs_mode = true;
                if (self.mac.current_ifs_val < self.mac.ifs_max_val) {
                    if self.mac.current_ifs_val == 0 {
                        self.mac.current_ifs_val = self.mac.ifs_min_val;
                    } else {
                        self.mac.current_ifs_val += self.mac.ifs_step_size;
                    }
                    self.ew32(AIT, self.mac.current_ifs_val as u32);
                }
            }
        } else {
            if (self.mac.in_ifs_mode && (self.mac.tx_packet_delta <= MIN_NUM_XMITS)) {
                self.mac.current_ifs_val = 0;
                self.mac.in_ifs_mode = false;
                self.ew32(AIT, 0);
            }
        }
    }

    pub fn task_watchdog(&mut self) -> Result<bool> {
        Ok(false)
    }
}

impl Drop for E1000 {
    fn drop(&mut self) {
        // debug!("unmap mmio {:#X}", self.hw.mmio);
        // self.kernel().iounmap(self.hw.mmio);
        // if let Some(flash) = self.flash {
        //     debug!("unmap flash {:#X}", flash);
        //     self.kernel().iounmap(flash);
        // }
    }
}

fn align_to(value: u32, alignment: u32) -> u32 {
    let remainder = value % alignment;
    if remainder == 0 {
        value
    } else {
        value + (alignment - remainder)
    }
}

pub struct RcvPkt {
    pub data: Vec<u8>,
}
