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
    pub(crate) reg: Reg,
    pub(crate) params: Params,
    pub(crate) mac: MacInfo,
    pub(crate) phy: PhyInfo2,
    pub(crate) nvm: NvmInfo,
    pub(crate) bus: BusInfo,
    pub(crate) fc: FcInfo,
    pub(crate) eiac_mask: u32,
    pub(crate) mng_cookie: HostMngDhcpCookie,
    hw_contrl: Option<HwControl>,
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
            hw_contrl: None,
            hw_stats: Default::default(),
            mng_cookie: Default::default(),
        };
        e1000.irq_disable();

        unsafe { CARD_FOUND = card_found };

        e1000.reset_hw()?;

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

        if e1000.params.flags.has_amt {
            e1000.hw_contrl = Some(HwControl::new(reg, e1000.flags()));
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
    /// open
    pub fn open(&mut self) -> Result {
        if self.params.flags.has_amt {
            self.hw_contrl = Some(HwControl::new(self.reg, self.flags()));
            self.reset()?;
        }

        self.power_up_phy()?;

        self.set_rx_mode();
        self.init_manageability_pt();
        self.configure_tx();
        self.setup_rctl();
        self.configure_rx();

        Ok(())
    }

    fn power_up_phy(&mut self) -> Result {
        info!("Power up phy");
        let mut mii = self.e1e_rphy(MII_BMCR)?;

        mii &= !BMCR_PDOWN;

        self.e1e_wphy(MII_BMCR, mii)?;

        self.setup_link();
        Ok(())
    }

    fn set_rx_mode(&mut self) {
        /* Check for Promiscuous and All Multicast modes */
        let mut rctl = self.er32(RCTL);

        /* clear the affected bits */
        rctl &= !(RCTL_UPE | RCTL_MPE);

        rctl |= RCTL_UPE | RCTL_LPE | RCTL_BAM | RCTL_SECRC | RCTL_BSEX;

        self.ew32(RCTL, rctl);
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

    fn configure_tx(&mut self) {
        let mut tdba = self.tx_ring.dma_desc.dma_addr();
        let tdlen = self.tx_ring.len() as u32;
        self.ew32(TDBAL(0), (tdba & DMA_BIT_MASK(32)) as u32);
        self.ew32(TDBAH(0), (tdba >> 32) as u32);
        self.ew32(TDLEN(0), tdlen);
        self.ew32(TDH(0), 0);
        self.ew32(TDT(0), 0);

        /* Set the Tx Interrupt Delay register */
        self.ew32(TIDV, self.params.tx_int_delay);
        /* Tx irq moderation */
        self.ew32(TADV, self.params.tx_abs_int_delay);

        if self.params.flags.dma_burst {
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

        if self.params.flags.tarc_speed_mode_bit {
            let mut tarc = self.er32(TARC(0));
            /* set the speed mode bit, we'll clear it if we're not at
             * gigabit link later
             */
            tarc |= BIT(21);
            self.ew32(TARC(0), tarc);
        }

        /* errata: program both queues to unweighted RR */
        if (self.params.flags.tarc_set_bit_zero) {
            let mut tarc = self.er32(TARC(0));
            tarc |= 1;
            self.ew32(TARC(0), tarc);
            tarc = self.er32(TARC(1));
            tarc |= 1;
            self.ew32(TARC(1), tarc);
        }

        self.ew32(TCTL, tctl);

        self.mac.config_collision_dist_generic();

        /* SPT and KBL Si errata workaround to avoid data corruption */
        if self.mac.kind == MacType::e1000_pch_spt {
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
    fn setup_rctl(&mut self) {
        /* Workaround Si errata on PCHx - configure jumbo frame flow.
         * If jumbo frames not set, program related MAC/PHY registers
         * to h/w defaults
         */
        if (self.mac.kind >= MacType::e1000_pch2lan) {
            let ret = if self.params.mtu > ETH_DATA_LEN {
                self.lv_jumbo_workaround_ich8lan(true)
            } else {
                self.lv_jumbo_workaround_ich8lan(false)
            };

            if ret.is_err() {
                error!("failed to enable|disable jumbo frame workaround mode");
            }
        }

        /* Program MC offset vector base */
        let mut rctl = self.er32(RCTL);
        rctl &= !(3 << RCTL_MO_SHIFT);
        rctl =
            RCTL_BAM | RCTL_LBM_NO | RCTL_RDMTS_HALF | (self.mac.mc_filter_type << RCTL_MO_SHIFT);

        rctl &= !(0b11 << 10); // Legacy description type

        /* Do not Store bad packets */
        rctl &= !RCTL_SBP;

        /* Enable Long Packet receive */
        if self.params.mtu > ETH_DATA_LEN {
            rctl |= RCTL_LPE;
        } else {
            rctl &= !RCTL_LPE;
        }

        /* Some systems expect that the CRC is included in SMBUS traffic. The
         * hardware strips the CRC before sending to both SMBUS (BMC) and to
         * host memory when this is enabled
         */
        if (self.params.flags.crc_stripping) {
            rctl |= RCTL_SECRC;
        }

        /* Setup buffer sizes */
        rctl &= !RCTL_SZ_4096;
        rctl |= RCTL_BSEX;

        let rx_buffer_len = self.rx_ring.buff_len;

        match rx_buffer_len {
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

        self.ew32(RCTL, rctl);

        self.params.flags.restart_now = false;
    }
    fn configure_rx(&mut self) {
        //TODO

        let rdlen = self.rx_ring.len() as u32;
        /* disable receives while setting up the descriptors */
        let mut rctl = self.er32(RCTL);
        if !self.params.flags.no_disable_rx {
            self.ew32(RCTL, rctl & !RCTL_EN);
        }
        self.e1e_flush();

        kfn().delay(Duration::from_micros(10000));

        if self.params.flags.dma_burst {
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
        self.ew32(RDTR, self.params.rx_int_delay);
        // self.ew32(RDTR, 0);

        /* irq moderation */
        self.ew32(RADV, self.params.rx_abs_int_delay);
        // self.ew32(RADV, 0);

        if (self.params.itr_settings != 0) && (self.params.itr != 0) {
            self.write_itr(self.params.itr);
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
        let mut rdba = self.rx_ring.dma_desc.dma_addr();
        self.ew32(RDBAL(0), (rdba & DMA_BIT_MASK(32)) as u32);
        self.ew32(RDBAH(0), (rdba >> 32) as u32);
        self.ew32(RDLEN(0), rdlen);
        self.ew32(RDH(0), 0);

        // self.rx_ring.next_to_clean = 0;
        // self.ew32(RDT(0), 0);

        self.ew32(RDT(0), rdlen - 1);
        self.rx_ring.next_to_clean = 0;

        // writel(0, rx_ring->head);
        // if (adapter->flags2 & FLAG2_PCIM2PCI_ARBITER_WA)
        // 	e1000e_update_rdt_wa(rx_ring, 0);
        // else
        // 	writel(0, rx_ring->tail);

        /* Enable Receive Checksum Offload for TCP and UDP */
        let mut rxcsum = self.er32(RXCSUM);
        rxcsum &= !RXCSUM_IPOFLD;
        rxcsum &= !RXCSUM_TUOFL;
        self.ew32(RXCSUM, rxcsum);

        self.ew32(RCTL, rctl);
    }
    fn setup_default_fc(&mut self) -> Result {
        if (self.fc.requested_mode == FcMode::Default) {
            match &self.variant.kind {
                variant::VariantKind::B82571(_) => match self.mac.kind {
                    MacType::e1000_82573 | MacType::e1000_82574 | MacType::e1000_82583 => {
                        self.fc.requested_mode = FcMode::Full;
                    }
                    _ => {}
                },
                variant::VariantKind::Ich8lan(_) => match self.mac.kind {
                    MacType::e1000_pchlan => {
                        self.fc.requested_mode = FcMode::RxPause;
                    }
                    _ => {
                        self.fc.requested_mode = FcMode::Full;
                    }
                },
            }
        }
        if self.fc.requested_mode == FcMode::Default {
            return self.set_default_fc_generic();
        }

        Ok(())
    }

    fn setup_link(&mut self) -> Result {
        self.setup_default_fc()?;

        /* Save off the requested flow control mode for use later.  Depending
         * on the link partner's capabilities, we may or may not use this mode.
         */
        self.fc.current_mode = self.fc.requested_mode;

        info!(
            "After fix-ups FlowControl is now = {:?}",
            self.fc.current_mode
        );

        if self.mac.autoneg {
            self.phy.copper_link_autoneg(self.fc.current_mode)?;
        } else {
            info!("Forcing Speed and Duplex");
        }

        match self.mac.ops.setup_physical_interface {
            SetupPhysicalInterface::CopperLink82571 => self.phy.setup_copper_link_82571(),
            SetupPhysicalInterface::CopperLinkPchLpt => self.phy.setup_copper_link_pch_lpt(),
        }?;

        let link = self.phy.phy_has_link_generic(10, 10)?;

        if link {
            info!("Valid link established!!!");
            self.mac.config_collision_dist_generic();
            self.config_fc_after_link_up();
        } else {
            info!("Unable to establish link!!!");
        }

        Ok(())
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
     *  e1000e_init_rx_addrs - Initialize receive address's
     *  @hw: pointer to the HW structure
     *  @rar_count: receive address registers
     *
     *  Setup the receive address registers by setting the base receive address
     *  register to the devices MAC address and clearing all the other receive
     *  address registers to 0.
     **/
    pub(crate) fn init_rx_addrs(&self, rar_count: u16) {
        let mut mac = MacAddress(self.mac.addr);

        /* Setup the receive address */
        info!("Programming MAC Address into RAR[0]: {}", mac);

        self.rar_set(0, mac);

        /* Zero out the other (rar_entry_count - 1) receive addresses */
        info!("Clearing RAR[1-{}]", rar_count - 1);

        mac = MacAddress([0; 6]);
        for i in 1..rar_count {
            self.rar_set(i as usize, mac);
        }
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

        self.reset_hw()?;

        if self.params.flags.has_amt {
            self.hw_contrl = Some(HwControl::new(self.reg, self.flags()));
        }

        self.ew32(WUC, 0);

        self.init_hw()?;

        self.reset_adaptive();

        Ok(())
    }
    /**
     *  e1000_lv_jumbo_workaround_ich8lan - required for jumbo frame operation
     *  with 82579 PHY
     *  @hw: pointer to the HW structure
     *  @enable: flag to enable/disable workaround when enabling/disabling jumbos
     **/
    fn lv_jumbo_workaround_ich8lan(&mut self, enable: bool) -> Result {
        /* disable Rx path while enabling/disabling workaround */
        let mut phy_reg = self.e1e_rphy(PHY_REG(769, 20))?;
        self.e1e_wphy(PHY_REG(769, 20), phy_reg | (BIT(14) as u16))?;

        if (enable) {
            /* Write Rx addresses (rar_entry_count for RAL/H, and
             * SHRAL/H) and initial CRC values to the MAC
             */
            // for i in 0..self.hw.as_ref().mac.rar_entry_count as usize {
            //     let mut mac_addr = [0u8; 6];

            // 	let addr_high = self.er32(RAH(i));
            // 	if (addr_high & RAH_AV)==0{
            // 		continue;
            //     }
            // 	let addr_low = self.er32(RAL(i));
            // 	mac_addr[0] = (addr_low & 0xFF) as u8;
            // 	mac_addr[1] = ((addr_low >> 8) & 0xFF) as u8;
            // 	mac_addr[2] = ((addr_low >> 16) & 0xFF) as u8;
            // 	mac_addr[3] = ((addr_low >> 24) & 0xFF) as u8;
            // 	mac_addr[4] = (addr_high & 0xFF) as u8;
            // 	mac_addr[5] = ((addr_high >> 8) & 0xFF) as u8;

            //     self.	ew32(PCH_RAICC(i), !ether_crc_le(6, mac_addr));
            // }

            // /* Write Rx addresses to the PHY */
            // e1000_copy_rx_addrs_to_phy_ich8lan(hw);

            // /* Enable jumbo frame workaround in the MAC */
            // mac_reg = er32(FFLT_DBG);
            // mac_reg &= ~BIT(14);
            // mac_reg |= (7 << 15);
            // ew32(FFLT_DBG, mac_reg);

            // mac_reg = er32(RCTL);
            // mac_reg |= E1000_RCTL_SECRC;
            // ew32(RCTL, mac_reg);

            // ret_val = e1000e_read_kmrn_reg(hw,
            // 			       E1000_KMRNCTRLSTA_CTRL_OFFSET,
            // 			       &data);
            // if (ret_val)
            // 	return ret_val;
            // ret_val = e1000e_write_kmrn_reg(hw,
            // 				E1000_KMRNCTRLSTA_CTRL_OFFSET,
            // 				data | BIT(0));
            // if (ret_val)
            // 	return ret_val;
            // ret_val = e1000e_read_kmrn_reg(hw,
            // 			       E1000_KMRNCTRLSTA_HD_CTRL,
            // 			       &data);
            // if (ret_val)
            // 	return ret_val;
            // data &= ~(0xF << 8);
            // data |= (0xB << 8);
            // ret_val = e1000e_write_kmrn_reg(hw,
            // 				E1000_KMRNCTRLSTA_HD_CTRL,
            // 				data);
            // if (ret_val)
            // 	return ret_val;

            // /* Enable jumbo frame workaround in the PHY */
            // e1e_rphy(hw, PHY_REG(769, 23), &data);
            // data &= ~(0x7F << 5);
            // data |= (0x37 << 5);
            // ret_val = e1e_wphy(hw, PHY_REG(769, 23), data);
            // if (ret_val)
            // 	return ret_val;
            // e1e_rphy(hw, PHY_REG(769, 16), &data);
            // data &= ~BIT(13);
            // ret_val = e1e_wphy(hw, PHY_REG(769, 16), data);
            // if (ret_val)
            // 	return ret_val;
            // e1e_rphy(hw, PHY_REG(776, 20), &data);
            // data &= ~(0x3FF << 2);
            // data |= (E1000_TX_PTR_GAP << 2);
            // ret_val = e1e_wphy(hw, PHY_REG(776, 20), data);
            // if (ret_val)
            // 	return ret_val;
            // ret_val = e1e_wphy(hw, PHY_REG(776, 23), 0xF100);
            // if (ret_val)
            // 	return ret_val;
            // e1e_rphy(hw, HV_PM_CTRL, &data);
            // ret_val = e1e_wphy(hw, HV_PM_CTRL, data | BIT(10));
            // if (ret_val)
            // 	return ret_val;
        } else {
            /* Write MAC register values back to h/w defaults */
            let mut mac_reg = self.er32(FFLT_DBG);
            mac_reg &= !(0xF << 14);
            self.ew32(FFLT_DBG, mac_reg);

            mac_reg = self.er32(RCTL);
            mac_reg &= !RCTL_SECRC;
            self.ew32(RCTL, mac_reg);

            let mut data = self.phy.read_kmrn_reg(KMRNCTRLSTA_CTRL_OFFSET);

            self.phy
                .write_kmrn_reg(KMRNCTRLSTA_CTRL_OFFSET, data & !(BIT(0) as u16));

            data = self.phy.read_kmrn_reg(KMRNCTRLSTA_HD_CTRL);

            data &= !(0xF << 8);
            data |= (0xB << 8);

            self.phy.write_kmrn_reg(KMRNCTRLSTA_HD_CTRL, data);

            /* Write PHY register values back to h/w defaults */
            let mut data = self.e1e_rphy(PHY_REG(769, 23))?;
            data &= !(0x7F << 5);
            self.e1e_wphy(PHY_REG(769, 23), data)?;
            data = self.e1e_rphy(PHY_REG(769, 16))?;
            data |= BIT(13) as u16;
            self.e1e_wphy(PHY_REG(769, 16), data)?;
            data = self.e1e_rphy(PHY_REG(776, 20))?;
            data &= !(0x3FF << 2);
            data |= (0x8 << 2);
            self.e1e_wphy(PHY_REG(776, 20), data)?;
            self.e1e_wphy(PHY_REG(776, 23), 0x7E00)?;
            data = self.e1e_rphy(HV_PM_CTRL)?;
            self.e1e_wphy(HV_PM_CTRL, data & !(BIT(10) as u16))?;
        }

        /* re-enable Rx path after enabling/disabling workaround */
        return self.e1e_wphy(PHY_REG(769, 20), phy_reg & !(BIT(14) as u16));
    }
    /**
     *  e1000e_reset_adaptive - Reset Adaptive Interframe Spacing
     *  @hw: pointer to the HW structure
     *
     *  Reset the Adaptive Interframe Spacing throttle to default values.
     **/
    fn reset_adaptive(&mut self) {
        if !self.mac.adaptive_ifs {
            debug!("Not in Adaptive IFS mode!\n");
            return;
        }

        self.mac.current_ifs_val = 0;
        self.mac.ifs_min_val = IFS_MIN;
        self.mac.ifs_max_val = IFS_MAX;
        self.mac.ifs_step_size = IFS_STEP;
        self.mac.ifs_ratio = IFS_RATIO;

        self.mac.in_ifs_mode = false;
        self.ew32(AIT, 0);
    }

    fn reset_hw(&self) -> Result {
        self.variant.reset_hw(self)
    }
    fn init_hw(&self) -> Result {
        self.variant.init_hw(self)
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

        kfn().delay(Duration::from_micros(200));
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

        kfn().delay(Duration::from_micros(100));

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

        kfn().delay(Duration::from_micros(100));

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

    pub(crate) fn ew32(&self, offset: u32, value: u32) {
        self.reg.ew32(offset, value);
    }

    pub(crate) fn er32(&self, offset: u32) -> u32 {
        self.reg.er32(offset)
    }
    pub(crate) fn e1e_flush(&self) {
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
    /**
     * e1000_get_speed_and_duplex
     * @hw: Struct containing variables accessed by shared code
     * @speed: Speed of the connection
     * @duplex: Duplex setting of the connection
     *
     * Detects the current speed and duplex settings of the hardware.
     */
    pub(crate) fn get_speed_and_duplex_copper(&mut self) {
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
        let link = status & STATUS_LU != 0;
        if (status & STATUS_FD) > 0 {
            duplex = FULL_DUPLEX;
        } else {
            duplex = HALF_DUPLEX;
        }

        self.params.link_speed = speed;
        self.params.link_duplex = duplex;

        info!(
            "{} Mbps, {} Duplex Link {}",
            speed,
            if duplex == FULL_DUPLEX {
                "Full"
            } else {
                "Half"
            },
            link
        );
    }

    fn config_fc_after_link_up(&mut self) -> Result {
        self.get_speed_and_duplex_copper();

        if self.mac.autoneg_failed {
        } else {
            self.mac.force_mac_fc(self.fc.current_mode)?;
        }

        /* Check for the case where we have copper media and auto-neg is
         * enabled.  In this case, we need to check and see if Auto-Neg
         * has completed, and if so, how the PHY and link partner has
         * flow control configured.
         */
        if ((self.phy.media_type == MediaType::media_type_copper) && self.mac.autoneg) {
            /* Read the MII Status Register and check to see if AutoNeg
             * has completed.  We read this twice because this reg has
             * some "sticky" (latched) bits.
             */
            let mut mii_status_reg = self.e1e_rphy(MII_BMSR)?;

            mii_status_reg = self.e1e_rphy(MII_BMSR)?;

            if (mii_status_reg & BMSR_ANEGCOMPLETE) == 0 {
                error!("Copper PHY and Auto Neg has not completed.");
                return Err(Error::Unknown("Copper PHY and Auto Neg has not completed."));
            }

            /* The AutoNeg process has completed, so we now need to
             * read both the Auto Negotiation Advertisement
             * Register (Address 4) and the Auto_Negotiation Base
             * Page Ability Register (Address 5) to determine how
             * flow control was negotiated.
             */
            let mut mii_nway_adv_reg = self.e1e_rphy(MII_ADVERTISE)?;

            let mut mii_nway_lp_ability_reg = self.e1e_rphy(MII_LPA)?;

            /* Two bits in the Auto Negotiation Advertisement Register
             * (Address 4) and two bits in the Auto Negotiation Base
             * Page Ability Register (Address 5) determine flow control
             * for both the PHY and the link partner.  The following
             * table, taken out of the IEEE 802.3ab/D6.0 dated March 25,
             * 1999, describes these PAUSE resolution bits and how flow
             * control is determined based upon these settings.
             * NOTE:  DC = Don't Care
             *
             *   LOCAL DEVICE  |   LINK PARTNER
             * PAUSE | ASM_DIR | PAUSE | ASM_DIR | NIC Resolution
             *-------|---------|-------|---------|--------------------
             *   0   |    0    |  DC   |   DC    | e1000_fc_none
             *   0   |    1    |   0   |   DC    | e1000_fc_none
             *   0   |    1    |   1   |    0    | e1000_fc_none
             *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
             *   1   |    0    |   0   |   DC    | e1000_fc_none
             *   1   |   DC    |   1   |   DC    | e1000_fc_full
             *   1   |    1    |   0   |    0    | e1000_fc_none
             *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
             *
             * Are both PAUSE bits set to 1?  If so, this implies
             * Symmetric Flow Control is enabled at both ends.  The
             * ASM_DIR bits are irrelevant per the spec.
             *
             * For Symmetric Flow Control:
             *
             *   LOCAL DEVICE  |   LINK PARTNER
             * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
             *-------|---------|-------|---------|--------------------
             *   1   |   DC    |   1   |   DC    | E1000_fc_full
             *
             */
            if (mii_nway_adv_reg & ADVERTISE_PAUSE_CAP) > 0
                && (mii_nway_lp_ability_reg & LPA_PAUSE_CAP) > 0
            {
                /* Now we need to check if the user selected Rx ONLY
                 * of pause frames.  In this case, we had to advertise
                 * FULL flow control because we could not advertise Rx
                 * ONLY. Hence, we must now check to see if we need to
                 * turn OFF the TRANSMISSION of PAUSE frames.
                 */
                if (self.fc.requested_mode == FcMode::Full) {
                    self.fc.current_mode = FcMode::Full;
                    info!("Flow Control = FULL.");
                } else {
                    self.fc.current_mode = FcMode::RxPause;
                    info!("Flow Control = Rx PAUSE frames only.");
                }
            }
            /* For receiving PAUSE frames ONLY.
             *
             *   LOCAL DEVICE  |   LINK PARTNER
             * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
             *-------|---------|-------|---------|--------------------
             *   0   |    1    |   1   |    1    | e1000_fc_tx_pause
             */
            else if ((mii_nway_adv_reg & ADVERTISE_PAUSE_CAP) == 0
                && (mii_nway_adv_reg & ADVERTISE_PAUSE_ASYM) > 0
                && (mii_nway_lp_ability_reg & LPA_PAUSE_CAP) > 0
                && (mii_nway_lp_ability_reg & LPA_PAUSE_ASYM) > 0)
            {
                self.fc.current_mode = FcMode::TxPause;
                info!("Flow Control = Tx PAUSE frames only.");
            }
            /* For transmitting PAUSE frames ONLY.
             *
             *   LOCAL DEVICE  |   LINK PARTNER
             * PAUSE | ASM_DIR | PAUSE | ASM_DIR | Result
             *-------|---------|-------|---------|--------------------
             *   1   |    1    |   0   |    1    | e1000_fc_rx_pause
             */
            else if ((mii_nway_adv_reg & ADVERTISE_PAUSE_CAP) > 0
                && (mii_nway_adv_reg & ADVERTISE_PAUSE_ASYM) > 0
                && (mii_nway_lp_ability_reg & LPA_PAUSE_CAP) == 0
                && (mii_nway_lp_ability_reg & LPA_PAUSE_ASYM) > 0)
            {
                self.fc.current_mode = FcMode::RxPause;
                info!("Flow Control = Rx PAUSE frames only.\n");
            } else {
                /* Per the IEEE spec, at this point flow control
                 * should be disabled.
                 */
                self.fc.current_mode = FcMode::None;
                info!("Flow Control = NONE.");
            }

            /* Now we need to do one last check...  If we auto-
             * negotiated to HALF DUPLEX, flow control should not be
             * enabled per IEEE 802.3 spec.
             */
            self.get_speed_and_duplex_copper();

            if (self.params.link_duplex == HALF_DUPLEX) {
                self.fc.current_mode = FcMode::None;
            }

            /* Now we call a subroutine to actually force the MAC
             * controller to use the correct flow control settings.
             */
            self.mac.force_mac_fc(self.fc.current_mode)?;
        }

        Ok(())
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

    pub(crate) fn set_fc_watermarks(&mut self) -> Result {
        let mut fcrtl = 0;
        let mut fcrth = 0;
        if self.fc.current_mode == FcMode::TxPause {
            /* We need to set up the Receive Threshold high and low water
             * marks as well as (optionally) enabling the transmission of
             * XON frames.
             */
            fcrtl = self.fc.low_water;
            if self.fc.send_xon {
                fcrtl |= FCRTL_XONE;
            }

            fcrth = self.fc.high_water;
        }
        self.ew32(FCRTL, fcrtl);
        self.ew32(FCRTH, fcrth);

        Ok(())
    }

    pub(crate) fn setup_link_generic(&mut self) -> Result {
        if self.fc.requested_mode == FcMode::Default {
            return self.set_default_fc_generic();
        }
        /* Save off the requested flow control mode for use later.  Depending
         * on the link partner's capabilities, we may or may not use this mode.
         */
        self.fc.current_mode = self.fc.requested_mode;

        info!(
            "After fix-ups FlowControl is now = {:?}",
            self.fc.current_mode
        );

        // self.setup_physical_interface()?;

        /* Initialize the flow control address, type, and PAUSE timer
         * registers to their default values.  This is done even if flow
         * control is disabled, because it does not hurt anything to
         * initialize these registers.
         */
        info!("Initializing the Flow Control address, type and timer regs");
        self.ew32(FCT, FLOW_CONTROL_TYPE);
        self.ew32(FCAH, FLOW_CONTROL_ADDRESS_HIGH);
        self.ew32(FCAL, FLOW_CONTROL_ADDRESS_LOW);

        self.ew32(FCTTV, self.fc.pause_time as u32);

        self.set_fc_watermarks()?;

        Ok(())
    }

    fn set_default_fc_generic(&mut self) -> Result {
        /* Read and store word 0x0F of the EEPROM. This word contains bits
         * that determine the hardware's default PAUSE (flow control) mode,
         * a bit that determines whether the HW defaults to enabling or
         * disabling auto-negotiation, and the direction of the
         * SW defined pins. If there is no SW over-ride of the flow
         * control setting, then the variable hw->fc will
         * be initialized based on a value in the EEPROM.
         */
        let nvm_data = self.nvm.read_nvm(NVM_INIT_CONTROL2_REG, 1)?[0];

        if (nvm_data & NVM_WORD0F_PAUSE_MASK) == 0 {
            self.fc.requested_mode = FcMode::None;
        } else if (nvm_data & NVM_WORD0F_PAUSE_MASK) == NVM_WORD0F_ASM_DIR {
            self.fc.requested_mode = FcMode::TxPause;
        } else {
            self.fc.requested_mode = FcMode::Full;
        }
        Ok(())
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
