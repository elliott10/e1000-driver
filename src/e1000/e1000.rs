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
    fn e1000e_write_itr(&mut self, itr: u32) {
        let hw = self.hw.as_mut();
        let new_itr = if itr > 0 { 1000000000 / (itr * 256) } else { 0 };
        hw.ew32(ITR, new_itr);
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

            tx_desc.lower.data = (txd_lower | size).to_le();
            tx_desc.upper.data = 0;
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
        Ok(())
    }

    fn configure(&mut self) -> Result {
        let stat = self.hw.as_ref().er32(STATUS);

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

    fn power_up_phy(&mut self) -> Result {
        let mut mii = self.read_phy_reg_mdic(MII_BMCR)?;
        mii &= !BMCR_PDOWN;
        self.write_phy_reg_mdic(MII_BMCR, mii)?;
        info!("power up phy ok");
        Ok(())
    }

    fn read_phy_reg_mdic(&mut self, offset: u32) -> Result<u16> {
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

    fn write_phy_reg_mdic(&mut self, offset: u32, value: u16) -> Result {
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
