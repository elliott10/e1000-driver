use b82571::DevSpec82571;
use defines::*;
use phy::*;

use super::ich8lan::DevSpecIch8lan;
use super::regs::*;
use crate::err::{Error, Result};
use crate::*;
use core::alloc::Layout;
use core::ptr::NonNull;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use core::str;

pub(crate) const E1000_ICH_FWSM_RSPCIPHY: u32 = 0x00000040; // Reset PHY on PCI Reset
pub(crate) const E1000_ICH_FWSM_FW_VALID: u32 = 0x00008000; // FW established a valid mode
pub(crate) const E1000_ICH_FWSM_PCIM2PCI: u32 = 0x01000000; // ME PCIm-to-PCI active
pub(crate) const E1000_ICH_FWSM_PCIM2PCI_COUNT: u32 = 2000;
pub(crate) const E1000_ICH_MNG_IAMT_MODE: u32 = 0x2;

#[repr(C)]
#[derive(Clone, Default)]
pub(crate) struct MsixEntry {
    pub vector: u32, /* Kernel uses to write allocated vector */
    pub entry: u32,  /* Driver uses to specify entry, OS writes */
}
pub(crate) struct Hw<K>
where
    K: KernelFunc,
{
    pub(crate) kernel: K,
    pub(crate) mmio: usize,
    pub(crate) flash: Option<usize>,
    pub(crate) phy: PhyInfo,
    pub(crate) phy_regs: PhyRegs,
    pub(crate) fc: FcInfo,
    pub(crate) mac: MacInfo,
    pub(crate) nvm: NvmInfo,
    pub(crate) bus: BusInfo,
    pub(crate) flags: Flags,
    pub(crate) flags2: Flags2,
    pub(crate) variant: BoardVariant,
    pub(crate) status: E1000Status,
    pub(crate) mng_cookie: HostMngDhcpCookie,
    pub(crate) is_down: bool,
}

