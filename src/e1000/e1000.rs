use super::defines::*;
use super::hw::{self, DevSpec, Hw, HwRc, MacType};
use super::regs::*;
use super::ring::*;
use crate::err::{Error, Result};
use crate::phy::*;
use crate::*;
use core::alloc::Layout;
use core::borrow::Borrow;
use core::cmp::min;
use core::iter::Scan;
use core::mem::{self, align_of_val};
use core::ptr::NonNull;
use core::u32;
use e1000::board;
use ffi::CStr;
use hw_semaphore::HwSemaphore;

const TX_RING_SIZE: usize = 256;
const RX_RING_SIZE: usize = 256;

pub struct E1000<K>
where
    K: KernelFunc,
{
    pub(crate) eiac_mask: u32,
    pub(crate) tx_ring: Ring<TxDesc, K>,
    pub(crate) rx_ring: Ring<RxDesc, K>,
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

        let tx_ring = Ring::new(kfun.clone(), TX_RING_SIZE);
        let rx_ring = Ring::new(kfun.clone(), RX_RING_SIZE);

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
        self.hw.as_ref().rar_set(MacAddress(mac), index);
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

    pub fn transmit(&mut self, packet: &[u8]) -> Result<u32> {
        info!("transmit {}", packet.len());

        let tdh = self.er32(TDH(0));
        let mut i = self.er32(TDT(0)) as usize;

        info!("Read TDH 1= {:#x}\n", tdh);
        info!("Read TDT 1= {:#x}\n", i);

        let mut offset = 0;

        let mut count = 0;

        let mut src = packet;

        loop {
            let n = self.tx_ring.one_copy_data_from(i, src);
            let mut tx_desc = &mut self.tx_ring.as_mut()[i];
            tx_desc.length = n as u16;
            tx_desc.cmd = (TXD_CMD_RS | TXD_CMD_EOP | TXD_CMD_IFCS) as u8;
            tx_desc.status = 0;

            count += 1;
            i += 1;

            if n >= src.len() {
                break;
            }
            src = &src[n..];
        }

        self.ew32(TDT(0), (i % TX_RING_SIZE) as u32);

        // sync
        fence_w();
        info!("link speed: 0x{:08x}", self.er32(STATUS));
        info!("Read E1000_TDH 2= {:#x}\n", self.er32(TDH(0)));
        info!("Read E1000_TDT 2= {:#x}\n", self.er32(TDT(0)));

        Ok(packet.len() as u32)
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
        self.get_speed_and_duplex_copper();
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
        // if hw.status.itr_setting & 0x3 > 0 {
        //     let max_frame_size = hw.status.max_frame_size * 2;
        //     let pba_value = pba << 10;

        //     if max_frame_size > pba_value {
        //         if !(hw.flags2.disable_aim) {
        //             info!("Interrupt Throttle Rate off");
        //             hw.flags2.disable_aim = true;
        //             self.e1000e_write_itr(0);
        //         }
        //     } else if hw.flags2.disable_aim {
        //         info!("Interrupt Throttle Rate on");
        //         hw.flags2.disable_aim = false;
        //         hw.status.itr = 20000;
        //         self.e1000e_write_itr(hw.status.itr);
        //     }
        // }
        if self.hw.as_ref().mac.kind >= MacType::e1000_pch_spt {
            self.flush_desc_rings();
        }

        self.dev_spec.reset_hw(hw)?;

        if hw.flags.has_amt {
            self.get_hw_control();
        }

        self.ew32(WUC, 0);

        self.dev_spec.init_hw(hw)?;

        Ok(())
    }

    pub fn trigger_lsc(&self) {
        if self.hw.as_ref().status.msix_vec_num > 0 {
            self.ew32(ICS, ICS_LSC | ICS_OTHER);
        } else {
            self.ew32(ICS, ICS_LSC);
        }
    }

    /**
     * e1000e_get_hw_control - get control of the h/w from f/w
     * @adapter: address of board private structure
     *
     * e1000e_get_hw_control sets {CTRL_EXT|SWSM}:DRV_LOAD bit.
     * For ASF and Pass Through versions of f/w this means that
     * the driver is loaded. For AMT version (only with 82573)
     * of the f/w this means that the network i/f is open.
     **/
    fn get_hw_control(&mut self) {
        if self.hw.as_mut().flags.has_swsm_on_load {
            let swsm = self.er32(SWSM);
            self.ew32(SWSM, swsm | SWSM_DRV_LOAD);
        } else if self.hw.as_mut().flags.has_ctrlext_on_load {
            let ctrl_ext = self.er32(CTRL_EXT);
            self.ew32(CTRL_EXT, ctrl_ext | CTRL_EXT_DRV_LOAD);
        }
    }
    /**
     * e1000e_release_hw_control - release control of the h/w to f/w
     * @adapter: address of board private structure
     *
     * e1000e_release_hw_control resets {CTRL_EXT|SWSM}:DRV_LOAD bit.
     * For ASF and Pass Through versions of f/w this means that the
     * driver is no longer loaded. For AMT version (only with 82573) i
     * of the f/w this means that the network i/f is closed.
     *
     **/
    fn release_hw_control(&mut self) {
        if self.hw.as_mut().flags.has_swsm_on_load {
            let swsm = self.er32(SWSM);
            self.ew32(SWSM, swsm & !SWSM_DRV_LOAD);
        } else if self.hw.as_mut().flags.has_ctrlext_on_load {
            let ctrl_ext = self.er32(CTRL_EXT);
            self.ew32(CTRL_EXT, ctrl_ext & !CTRL_EXT_DRV_LOAD);
        }
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
    fn write_itr(&mut self, itr: u32) {
        let hw = self.hw.as_mut();
        let new_itr = if itr > 0 { 1000000000 / (itr * 256) } else { 0 };
        let msix_vec_num = hw.status.msix_vec_num;

        if msix_vec_num > 0 {
            for vector in 0..msix_vec_num {
                unsafe {
                    let ptr = (hw.mmio + EITR_82574(vector as usize)) as *mut u32;
                    ptr.write_volatile(new_itr);
                }
            }
        } else {
            hw.ew32(ITR, new_itr);
        }
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

    fn flush_desc_rings(&mut self) {
        /* First, disable MULR fix in FEXTNVM11 */
        let mut fext_nvm11 = self.er32(FEXTNVM11);
        fext_nvm11 |= FEXTNVM11_DISABLE_MULR_FIX;
        self.ew32(FEXTNVM11, fext_nvm11);
        /* do nothing if we're not in faulty state, or if the queue is empty */
        let tdlen = self.er32(TDLEN(0));
        let mut hang_state = self.kernel().pci_read_config_word(PCICFG_DESC_RING_STATUS);
        if ((hang_state & FLUSH_DESC_REQUIRED) == 0 || tdlen == 0) {
            return;
        }
        self.flush_tx_ring();

        /* recheck, maybe the fault is caused by the rx ring */
        hang_state = self.kernel().pci_read_config_word(PCICFG_DESC_RING_STATUS);

        if (hang_state & FLUSH_DESC_REQUIRED) > 0 {
            self.flush_rx_ring();
        }
    }
    /**
     * flush_tx_ring - remove all descriptors from the tx_ring
     * @adapter: board private structure
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
        let dma = self.tx_ring.dma.to_le();
        {
            // BUG_ON(tdt != tx_ring->next_to_use);
            let tx_desc = self.tx_ring.next_to_use();
            tx_desc.addr = dma;

            // tx_desc.data = (txd_lower | size).to_le();
            // tx_desc.data = 0;
        }
        /* flush descriptors to memory before notifying the HW */
        fence();
        self.tx_ring.next_to_use += 1;
        if (self.tx_ring.next_to_use == self.tx_ring.size) {
            self.tx_ring.next_to_use = 0;
        }
        self.ew32(TDT(0), self.tx_ring.next_to_use as u32);
        self.kernel().delay(200_000);
    }
    /**
     * flush_tx_ring - remove all descriptors from the tx_ring
     * @adapter: board private structure
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

        self.kernel().delay(100_000);

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
        self.kernel().delay(100_000);
        self.ew32(RCTL, rctl & (!RCTL_EN));
    }

    /// open
    pub fn open(&mut self) -> Result {
        info!("e1000 begin open");
        self.tx_ring.init();
        self.rx_ring.init();
        info!("ring ok");

        self.power_up_phy()?;
        self.configure()?;

        info!("e1000_init has been completed");
        Ok(())
    }

    fn configure(&mut self) -> Result {
        self.set_rx_mode();
        self.init_manageability_pt();
        self.configure_tx();
        self.setup_rctl();
        self.configure_rx();
        self.setup_tctl();

        Ok(())
    }

    fn get_hw_semaphore(&mut self) -> Result<HwSemaphore<K>> {
        HwSemaphore::new(self.hw.clone(), &mut self.dev_spec)
    }

    // fn read_phy_reg_bm2(&self, offset: u32) -> Result<u16> {
    //     let sp = self.get_hw_semaphore()?;
    //     let mut page = (offset >> IGP_PAGE_SHIFT) as u16;
    //     self.hw.as_mut().phy.addr = 1;

    //     drop(sp);
    //     Ok(())
    // }

    fn set_rx_mode(&mut self) {
        /* Check for Promiscuous and All Multicast modes */
        let mut rctl = self.er32(RCTL);

        /* clear the affected bits */
        rctl &= !(RCTL_UPE | RCTL_MPE);

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
        if !self.hw.as_ref().flags.mng_pt_enabled {
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

        match self.hw.as_ref().mac.kind {
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

    fn power_up_phy(&mut self) -> Result {
        let mut mii = self.read_phy_reg_mdic(MII_BMCR)?;
        mii &= !BMCR_PDOWN;
        self.write_phy_reg_mdic(MII_BMCR, mii)?;
        info!("power up phy ok");
        Ok(())
    }

    fn configure_tx(&mut self) {
        let mut tdba = self.tx_ring.dma;
        let tdlen = self.tx_ring.data_len() as u32;
        self.ew32(TDBAL(0), (tdba & DMA_BIT_MASK(32)) as u32);
        self.ew32(TDBAH(0), (tdba >> 32) as u32);
        self.ew32(TDLEN(0), tdlen);
        self.ew32(TDH(0), 0);
        self.ew32(TDT(0), 0);

        /* Set the Tx Interrupt Delay register */
        self.ew32(TIDV, self.hw.as_ref().status.tx_int_delay);
        /* Tx irq moderation */
        self.ew32(TADV, self.hw.as_ref().status.tx_abs_int_delay);

        if self.hw.as_ref().flags2.dma_burst {
            let mut txdctl = self.er32(TXDCTL(0));

            txdctl &= !(TXDCTL_PTHRESH | TXDCTL_HTHRESH | TXDCTL_WTHRESH);
            /* set up some performance related parameters to encourage the
             * hardware to use the bus more efficiently in bursts, depends
             * on the tx_int_delay to be enabled,
             * wthresh = 1 ==> burst write is disabled to avoid Tx stalls
             * hthresh = 1 ==> prefetch when one or more available
             * pthresh = 0x1f ==> prefetch if internal cache 31 or less
             * BEWARE: this seems to work but should be considered first if
             * there are Tx hangs or other Tx related bugs
             */
            txdctl |= TXDCTL_DMA_BURST_ENABLE;
            self.ew32(TXDCTL(0), txdctl);
        }
        /* erratum work around: set txdctl the same for both queues */
        self.ew32(TXDCTL(1), self.er32(TXDCTL(0)));

        /* Program the Transmit Control Register */
        let mut tctl = self.er32(TCTL);
        tctl &= !TCTL_CT;
        tctl |= TCTL_PSP | TCTL_RTLC | (COLLISION_THRESHOLD << CT_SHIFT);

        if self.hw.as_ref().flags.tarc_speed_mode_bit {
            let mut tarc = self.er32(TARC(0));
            /* set the speed mode bit, we'll clear it if we're not at
             * gigabit link later
             */
            tarc |= BIT(21);
            self.ew32(TARC(0), tarc);
        }

        /* errata: program both queues to unweighted RR */
        if (self.hw.as_ref().flags.tarc_set_bit_zero) {
            let mut tarc = self.er32(TARC(0));
            tarc |= 1;
            self.ew32(TARC(0), tarc);
            tarc = self.er32(TARC(1));
            tarc |= 1;
            self.ew32(TARC(1), tarc);
        }

        self.ew32(TCTL, tctl);

        /* SPT and KBL Si errata workaround to avoid data corruption */
        if self.hw.as_ref().mac.kind == MacType::e1000_pch_spt {
            let mut reg_val = self.er32(IOSFPC);
            reg_val |= RCTL_RDMTS_HEX;
            self.ew32(IOSFPC, reg_val);

            reg_val = self.er32(TARC(0));
            /* SPT and KBL Si errata workaround to avoid Tx hang.
             * Dropping the number of outstanding requests from
             * 3 to 2 in order to avoid a buffer overrun.
             */
            reg_val &= !TARC0_CB_MULTIQ_3_REQ;
            reg_val |= TARC0_CB_MULTIQ_2_REQ;
            self.ew32(TARC(0), reg_val);
        }
    }

    fn configure_rx(&mut self) {
        //TODO

        let rdlen = self.rx_ring.data_len();
        let mut hw2 = self.hw.clone();
        let hw = hw2.as_mut();
        /* disable receives while setting up the descriptors */
        let mut rctl = self.er32(RCTL);
        if !hw.flags2.no_disable_rx {
            self.ew32(RCTL, rctl & !RCTL_EN);
        }
        self.e1e_flush();
        self.kernel().delay(10000_000);

        if hw.flags2.dma_burst {
            /* set the writeback threshold (only takes effect if the RDTR
             * is set). set GRAN=1 and write back up to 0x4 worth, and
             * enable prefetching of 0x20 Rx descriptors
             * granularity = 01
             * wthresh = 04,
             * hthresh = 04,
             * pthresh = 0x20
             */
            self.ew32(RXDCTL(0), RXDCTL_DMA_BURST_ENABLE);
            self.ew32(RXDCTL(1), RXDCTL_DMA_BURST_ENABLE);
        }

        /* set the Receive Delay Timer Register */
        self.ew32(RDTR, hw.status.rx_int_delay);

        /* irq moderation */
        self.ew32(RADV, hw.status.rx_abs_int_delay);
        if (hw.status.itr_setting != 0) && (hw.status.itr != 0) {
            self.write_itr(hw.status.itr);
        }

        let mut ctrl_ext = self.er32(CTRL_EXT);
        /* Auto-Mask interrupts upon ICR access */
        ctrl_ext |= CTRL_EXT_IAME;
        self.ew32(IAM, 0xffffffff);
        self.ew32(CTRL_EXT, ctrl_ext);
        self.e1e_flush();

        /* Setup the HW Rx Head and Tail Descriptor Pointers and
         * the Base and Length of the Rx Descriptor Ring
         */
        let mut rdba = self.rx_ring.dma;
        self.ew32(RDBAL(0), (rdba & DMA_BIT_MASK(32)) as u32);
        self.ew32(RDBAH(0), (rdba >> 32) as u32);
        self.ew32(RDLEN(0), rdlen as u32);
        self.ew32(RDH(0), 0);
        self.ew32(RDT(0), 0);
        // rx_ring->head = adapter->hw.hw_addr + E1000_RDH(0);
        // rx_ring->tail = adapter->hw.hw_addr + E1000_RDT(0);

        // writel(0, rx_ring->head);
        // if (adapter->flags2 & FLAG2_PCIM2PCI_ARBITER_WA)
        // 	e1000e_update_rdt_wa(rx_ring, 0);
        // else
        // 	writel(0, rx_ring->tail);

        /* Enable Receive Checksum Offload for TCP and UDP */
        // rxcsum = self.er32(RXCSUM);
        // if (adapter->netdev->features & NETIF_F_RXCSUM)
        // 	rxcsum |= E1000_RXCSUM_TUOFL;
        // else
        // 	rxcsum &= ~E1000_RXCSUM_TUOFL;
        // self.ew32(RXCSUM, rxcsum);
    }
    fn setup_tctl(&mut self) {
        let mut hw2 = self.hw.clone();
        let hw = hw2.as_mut();
        // transmitter control bits.
        self.ew32(
            TCTL,
            TCTL_EN |  // enable
            TCTL_PSP |  // pad short packets
            TCTL_RTLC |
            (0x0f << TCTL_CT_SHIFT) & // collision stuff
            !(TCTL_COLD) |
            (0x3f << TCTL_COLD_SHIFT),
        );
    }
    fn setup_rctl(&mut self) {
        let mut hw2 = self.hw.clone();
        let hw = hw2.as_mut();

        /* Program MC offset vector base */
        let mut rctl = self.er32(RCTL);
        rctl &= !(3 << RCTL_MO_SHIFT);
        rctl |= RCTL_EN
            | RCTL_BAM
            | RCTL_LBM_NO
            | RCTL_RDMTS_HALF
            | (hw.mac.mc_filter_type << RCTL_MO_SHIFT);

        /* Do not Store bad packets */
        rctl &= !RCTL_SBP;

        /* Enable Long Packet receive */
        if self.kernel().is_mtu_bg_eth() {
            rctl |= RCTL_LPE;
        } else {
            rctl &= !RCTL_LPE;
        }

        /* Some systems expect that the CRC is included in SMBUS traffic. The
         * hardware strips the CRC before sending to both SMBUS (BMC) and to
         * host memory when this is enabled
         */
        if (hw.flags2.crc_stripping) {
            rctl |= RCTL_SECRC;
        }

        /* Setup buffer sizes */
        rctl &= !RCTL_SZ_4096;
        rctl |= RCTL_BSEX;

        match hw.status.rx_buffer_len {
            2048 | _ => {
                // For 2048 or any other default/fallback case, disable BSEX and set 2048 size
                rctl |= RCTL_SZ_2048;
                rctl &= !RCTL_BSEX;
            }
            4096 => {
                rctl |= RCTL_SZ_4096;
            }
            8192 => {
                rctl |= RCTL_SZ_8192;
            }
            16384 => {
                rctl |= RCTL_SZ_16384;
            }
        }

        // Read RFCTL register, enable extended status, and write back
        let mut rfctl = self.er32(RFCTL);
        rfctl |= RFCTL_EXTEN;
        self.ew32(RFCTL, rfctl);

        hw.flags.restart_now = false;
    }

    fn read_phy_reg_mdic(&mut self, offset: u32) -> Result<u16> {
        self.hw.as_mut().phy.addr = 1;

        if (offset > MAX_PHY_REG_ADDRESS) {
            error!("PHY Address {} is out of range", offset);
            return Err(Error::Unknown("PHY Address"));
        }
        /* Set up Op-code, Phy Address, and register offset in the MDI
         * Control register.  The MAC will take care of interfacing with the
         * PHY to retrieve the desired data.
         */
        let mut mdic = ((offset << MDIC_REG_SHIFT)
            | (self.hw.as_mut().phy.addr << MDIC_PHY_SHIFT)
            | (MDIC_OP_READ));

        self.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT {
            self.kernel().delay(50_000);
            mdic = self.er32(MDIC);
            if (mdic & MDIC_READY) > 0 {
                break;
            }
        }

        if (mdic & MDIC_READY) == 0 {
            error!("MDI Read PHY Reg Address {} did not complete", offset);
            return Err(Error::Unknown("PHY"));
        }
        if (mdic & MDIC_ERROR) > 0 {
            error!("MDI Read PHY Reg Address {} Error", offset);
            return Err(Error::Unknown("PHY"));
        }
        if (((mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT) != offset) {
            error!(
                "MDI Read offset error - requested {}, returned {}",
                offset,
                (mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT,
            );

            return Err(Error::Unknown("PHY"));
        }

        /* Allow some time after each MDIC transaction to avoid
         * reading duplicate data in the next MDIC transaction.
         */
        if (self.hw.as_ref().mac.kind == MacType::e1000_pch2lan) {
            self.kernel().delay(100_000);
        }

        Ok(mdic as u16)
    }
    /**
     * e1000_get_speed_and_duplex
     * @hw: Struct containing variables accessed by shared code
     * @speed: Speed of the connection
     * @duplex: Duplex setting of the connection
     *
     * Detects the current speed and duplex settings of the hardware.
     */
    fn get_speed_and_duplex_copper(&mut self) {
        let mut speed = 0;
        let mut duplex = 0;
        let status = self.er32(STATUS);
        if (status & STATUS_SPEED_1000) > 0 {
            speed = 1000;
        } else if (status & STATUS_SPEED_100) > 0 {
            speed = 100;
        } else {
            speed = 10;
        }

        if (status & STATUS_FD) > 0 {
            duplex = FULL_DUPLEX;
        } else {
            duplex = HALF_DUPLEX;
        }

        let status = &mut self.hw.as_mut().status;
        status.link_speed = speed;
        status.link_duplex = duplex;

        info!(
            "{} Mbps, {} Duplex\n",
            speed,
            if duplex == FULL_DUPLEX {
                "Full"
            } else {
                "Half"
            }
        );
    }

    // fn check_for_copper_link(&mut self) -> Result {
    //     if self.hw.as_ref().mac.get_link_status {
    //         return Ok(());
    //     }

    //     self.hw.as_mut().mac.get_link_status = false;

    //     let link = self.phy_has_link_generic(1, 0)?;
    //     if !link {
    //         return Ok(());
    //     }

    //     self.hw.as_mut().mac.get_link_status = true;
    // }

    // fn phy_has_link_generic(&mut self, iterations: u32, usec_interval: u32) -> Result<bool> {
    //     let mut success = false;

    //     for i in 0..iterations {
    //         if self.e1e_rphy(MII_BMSR).is_err() {
    //             self.kernel().delay(usec_interval * 1000);
    //         }

    //         let phy_status = self.e1e_rphy(MII_BMSR)?;
    //         if phy_status & BMSR_LSTATUS != 0 {
    //             success = true;
    //             break;
    //         }

    //         self.kernel().delay(usec_interval * 1000);
    //     }

    //     success
    // }

    fn e1e_rphy(&mut self, offset: u32) -> Result<u16> {
        self.read_phy_reg_mdic(offset)
    }

    pub fn get_stats64(&mut self) -> E1000HWStats {
        let mut hw = self.hw.clone();
        let stats = &mut hw.as_mut().status.stats;
        let mut hw = self.hw.clone();
        let hw = hw.as_mut();

        stats.crcerrs += self.er32(CRCERRS) as u64;
        stats.gprc += self.er32(GPRC) as u64;
        stats.gorc += self.er32(GORCL) as u64;
        self.er32(GORCH); /* Clear gorc */
        stats.bprc += self.er32(BPRC) as u64;
        stats.mprc += self.er32(MPRC) as u64;
        stats.roc += self.er32(ROC) as u64;

        stats.mpc += self.er32(MPC) as u64;

        /* Half-duplex statistics */
        if (self.hw.as_ref().status.link_duplex == HALF_DUPLEX) {
            if (self.hw.as_ref().flags2.has_phy_stats) {
                // e1000e_update_phy_stats(adapter);
            } else {
                stats.scc += self.er32(SCC) as u64;
                stats.ecol += self.er32(ECOL) as u64;
                stats.mcc += self.er32(MCC) as u64;
                stats.latecol += self.er32(LATECOL) as u64;
                stats.dc += self.er32(DC) as u64;

                // hw->mac.collision_delta = self.er32(COLC);

                if ((hw.mac.kind != MacType::e1000_82574) && (hw.mac.kind != MacType::e1000_82583))
                {
                    stats.tncrs += self.er32(TNCRS) as u64;
                }
            }
            stats.colc += hw.mac.collision_delta as u64;
        }

        stats.xonrxc += self.er32(XONRXC) as u64;
        stats.xontxc += self.er32(XONTXC) as u64;
        stats.xoffrxc += self.er32(XOFFRXC) as u64;
        stats.xofftxc += self.er32(XOFFTXC) as u64;
        stats.gptc += self.er32(GPTC) as u64;
        stats.gotc += self.er32(GOTCL) as u64;
        self.er32(GOTCH); /* Clear gotc */
        stats.rnbc += self.er32(RNBC) as u64;
        stats.ruc += self.er32(RUC) as u64;

        stats.mptc += self.er32(MPTC) as u64;
        stats.bptc += self.er32(BPTC) as u64;

        /* used for adaptive IFS */

        hw.mac.tx_packet_delta = self.er32(TPT);
        stats.tpt += hw.mac.tx_packet_delta as u64;

        stats.algnerrc += self.er32(ALGNERRC) as u64;
        stats.rxerrc += self.er32(RXERRC) as u64;
        stats.cexterr += self.er32(CEXTERR) as u64;
        stats.tsctc += self.er32(TSCTC) as u64;
        stats.tsctfc += self.er32(TSCTFC) as u64;

        stats.clone()
    }

    fn write_phy_reg_mdic(&mut self, offset: u32, value: u16) -> Result {
        self.hw.as_mut().phy.addr = 1;

        if (offset > MAX_PHY_REG_ADDRESS) {
            error!("PHY Address {} is out of range", offset);
            return Err(Error::Unknown("PHY Address"));
        }
        /* Set up Op-code, Phy Address, and register offset in the MDI
         * Control register.  The MAC will take care of interfacing with the
         * PHY to retrieve the desired data.
         */
        let mut mdic = ((offset << MDIC_REG_SHIFT)
            | (self.hw.as_mut().phy.addr << MDIC_PHY_SHIFT)
            | (MDIC_OP_WRITE));

        self.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT {
            self.kernel().delay(50_000);
            mdic = self.er32(MDIC);
            if (mdic & MDIC_READY) > 0 {
                break;
            }
        }

        if (mdic & MDIC_READY) == 0 {
            error!("MDI Read PHY Reg Address {} did not complete", offset);
            return Err(Error::Unknown("PHY"));
        }
        if (mdic & MDIC_ERROR) > 0 {
            error!("MDI Read PHY Reg Address {} Error", offset);
            return Err(Error::Unknown("PHY"));
        }
        if (((mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT) != offset) {
            error!(
                "MDI Read offset error - requested {}, returned {}",
                offset,
                (mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT,
            );

            return Err(Error::Unknown("PHY"));
        }

        /* Allow some time after each MDIC transaction to avoid
         * reading duplicate data in the next MDIC transaction.
         */
        if (self.hw.as_ref().mac.kind == MacType::e1000_pch2lan) {
            self.kernel().delay(100_000);
        }

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
    pub msix_vec_num: u32,
    pub tx_it: u16,
    pub rx_itr: u16,

    pub tx_int_delay: u32,
    pub tx_abs_int_delay: u32,

    pub rx_int_delay: u32,
    pub rx_abs_int_delay: u32,
    pub rx_buffer_len: u32,

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