#[derive(Default, Clone)]
#[repr(C)]
pub(crate) struct HostMngDhcpCookie {
    pub(crate) signature: u32,
    pub(crate) status: u8,
    pub(crate) reserved0: u8,
    pub(crate) vlan_id: u16,
    pub(crate) reserved1: u32,
    pub(crate) reserved2: u16,
    pub(crate) reserved3: u8,
    pub(crate) checksum: u8,
}
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum MediaType {
    media_type_unknown = 0,
    media_type_copper = 1,
    media_type_fiber = 2,
    media_type_internal_serdes = 3,
    num_media_types,
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u32)]
pub enum MacType {
    e1000_82571,
    e1000_82572,
    e1000_82573,
    e1000_82574,
    e1000_82583,
    e1000_80003es2lan,
    e1000_ich8lan,
    e1000_ich9lan,
    e1000_ich10lan,
    e1000_pchlan,
    e1000_pch2lan,
    e1000_pch_lpt,
    e1000_pch_spt,
    e1000_pch_cnp,
    e1000_pch_tgp,
    e1000_pch_adp,
    e1000_pch_mtp,
    e1000_pch_lnp,
    e1000_pch_ptp,
    e1000_pch_nvp,
}
const MAX_MTA_REG: usize = 128;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct MacInfo {
    pub addr: [u8; 6],
    pub perm_addr: [u8; 6],

    pub kind: MacType,

    pub collision_delta: u32,
    pub ledctl_default: u32,
    pub ledctl_mode1: u32,
    pub ledctl_mode2: u32,
    pub mc_filter_type: u32,
    pub tx_packet_delta: u32,
    pub txcw: u32,

    pub current_ifs_val: u16,
    pub ifs_max_val: u16,
    pub ifs_min_val: u16,
    pub ifs_ratio: u16,
    pub ifs_step_size: u16,
    pub mta_reg_count: u16,

    // Maximum size of the MTA register table in all supported adapters
    // pub mta_shadow: [u32; MAX_MTA_REG],
    pub rar_entry_count: u16,

    pub forced_speed_duplex: u8,

    pub adaptive_ifs: bool,
    pub has_fwsm: bool,
    pub arc_subsystem_valid: bool,
    pub autoneg: bool,
    pub autoneg_failed: bool,
    pub get_link_status: bool,
    pub in_ifs_mode: bool,
    pub serdes_has_link: bool,
    pub tx_pkt_filtering: bool,
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct E1000HWStats {
    pub crcerrs: u64,
    pub algnerrc: u64,
    pub symerrs: u64,
    pub rxerrc: u64,
    pub mpc: u64,
    pub scc: u64,
    pub ecol: u64,
    pub mcc: u64,
    pub latecol: u64,
    pub colc: u64,
    pub dc: u64,
    pub tncrs: u64,
    pub sec: u64,
    pub cexterr: u64,
    pub rlec: u64,
    pub xonrxc: u64,
    pub xontxc: u64,
    pub xoffrxc: u64,
    pub xofftxc: u64,
    pub fcruc: u64,
    pub prc64: u64,
    pub prc127: u64,
    pub prc255: u64,
    pub prc511: u64,
    pub prc1023: u64,
    pub prc1522: u64,
    pub gprc: u64,
    pub bprc: u64,
    pub mprc: u64,
    pub gptc: u64,
    pub gorc: u64,
    pub gotc: u64,
    pub rnbc: u64,
    pub ruc: u64,
    pub rfc: u64,
    pub roc: u64,
    pub rjc: u64,
    pub mgprc: u64,
    pub mgpdc: u64,
    pub mgptc: u64,
    pub tor: u64,
    pub tot: u64,
    pub tpr: u64,
    pub tpt: u64,
    pub ptc64: u64,
    pub ptc127: u64,
    pub ptc255: u64,
    pub ptc511: u64,
    pub ptc1023: u64,
    pub ptc1522: u64,
    pub mptc: u64,
    pub bptc: u64,
    pub tsctc: u64,
    pub tsctfc: u64,
    pub iac: u64,
    pub icrxptc: u64,
    pub icrxatc: u64,
    pub ictxptc: u64,
    pub ictxatc: u64,
    pub ictxqec: u64,
    pub ictxqmtc: u64,
    pub icrxdmtc: u64,
    pub icrxoc: u64,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum MsType {
    HwDefault = 0,
    ForceMaster,
    ForceSlave,
    Auto,
}
impl Default for MsType {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PhyType {
    Unknown,
    None,
    M88,
    Igp,
    Igp2,
    Gg82563,
    Igp3,
    Ife,
    Bm,
    T82578,
    T82577,
    T82579,
    I217,
}
impl Default for PhyType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for MediaType {
    fn default() -> Self {
        Self::media_type_unknown
    }
}

#[derive(Debug, Default)]
pub struct PhyInfo {
    pub kind: PhyType,

    pub addr: u32,
    pub id: u32,
    pub reset_delay_us: u32, // in microseconds
    pub revision: u32,

    pub media_type: MediaType,

    pub autoneg_advertised: u16,
    pub autoneg_mask: u16,
    pub cable_length: u16,
    pub max_cable_length: u16,
    pub min_cable_length: u16,

    pub mdix: u8,

    pub disable_polarity_correction: bool,
    pub is_mdix: bool,
    pub polarity_correction: bool,
    pub speed_downgraded: bool,
    pub autoneg_wait_to_complete: bool,

    pub(crate) ms_type: MsType,
    pub(crate) original_ms_type: MsType,
}

impl PhyInfo {
    pub(crate) fn copper_options(&mut self) {
        if self.media_type == MediaType::media_type_copper {
            self.mdix = AUTO_ALL_MODES;
            self.disable_polarity_correction = false;
            self.ms_type = MsType::HwDefault;
        }
    }
}
#[derive(Default, Debug, Clone)]
#[repr(C)]
pub(crate) struct PhyRegs {
    pub(crate) bmcr: u16,      // Basic Mode Control Register
    pub(crate) bmsr: u16,      // Basic Mode Status Register
    pub(crate) advertise: u16, // Auto-Negotiation Advertisement
    pub(crate) lpa: u16,       // Link Partner Ability Register
    pub(crate) expansion: u16, // Auto-Negotiation Expansion Register
    pub(crate) ctrl1000: u16,  // 1000BASE-T Control Register
    pub(crate) stat1000: u16,  // 1000BASE-T Status Register
    pub(crate) estatus: u16,   // Extended Status Register
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub(crate) enum BusWidth {
    Unknown = 0,
    PcieX1,
    PcieX2,
    PcieX4 = 4,
    PcieX8 = 8,
    ThirtyTwo,
    SixtyFour,
    Reserved,
}
impl Default for BusWidth {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct FcInfo {
    pub high_water: u32,        // Flow control high-water mark
    pub low_water: u32,         // Flow control low-water mark
    pub pause_time: u16,        // Flow control pause timer
    pub refresh_time: u16,      // Flow control refresh timer
    pub send_xon: bool,         // Flow control send XON
    pub strict_ieee: bool,      // Strict IEEE mode
    pub current_mode: FcMode,   // FC mode in effect
    pub requested_mode: FcMode, // FC mode requested by caller
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub(crate) enum FcMode {
    None = 0,
    RxPause,
    TxPause,
    Full,
    Default = 0xFF,
}
impl Default for FcMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum NvmType {
    Unknown = 0,
    None,
    EepromSpi,
    FlashHw,
    FlashSw,
}
impl Default for NvmType {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default)]
pub(crate) struct NvmInfo {
    pub(crate) kind: NvmType,
    pub(crate) nvm_override: NvmOverride,

    pub(crate) flash_bank_size: u32,
    pub(crate) flash_base_addr: u32,

    pub(crate) word_size: u16,
    pub(crate) delay_usec: u16,
    pub(crate) address_bits: u16,
    pub(crate) opcode_bits: u16,
    pub(crate) page_size: u16,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum NvmOverride {
    None = 0,
    SpiSmall,
    SpiLarge,
}
impl Default for NvmOverride {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
pub(crate) struct BusInfo {
    pub(crate) width: BusWidth,
    pub(crate) fun: u16,
}

impl<K> Hw<K>
where
    K: KernelFunc,
{
    pub(crate) fn crc_stripping(&mut self) {
        self.flags2.crc_stripping = true;
        self.flags2.dflt_crc_stripping = true;
    }

    pub(crate) fn write_protect_nvm(&mut self) {
        if self.flags.is_ich {
            self.flags.read_only_nvm = true;
        }
    }
    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.mmio + offset) as *const T;
            ptr.read_volatile()
        }
    }
    pub(crate) fn er32(&self, offset: usize) -> u32 {
        self.read(offset)
    }
    fn ew32_prepare(&self) {
        let mut i = E1000_ICH_FWSM_PCIM2PCI_COUNT;

        while ((self.er32(FWSM) & E1000_ICH_FWSM_PCIM2PCI) != 0) && i > 0 {
            self.kernel.delay(50_000);
            i -= 1;
        }
    }
    pub(crate) fn ew32(&self, offset: usize, value: u32) {
        if self.flags2.pcim2pci_arbiter_wa {
            self.ew32_prepare();
        }
        unsafe {
            let ptr = (self.mmio + offset) as *mut u32;
            ptr.write_volatile(value);
        }
    }

    pub(crate) fn e1e_flush(&self) {
        self.er32(STATUS);
    }

    pub(crate) fn er16flash(&self, reg: usize) -> Result<u16> {
        match self.flash {
            None => {
                error!("flash not set");
                Err(Error::Nvm("flash not set"))
            }
            Some(flash) => unsafe { Ok(((flash + reg) as *const u16).read_volatile()) },
        }
    }
    pub(crate) fn er32flash(&self, reg: usize) -> Result<u32> {
        match self.flash {
            None => {
                error!("flash not set");
                Err(Error::Nvm("flash not set"))
            }
            Some(flash) => unsafe { Ok(((flash + reg) as *const u32).read_volatile()) },
        }
    }
    pub(crate) fn ew16flash(&self, reg: usize, val: u16) -> Result {
        match self.flash {
            None => {
                error!("flash not set");
                Err(Error::Nvm("flash not set"))
            }
            Some(flash) => unsafe {
                ((flash + reg) as *mut u16).write_volatile(val);
                Ok(())
            },
        }
    }
    pub(crate) fn ew32flash(&self, reg: usize, val: u32) -> Result {
        match self.flash {
            None => {
                error!("flash not set");
                Err(Error::Nvm("flash not set"))
            }
            Some(flash) => unsafe {
                ((flash + reg) as *mut u32).write_volatile(val);
                Ok(())
            },
        }
    }
    pub(crate) fn ral(&self, index: usize) -> usize {
        RA + index * 8
    }

    pub(crate) fn rah(&self, index: usize) -> usize {
        RA + index * 8 + 4
    }

    pub(crate) fn write_reg_array(&self, reg: usize, offset: usize, value: u32) {
        let address = reg + (offset << 2);
        self.ew32(address, value);
    }

    pub(crate) fn poll_eerd_eewr_done(&self, ee_reg: u8) -> Result {
        let mut reg;

        for _ in 0..100000 {
            if (ee_reg == NVM_POLL_READ) {
                reg = self.er32(EERD);
            } else {
                reg = self.er32(EEWR);
            }

            if (reg & NVM_RW_REG_DONE > 0) {
                return Ok(());
            }

            self.kernel.delay(5000);
        }

        Err(Error::Nvm("poll fail"))
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

    pub(crate) fn disable_pcie_master(&mut self) -> Result {
        let mut ctrl = self.er32(CTRL);
        ctrl |= CTRL_GIO_MASTER_DISABLE;
        let mut timeout = MASTER_DISABLE_TIMEOUT;

        self.ew32(CTRL, ctrl);

        while timeout > 0 {
            if self.er32(STATUS) & STATUS_GIO_MASTER_ENABLE == 0 {
                break;
            }
            self.kernel.delay(100_000);
            timeout -= 1;
        }

        if timeout > 0 {
            return Ok(());
        }

        return Err(Error::Unknown("Master requests are pending."));
    }

    /**
     *  e1000e_get_auto_rd_done - Check for auto read completion
     *  @hw: pointer to the HW structure
     *
     *  Check EEPROM for Auto Read done bit.
     **/
    pub(crate) fn get_auto_rd_done(&mut self) -> Result {
        for _ in 0..AUTO_READ_DONE_TIMEOUT {
            if (self.er32(EECD) & EECD_AUTO_RD) > 0 {
                return Ok(());
            }
            self.kernel.delay(1000_000);
        }

        error!("Auto read by HW from NVM has not completed.\n");

        Err(Error::Nvm("Auto read by HW from NVM has not completed."))
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

        self.rar_set(mac, 0);

        /* Zero out the other (rar_entry_count - 1) receive addresses */
        info!("Clearing RAR[1-{}]\n", rar_count - 1);

        mac = MacAddress([0; 6]);
        for i in 1..rar_count {
            self.rar_set(mac, i as usize);
        }
    }

    pub(crate) fn rar_set(&self, mac: MacAddress, index: usize) {
        if self.mac.kind >= MacType::e1000_pch_lpt {
            self.rar_set_pch_lpt(mac, index);
        } else {
            self.rar_set_generic(mac, index);
        }
    }

    /**
     *  e1000e_rar_set_generic - Set receive address register
     *  @hw: pointer to the HW structure
     *  @addr: pointer to the receive address
     *  @index: receive address array register
     *
     *  Sets the receive address array register at index to the address passed
     *  in by addr.
     **/
    fn rar_set_generic(&self, addr: MacAddress, index: usize) {
        let rar_low = u32::from_le_bytes([addr.0[0], addr.0[1], addr.0[2], addr.0[3]]);
        let mut rar_high = u16::from_le_bytes([addr.0[4], addr.0[5]]) as u32;
        /* HW expects these in little endian so we reverse the byte order
         * from network order (big endian) to little endian
         */

        /* If MAC address zero, no need to set the AV bit */
        if rar_low > 0 || rar_high > 0 {
            rar_high |= RAH_AV;
        }

        /* Some bridges will combine consecutive 32-bit writes into
         * a single burst write, which will malfunction on some parts.
         * The flushes avoid this.
         */
        self.ew32(self.ral(index), rar_low);
        self.e1e_flush();
        self.ew32(self.rah(index), rar_high);
        self.e1e_flush();
    }

    /**
     *  e1000e_rar_set_generic - Set receive address register
     *  @hw: pointer to the HW structure
     *  @addr: pointer to the receive address
     *  @index: receive address array register
     *
     *  Sets the receive address array register at index to the address passed
     *  in by addr.
     **/
    fn rar_set_pch_lpt(&self, addr: MacAddress, index: usize) {
        let rar_low = u32::from_le_bytes([addr.0[0], addr.0[1], addr.0[2], addr.0[3]]);
        let mut rar_high = u16::from_le_bytes([addr.0[4], addr.0[5]]) as u32;
        /* HW expects these in little endian so we reverse the byte order
         * from network order (big endian) to little endian
         */

        /* If MAC address zero, no need to set the AV bit */

        if rar_low > 0 || rar_high > 0 {
            rar_high |= RAH_AV;
        }

        if index == 0 {
            self.ew32(self.ral(index), rar_low);
            self.e1e_flush();
            self.ew32(self.rah(index), rar_high);
            self.e1e_flush();
            return;
        }
        if index < self.mac.rar_entry_count as usize {
            let mut wlock_mac = self.er32(FWSM) & FWSM_WLOCK_MAC_MASK;
            wlock_mac >>= FWSM_WLOCK_MAC_SHIFT;

            // Check if all SHRAR registers are locked
            if wlock_mac == 1 {
                return; // Replace 'goto out' with a direct return or continue logic as per your function design
            }

            if wlock_mac == 0 || index <= wlock_mac as usize {
                // let ret_val = e1000_acquire_swflag_ich8lan(hw);

                // if ret_val != 0 {
                //     return; // Same as above, replace 'goto out' with a return or continue logic
                // }

                self.ew32(SHRAL_PCH_LPT(index - 1), rar_low);
                self.e1e_flush();
                self.ew32(SHRAH_PCH_LPT(index - 1), rar_high);
                self.e1e_flush();

                // e1000_release_swflag_ich8lan(hw);

                // Verify the register updates
                if self.er32(SHRAL_PCH_LPT(index - 1)) == rar_low
                    && self.er32(SHRAH_PCH_LPT(index - 1)) == rar_high
                {
                    return; // Success
                }
            }
        }
    }
    /**
     *  e1000e_get_phy_id - Retrieve the PHY ID and revision
     *  @hw: pointer to the HW structure
     *
     *  Reads the PHY registers and stores the PHY ID and possibly the PHY
     *  revision in the hardware structure.
     **/
    pub(crate) fn get_phy_id(&mut self) -> Result {
        for _ in 0..2 {
            let mut phy_id: u16 = self.e1e_rphy(MII_PHYSID1)?;
            self.phy.id = (phy_id as u32) << 16;
            self.kernel.delay(20_000);
            phy_id = self.e1e_rphy(MII_PHYSID2)?;

            self.phy.id |= (phy_id as u32 & PHY_REVISION_MASK);
            self.phy.revision = (phy_id as u32 & !PHY_REVISION_MASK);

            if (self.phy.id != 0 && self.phy.id != PHY_REVISION_MASK) {
                return Ok(());
            }
        }

        return Ok(());
    }

    /**
     *  get_phy_type_from_id - Get PHY type from id
     *  @phy_id: phy_id read from the phy
     *
     *  Returns the phy type from the id.
     **/
    pub(crate) fn get_phy_type_from_id(&self, phy_id: u32) -> PhyType {
        let mut phy_type = PhyType::Unknown;

        match phy_id {
            M88E1000_I_PHY_ID | M88E1000_E_PHY_ID | M88E1111_I_PHY_ID | M88E1011_I_PHY_ID => {
                phy_type = PhyType::M88;
            }
            IGP01E1000_I_PHY_ID => {
                /* IGP 1 & 2 share this */
                phy_type = PhyType::Igp2;
            }
            GG82563_E_PHY_ID => {
                phy_type = PhyType::Gg82563;
            }
            IGP03E1000_E_PHY_ID => {
                phy_type = PhyType::Igp3;
            }
            IFE_E_PHY_ID | IFE_PLUS_E_PHY_ID | IFE_C_E_PHY_ID => {
                phy_type = PhyType::Ife;
            }
            BME1000_E_PHY_ID | BME1000_E_PHY_ID_R2 => {
                phy_type = PhyType::Bm;
            }
            I82578_E_PHY_ID => {
                phy_type = PhyType::T82578;
            }
            I82577_E_PHY_ID => {
                phy_type = PhyType::T82577;
            }
            I82579_E_PHY_ID => {
                phy_type = PhyType::T82579;
            }
            I217_E_PHY_ID => {
                phy_type = PhyType::I217;
            }
            _ => {
                phy_type = PhyType::Unknown;
            }
        }
        return phy_type;
    }
    /**
     *  e1000e_determine_phy_address - Determines PHY address.
     *  @hw: pointer to the HW structure
     *
     *  This uses a trial and error method to loop through possible PHY
     *  addresses. It tests each by reading the PHY ID registers and
     *  checking for a match.
     **/
    pub(crate) fn determine_phy_address(&mut self) -> Result {
        info!("Determining PHY address");
        let mut phy_type = PhyType::Unknown;
        self.phy.id = 0;

        for phy_addr in 0..MAX_PHY_ADDR {
            self.phy.addr = phy_addr;

            for _ in 0..10 {
                self.get_phy_id();
                phy_type = self.get_phy_type_from_id(self.phy.id);

                /* If phy_type is valid, break - we found our
                 * PHY address
                 */
                if (phy_type != PhyType::Unknown) {
                    self.phy.kind = phy_type;
                    return Ok(());
                }

                self.kernel.delay(1000_000);
            }
        }

        return Err(Error::Unknown("Phy type error"));
    }
    /**
     *  e1000e_clear_hw_cntrs_base - Clear base hardware counters
     *  @hw: pointer to the HW structure
     *
     *  Clears the base hardware counters by reading the counter registers.
     **/
    pub(crate) fn clear_hw_cntrs_base(&self) {
        self.er32(CRCERRS);
        self.er32(SYMERRS);
        self.er32(MPC);
        self.er32(SCC);
        self.er32(ECOL);
        self.er32(MCC);
        self.er32(LATECOL);
        self.er32(COLC);
        self.er32(DC);
        self.er32(SEC);
        self.er32(RLEC);
        self.er32(XONRXC);
        self.er32(XONTXC);
        self.er32(XOFFRXC);
        self.er32(XOFFTXC);
        self.er32(FCRUC);
        self.er32(GPRC);
        self.er32(BPRC);
        self.er32(MPRC);
        self.er32(GPTC);
        self.er32(GORCL);
        self.er32(GORCH);
        self.er32(GOTCL);
        self.er32(GOTCH);
        self.er32(RNBC);
        self.er32(RUC);
        self.er32(RFC);
        self.er32(ROC);
        self.er32(RJC);
        self.er32(TORL);
        self.er32(TORH);
        self.er32(TOTL);
        self.er32(TOTH);
        self.er32(TPR);
        self.er32(TPT);
        self.er32(MPTC);
        self.er32(BPTC);
    }
    pub(crate) fn e1e_rphy(&mut self, offset: u32) -> Result<u16> {
        match self.variant {
            BoardVariant::B82571 => self.read_phy_reg_mdic(offset),
            BoardVariant::Ich8lan => self.read_phy_reg_hv(offset),
        }
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
        let mut mdic =
            ((offset << MDIC_REG_SHIFT) | (self.phy.addr << MDIC_PHY_SHIFT) | (MDIC_OP_READ));

        self.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT {
            self.udelay(50);
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
        if (self.mac.kind == MacType::e1000_pch2lan) {
            self.kernel.delay(100_000);
        }

        Ok(mdic as u16)
    }

    pub(crate) fn e1e_wphy(&mut self, offset: u32, value: u16) -> Result {
        match self.variant {
            BoardVariant::B82571 => self.write_phy_reg_mdic(offset, value),
            BoardVariant::Ich8lan => self.write_phy_reg_hv(offset, value),
        }
    }

    /**
     *  get_phy_addr_for_hv_page - Get PHY address based on page
     *  @page: page to be accessed
     **/
    fn get_phy_addr_for_hv_page(&mut self, page: u16) -> u32 {
        let mut phy_addr = 2;

        if (page >= HV_INTC_FC_PAGE_START) {
            phy_addr = 1;
        }

        return phy_addr;
    }

    fn read_phy_reg_hv(&mut self, offset: u32) -> Result<u16> {
        let mut page = BM_PHY_REG_PAGE(offset);
        let reg = BM_PHY_REG_NUM(offset);
        let phy_addr = self.get_phy_addr_for_hv_page(page);
        self.phy.addr = phy_addr;

        if page == BM_WUC_PAGE {
            error!("read_phy_reg_hv: BM_WUC_PAGE");
            return todo!();
        }
        if page > 0 && page < HV_INTC_FC_PAGE_START {
            error!("read_phy_reg_hv: HV_INTC_FC_PAGE_START");
            return todo!();
        }

        if (page == HV_INTC_FC_PAGE_START) {
            page = 0;
        }

        if reg > MAX_PHY_MULTI_PAGE_REG {
            /* Page is shifted left, PHY expects (page x 32) */
            let res = self.set_page_igp((page << IGP_PAGE_SHIFT));
            self.phy.addr = phy_addr;

            res?;
        }

        info!(
            "reading PHY addr {} page {} (or {:#X} shifted) reg {:#X}",
            self.phy.addr,
            page,
            page << IGP_PAGE_SHIFT,
            reg
        );

        self.read_phy_reg_mdic(MAX_PHY_REG_ADDRESS & reg as u32)
    }
    fn write_phy_reg_hv(&mut self, offset: u32, value: u16) -> Result {
        let mut page = BM_PHY_REG_PAGE(offset);
        let reg = BM_PHY_REG_NUM(offset);
        let phy_addr = self.get_phy_addr_for_hv_page(page);
        self.phy.addr = phy_addr;

        if page == BM_WUC_PAGE {
            error!("read_phy_reg_hv: BM_WUC_PAGE");
            return todo!();
        }
        if page > 0 && page < HV_INTC_FC_PAGE_START {
            error!("read_phy_reg_hv: HV_INTC_FC_PAGE_START");
            return todo!();
        }

        if (page == HV_INTC_FC_PAGE_START) {
            page = 0;
        }

        if reg > MAX_PHY_MULTI_PAGE_REG {
            /* Page is shifted left, PHY expects (page x 32) */
            let res = self.set_page_igp((page << IGP_PAGE_SHIFT));
            self.phy.addr = phy_addr;

            res?;
        }

        info!(
            "writing PHY addr {} page {} (or {:#X} shifted) reg {:#X}",
            self.phy.addr,
            page,
            page << IGP_PAGE_SHIFT,
            reg
        );

        self.write_phy_reg_mdic(MAX_PHY_REG_ADDRESS & reg as u32, value)
    }

    /**
     *  e1000_set_page_igp - Set page as on IGP-like PHY(s)
     *  @hw: pointer to the HW structure
     *  @page: page to set (shifted left when necessary)
     *
     *  Sets PHY page required for PHY register access.  Assumes semaphore is
     *  already acquired.  Note, this function sets phy.addr to 1 so the caller
     *  must set it appropriately (if necessary) after this function returns.
     **/
    fn set_page_igp(&mut self, page: u16) -> Result {
        info!("Setting page {:#X}", page);

        self.phy.addr = 1;

        self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, page)
    }

    fn write_phy_reg_igp(&mut self, offset: u32, value: u16) -> Result {
        if (offset as u16 > MAX_PHY_MULTI_PAGE_REG) {
            self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, offset as u16)?;
        }
        self.write_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset, value)
    }
    fn read_phy_reg_igp(&mut self, offset: u32) -> Result<u16> {
        if (offset as u16 > MAX_PHY_MULTI_PAGE_REG) {
            self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, offset as u16)?;
        }
        self.read_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset)
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
        let mut mdic =
            ((offset << MDIC_REG_SHIFT) | (self.phy.addr << MDIC_PHY_SHIFT) | (MDIC_OP_WRITE));

        self.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT {
            self.udelay(50);
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
        if (self.mac.kind == MacType::e1000_pch2lan) {
            self.udelay(100);
        }

        Ok(())
    }

    pub(crate) fn udelay(&self, val: u32) {
        self.kernel.delay(val as u64 * 1000);
    }

    /**
     * e1000_power_up_phy_copper - Restore copper link in case of PHY power down
     * @hw: pointer to the HW structure
     *
     * In the case of a PHY power down to save power, or to turn off link during a
     * driver unload, or wake on lan is not enabled, restore the link to previous
     * settings.
     **/
    pub fn power_up_phy_copper(&self) {
        // u16 mii_reg = 0;
        // int ret;

        // /* The PHY will retain its settings across a power down/up cycle */
        // let ret = e1e_rphy(hw, MII_BMCR, &mii_reg);
        // if (ret) {
        // e_dbg("Error reading PHY register\n");
        // /		return;
        // /}
        // mii_reg &= ~BMCR_PDOWN;
        // e1e_wphy(hw, MII_BMCR, mii_reg);
    }



}

#[derive(Clone)]
pub(crate) struct HwRc<K>(NonNull<Hw<K>>)
where
    K: KernelFunc;

unsafe impl<K: KernelFunc> Send for HwRc<K> {}

impl<K> HwRc<K>
where
    K: KernelFunc,
{
    pub(crate) fn new(kfun: K, board: &BoardInfo) -> Self {
        unsafe {
            let mmio = kfun.bar_n_remap(0);
            let layout = Layout::new::<Hw<K>>();
            let ptr = alloc::alloc(layout) as *mut Hw<K>;
            let hw = &mut *ptr;
            hw.kernel = kfun;
            hw.mmio = mmio;
            hw.flash = None;
            hw.status = Default::default();
            hw.phy = Default::default();
            hw.mac = Default::default();
            hw.nvm = Default::default();
            hw.fc = Default::default();
            hw.mng_cookie = Default::default();
            hw.is_down = false;
            hw.mac.kind = board.mac;
            hw.flags = board.flags;
            hw.flags2 = board.flags2;
            hw.variant = board.variant;
            hw.status.max_hw_frame_size = board.max_hw_frame_size;
            hw.status.pba = board.pba;

            /* Initialize link parameters. User can change them with ethtool */
            hw.mac.autoneg = true;
            hw.fc.requested_mode = FcMode::Default;
            hw.fc.current_mode = FcMode::Default;
            hw.phy.autoneg_advertised = 0x2f;

            Self(NonNull::new_unchecked(ptr))
        }
    }
    pub(crate) fn init(&mut self) -> Result {
        let hw = self.as_mut();
        hw.status.rx_buffer_len = (VLAN_ETH_FRAME_LEN + ETH_FCS_LEN) as u32;

        if hw.flags.has_flash && hw.mac.kind < MacType::e1000_pch_spt {
            let addr = hw.kernel.bar_n_remap(1);
            if addr > 0 {
                info!("flash: {:#X}", addr);
                hw.flash = Some(addr);
            } else {
                error!("remap bar1 failed");
                return Err(Error::Unknown("remap bar1 failed"));
            }
        }
        Ok(())
    }

    pub(crate) fn drop(&mut self) {
        let base = self.as_mut();
        debug!("unmap mmio {:#X}", base.mmio);
        base.kernel.iounmap(base.mmio);
        if let Some(flash) = base.flash {
            debug!("unmap flash {:#X}", flash);
            base.kernel.iounmap(flash);
        }
        unsafe {
            let layout = Layout::new::<Hw<K>>();
            let ptr = self.0.as_ptr() as *mut u8;
            alloc::dealloc(ptr, layout);
        }
    }
}

impl<K> AsRef<Hw<K>> for HwRc<K>
where
    K: KernelFunc,
{
    fn as_ref(&self) -> &Hw<K> {
        unsafe { self.0.as_ref() }
    }
}

impl<K> AsMut<Hw<K>> for HwRc<K>
where
    K: KernelFunc,
{
    fn as_mut(&mut self) -> &mut Hw<K> {
        unsafe { self.0.as_mut() }
    }
}

pub(crate) enum DevSpec {
    E82571(DevSpec82571),
    E80003es2lan,
    Ich8lan(DevSpecIch8lan),
}
pub(crate) struct DevSpecParam<'a> {
    pub(crate) nvm: &'a mut NvmInfo,
    pub(crate) phy: &'a mut PhyInfo,
    pub(crate) mac: &'a mut MacInfo,
    pub(crate) mmio: usize,
    pub(crate) flash: &'a mut Option<usize>,
    pub(crate) strap: u32,
}

impl DevSpec {
    pub(crate) fn get_variants<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        match self {
            DevSpec::E82571(i) => i.get_variants(hw),
            DevSpec::E80003es2lan => Ok(()),
            DevSpec::Ich8lan(i) => i.get_variants(hw),
        }
    }

    pub(crate) fn get_bus_info<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        match self {
            DevSpec::E82571(i) => i.get_bus_info(hw),
            DevSpec::E80003es2lan => Ok(()),
            DevSpec::Ich8lan(i) => Ok(()),
        }
    }

    pub(crate) fn read_nvm<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        offset: u16,
        words: u16,
    ) -> Result<Vec<u16>> {
        match self {
            DevSpec::E82571(i) => i.read_nvm(hw, offset, words),
            DevSpec::E80003es2lan => todo!(),
            DevSpec::Ich8lan(i) => i.read_nvm(hw, offset, words),
        }
    }

    pub(crate) fn reset_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        info!("reset hw");
        match self {
            DevSpec::E82571(i) => i.reset_hw(hw),
            DevSpec::E80003es2lan => todo!(),
            DevSpec::Ich8lan(i) => i.reset_hw(hw),
        }
    }

    pub(crate) fn init_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        match self {
            DevSpec::E82571(i) => i.init_hw(hw),
            DevSpec::E80003es2lan => todo!(),
            DevSpec::Ich8lan(i) => i.init_hw(hw),
        }
    }
}
