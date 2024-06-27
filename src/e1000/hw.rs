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
    pub(crate) is_print_ew: bool,
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

#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
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
        if self.is_print_ew {
            info!("reg: {:#x} val: {:#x}", offset, value);
        }

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
    pub(crate) fn read_kmrn_reg(&mut self, offset: u32) -> u16 {
        let mut kmrnctrlsta =
            ((offset << KMRNCTRLSTA_OFFSET_SHIFT) & KMRNCTRLSTA_OFFSET) | KMRNCTRLSTA_REN;
        self.ew32(KMRNCTRLSTA, kmrnctrlsta);
        self.e1e_flush();

        self.udelay(2);

        self.er32(KMRNCTRLSTA) as u16
    }
    pub(crate) fn write_kmrn_reg(&mut self, offset: u32, data: u16) {
        let mut kmrnctrlsta =
            ((offset << KMRNCTRLSTA_OFFSET_SHIFT) & KMRNCTRLSTA_OFFSET) | data as u32;
        self.ew32(KMRNCTRLSTA, kmrnctrlsta);
        self.e1e_flush();
        self.udelay(2);
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
    pub(crate) fn read_phy_reg_mdic(&mut self, offset: u32) -> Result<u16> {
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

        Ok((mdic & 0xFFFF) as u16)
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
        // let _g = self.get_phy_lock_ich8lan()?;
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

        self.acquire()?;
        if reg > MAX_PHY_MULTI_PAGE_REG {
            /* Page is shifted left, PHY expects (page x 32) */
            let res = self.set_page_igp((page << IGP_PAGE_SHIFT));
            self.phy.addr = phy_addr;

            if res.is_err() {
                self.release();
                res?;
            }
        }

        info!(
            "reading PHY addr {} page {} (or {:#X} shifted) reg {:#X}",
            self.phy.addr,
            page,
            page << IGP_PAGE_SHIFT,
            reg
        );

        let res = self.read_phy_reg_mdic(MAX_PHY_REG_ADDRESS & reg as u32);
        self.release();

        res
    }
    fn write_phy_reg_hv(&mut self, offset: u32, value: u16) -> Result {
        let mut page = BM_PHY_REG_PAGE(offset);
        let reg = BM_PHY_REG_NUM(offset);
        let mut phy_addr = self.get_phy_addr_for_hv_page(page);
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

        self.acquire()?;
        if reg > MAX_PHY_MULTI_PAGE_REG {
            /* Page is shifted left, PHY expects (page x 32) */
            let res = self.set_page_igp((page << IGP_PAGE_SHIFT));
            self.phy.addr = phy_addr;

            if res.is_err() {
                self.release();
                return res;
            }
        }

        info!(
            "writing PHY addr {} page {} (or {:#X} shifted) reg {:#X} data {:#X}",
            self.phy.addr,
            page,
            page << IGP_PAGE_SHIFT,
            reg,
            value
        );

        let res = self.write_phy_reg_mdic(MAX_PHY_REG_ADDRESS & reg as u32, value);

        self.release();

        res
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
        if (offset > MAX_PHY_MULTI_PAGE_REG) {
            self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, offset as u16)?;
        }
        self.write_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset, value)
    }
    fn read_phy_reg_igp(&mut self, offset: u32) -> Result<u16> {
        if (offset > MAX_PHY_MULTI_PAGE_REG) {
            self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, offset as u16)?;
        }
        self.read_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset)
    }

    pub(crate) fn write_phy_reg_mdic(&mut self, offset: u32, value: u16) -> Result {
        if (offset > MAX_PHY_REG_ADDRESS) {
            error!("PHY Address {} is out of range", offset);
            return Err(Error::Unknown("PHY Address"));
        }
        /* Set up Op-code, Phy Address, and register offset in the MDI
         * Control register.  The MAC will take care of interfacing with the
         * PHY to retrieve the desired data.
         */
        let mut mdic = ((value as u32)
            | (offset << MDIC_REG_SHIFT)
            | (self.phy.addr << MDIC_PHY_SHIFT)
            | (MDIC_OP_WRITE));

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

    pub(crate) fn config_fc_after_link_up(&mut self) -> Result {
        self.get_speed_and_duplex_copper();

        if self.mac.autoneg_failed {
        } else {
            self.force_mac_fc()?;
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

            if (self.status.link_duplex == HALF_DUPLEX) {
                self.fc.current_mode = FcMode::None;
            }

            /* Now we call a subroutine to actually force the MAC
             * controller to use the correct flow control settings.
             */
            self.force_mac_fc()?;
        }

        Ok(())
    }

    fn copper_link_autoneg(&mut self) -> Result {
        /* Perform some bounds checking on the autoneg advertisement
         * parameter.
         */
        self.phy.autoneg_advertised &= self.phy.autoneg_mask;

        /* If autoneg_advertised is zero, we assume it was not defaulted
         * by the calling code so we set to advertise full capability.
         */
        if self.phy.autoneg_advertised == 0 {
            self.phy.autoneg_advertised = self.phy.autoneg_mask;
        }

        info!("Reconfiguring auto-neg advertisement params");
        self.phy_setup_autoneg()?;

        info!("Restarting Auto-Neg");

        /* Restart auto-negotiation by setting the Auto Neg Enable bit and
         * the Auto Neg Restart bit in the PHY control register.
         */
        let mut phy_ctrl = self.e1e_rphy(MII_BMCR)?;

        phy_ctrl |= (BMCR_ANENABLE | BMCR_ANRESTART);
        self.e1e_wphy(MII_BMCR, phy_ctrl)?;

        /* Does the user want to wait for Auto-Neg to complete here, or
         * check at a later time (for example, callback routine).
         */
        if (self.phy.autoneg_wait_to_complete) {
            let r = self.wait_autoneg();
            if r.is_err() {
                info!("Error while waiting for autoneg to complete");
                r?;
            }
        }

        info!("Auto-Neg Complete");
        self.mac.get_link_status = true;

        Ok(())
    }

    /**
     *  e1000_set_master_slave_mode - Setup PHY for Master/slave mode
     *  @hw: pointer to the HW structure
     *
     *  Sets up Master/slave mode
     **/
    pub(crate) fn set_master_slave_mode(&mut self) -> Result {
        /* Resolve Master/Slave mode */
        let mut phy_data = self.e1e_rphy(MII_CTRL1000)?;

        /* load defaults for future use */
        self.phy.original_ms_type = if (phy_data & CTL1000_ENABLE_MASTER) > 0 {
            if (phy_data & CTL1000_AS_MASTER) > 0 {
                MsType::ForceMaster
            } else {
                MsType::ForceSlave
            }
        } else {
            MsType::Auto
        };

        match self.phy.ms_type {
            MsType::HwDefault => {}
            MsType::ForceMaster => phy_data |= (CTL1000_ENABLE_MASTER | CTL1000_AS_MASTER),
            MsType::ForceSlave => {
                phy_data |= CTL1000_ENABLE_MASTER;
                phy_data &= !(CTL1000_AS_MASTER);
            }
            MsType::Auto => phy_data &= !CTL1000_ENABLE_MASTER,
        }

        return self.e1e_wphy(MII_CTRL1000, phy_data);
    }
    fn force_mac_fc(&mut self) -> Result {
        let mut ctrl = self.er32(CTRL);

        /* Because we didn't get link via the internal auto-negotiation
         * mechanism (we either forced link or we got link via PHY
         * auto-neg), we have to manually enable/disable transmit an
         * receive flow control.
         *
         * The "Case" statement below enables/disable flow control
         * according to the "hw->fc.current_mode" parameter.
         *
         * The possible values of the "fc" parameter are:
         *      0:  Flow control is completely disabled
         *      1:  Rx flow control is enabled (we can receive pause
         *          frames but not send pause frames).
         *      2:  Tx flow control is enabled (we can send pause frames
         *          but we do not receive pause frames).
         *      3:  Both Rx and Tx flow control (symmetric) is enabled.
         *  other:  No other values should be possible at this point.
         */
        let mode = self.fc.current_mode;
        info!("hw->fc.current_mode = {:?}\n", mode);

        match mode {
            FcMode::None => ctrl &= (!(CTRL_TFCE | CTRL_RFCE)),
            FcMode::RxPause => {
                ctrl &= !CTRL_TFCE;
                ctrl |= CTRL_RFCE;
            }
            FcMode::TxPause => {
                ctrl &= !CTRL_RFCE;
                ctrl |= CTRL_TFCE;
            }
            FcMode::Full => ctrl |= (CTRL_TFCE | CTRL_RFCE),
            FcMode::Default => {
                error!("Flow control param set incorrectly");
                return Err(Error::Config("Flow control param set incorrectly"));
            }
        }

        self.ew32(CTRL, ctrl);

        return Ok(());
    }

    /**
     *  e1000_wait_autoneg - Wait for auto-neg completion
     *  @hw: pointer to the HW structure
     *
     *  Waits for auto-negotiation to complete or for the auto-negotiation time
     *  limit to expire, which ever happens first.
     **/
    fn wait_autoneg(&mut self) -> Result {
        /* Break after autoneg completes or PHY_AUTO_NEG_LIMIT expires. */
        for _ in 0..45 {
            let mut phy_status = self.e1e_rphy(MII_BMSR)?;
            phy_status = self.e1e_rphy(MII_BMSR)?;

            if (phy_status & BMSR_ANEGCOMPLETE) > 0 {
                break;
            }
            self.udelay(100 * 1000 * 1000);
        }

        /* PHY_AUTO_NEG_TIME expiration doesn't guarantee auto-negotiation
         * has completed.
         */
        Ok(())
    }
    pub(crate) fn print_status(&mut self) {
        let status = self.er32(STATUS);
        let ctrl = self.er32(CTRL);

        let speed = if status & STATUS_SPEED_10 > 0 {
            "10Mbps"
        } else if status & STATUS_SPEED_100 > 0 {
            "100Mbps"
        } else {
            "1000Mbps"
        };

        let duplex = if status & STATUS_FD > 0 {
            "Full"
        } else {
            "Half"
        };

        info!(
            "MAC Status: {:#X} Duplex {}, LU {}, Flow Control {}, Speed {}",
            status,
            duplex,
            (status & STATUS_LU) != 0,
            if ((ctrl & CTRL_TFCE) > 0) && ((ctrl & CTRL_RFCE) > 0) {
                "Rx/Tx"
            } else if (ctrl & CTRL_RFCE) > 0 {
                "Rx"
            } else if (ctrl & CTRL_TFCE) > 0 {
                "Tx"
            } else {
                "None"
            },
            speed,
        );
    }

    /**
     *  e1000e_config_collision_dist_generic - Configure collision distance
     *  @hw: pointer to the HW structure
     *
     *  Configures the collision distance to the default value and is used
     *  during link setup.
     **/
    pub(crate) fn config_collision_dist_generic(&mut self) {
        let mut tctl = self.er32(TCTL);

        tctl &= !TCTL_COLD;
        tctl |= COLLISION_DISTANCE << COLD_SHIFT;

        self.ew32(TCTL, tctl);
        self.e1e_flush();
    }
    pub(crate) fn setup_copper_link(&mut self) -> Result {
        if self.mac.autoneg {
            self.copper_link_autoneg()?;
        } else {
            info!("Forcing Speed and Duplex");
        }

        self.print_status();

        let link = self.phy_has_link_generic(10, 10)?;

        if link {
            info!("Valid link established!!!\n");
            self.config_collision_dist_generic();
            self.config_fc_after_link_up();
        } else {
            info!("Unable to establish link!!!\n");
        }

        Ok(())
    }
    pub(crate) fn phy_has_link_generic(
        &mut self,
        iterations: u32,
        usec_interval: u32,
    ) -> Result<bool> {
        let mut success = false;

        for i in 0..iterations {
            if self.e1e_rphy(MII_BMSR).is_err() {
                self.udelay(usec_interval);
            }

            let phy_status = self.e1e_rphy(MII_BMSR)?;
            if phy_status & BMSR_LSTATUS != 0 {
                success = true;
                break;
            }

            self.udelay(usec_interval);
        }
        Ok(success)
    }
    fn phy_setup_autoneg(&mut self) -> Result {
        let mut mii_1000t_ctrl_reg = 0;
        let mut mii_autoneg_adv_reg = 0;
        let current_mode = self.fc.current_mode;

        self.phy.autoneg_advertised &= self.phy.autoneg_mask;

        /* Read the MII Auto-Neg Advertisement Register (Address 4). */
        mii_autoneg_adv_reg = self.e1e_rphy(MII_ADVERTISE)?;

        if (self.phy.autoneg_mask & ADVERTISE_1000_FULL) > 0 {
            /* Read the MII 1000Base-T Control Register (Address 9). */
            mii_1000t_ctrl_reg = self.e1e_rphy(MII_CTRL1000)?;
        }

        /* Need to parse both autoneg_advertised and fc and set up
         * the appropriate PHY registers.  First we will parse for
         * autoneg_advertised software override.  Since we can advertise
         * a plethora of combinations, we need to check each bit
         * individually.
         */

        /* First we clear all the 10/100 mb speed bits in the Auto-Neg
         * Advertisement Register (Address 4) and the 1000 mb speed bits in
         * the  1000Base-T Control Register (Address 9).
         */
        mii_autoneg_adv_reg &=
            !(ADVERTISE_100FULL | ADVERTISE_100HALF | ADVERTISE_10FULL | ADVERTISE_10HALF);
        mii_1000t_ctrl_reg &= !(ADVERTISE_1000HALF | ADVERTISE_1000FULL);

        info!("autoneg_advertised {:#X}", self.phy.autoneg_advertised);

        /* Do we want to advertise 10 Mb Half Duplex? */
        if (self.phy.autoneg_advertised & ADVERTISE_10_HALF) > 0 {
            info!("Advertise 10mb Half duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_10HALF;
        }

        /* Do we want to advertise 10 Mb Full Duplex? */
        if (self.phy.autoneg_advertised & ADVERTISE_10_FULL) > 0 {
            info!("Advertise 10mb Full duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_10FULL;
        }

        /* Do we want to advertise 100 Mb Half Duplex? */
        if (self.phy.autoneg_advertised & ADVERTISE_100_HALF) > 0 {
            info!("Advertise 100mb Half duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_100HALF;
        }

        /* Do we want to advertise 100 Mb Full Duplex? */
        if (self.phy.autoneg_advertised & ADVERTISE_100_FULL) > 0 {
            info!("Advertise 100mb Full duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_100FULL;
        }

        /* We do not allow the Phy to advertise 1000 Mb Half Duplex */
        if (self.phy.autoneg_advertised & ADVERTISE_1000_HALF) > 0 {
            info!("Advertise 1000mb Half duplex request denied!\n");
        }
        /* Do we want to advertise 1000 Mb Full Duplex? */
        if (self.phy.autoneg_advertised & ADVERTISE_1000_FULL) > 0 {
            info!("Advertise 1000mb Full duplex\n");
            mii_1000t_ctrl_reg |= ADVERTISE_1000FULL;
        }

        /* Check for a software override of the flow control settings, and
         * setup the PHY advertisement registers accordingly.  If
         * auto-negotiation is enabled, then software will have to set the
         * "PAUSE" bits to the correct value in the Auto-Negotiation
         * Advertisement Register (MII_ADVERTISE) and re-start auto-
         * negotiation.
         *
         * The possible values of the "fc" parameter are:
         *      0:  Flow control is completely disabled
         *      1:  Rx flow control is enabled (we can receive pause frames
         *          but not send pause frames).
         *      2:  Tx flow control is enabled (we can send pause frames
         *          but we do not support receiving pause frames).
         *      3:  Both Rx and Tx flow control (symmetric) are enabled.
         *  other:  No software override.  The flow control configuration
         *          in the EEPROM is used.
         */
        match current_mode {
            FcMode::None => {
                /* Flow control (Rx & Tx) is completely disabled by a
                 * software over-ride.
                 */
                mii_autoneg_adv_reg &= !(ADVERTISE_PAUSE_ASYM | ADVERTISE_PAUSE_CAP);
                self.phy.autoneg_advertised &= !(ADVERTISED_Pause | ADVERTISED_Asym_Pause);
            }
            FcMode::RxPause => {
                /* Rx Flow control is enabled, and Tx Flow control is
                 * disabled, by a software over-ride.
                 *
                 * Since there really isn't a way to advertise that we are
                 * capable of Rx Pause ONLY, we will advertise that we
                 * support both symmetric and asymmetric Rx PAUSE.  Later
                 * (in e1000e_config_fc_after_link_up) we will disable the
                 * hw's ability to send PAUSE frames.
                 */
                mii_autoneg_adv_reg |= (ADVERTISE_PAUSE_ASYM | ADVERTISE_PAUSE_CAP);
                self.phy.autoneg_advertised |= (ADVERTISED_Pause | ADVERTISED_Asym_Pause);
            }
            FcMode::TxPause => {
                /* Tx Flow control is enabled, and Rx Flow control is
                 * disabled, by a software over-ride.
                 */
                mii_autoneg_adv_reg |= ADVERTISE_PAUSE_ASYM;
                mii_autoneg_adv_reg &= !ADVERTISE_PAUSE_CAP;
                self.phy.autoneg_advertised |= ADVERTISED_Asym_Pause;
                self.phy.autoneg_advertised &= !ADVERTISED_Pause;
            }
            FcMode::Full => {
                /* Flow control (both Rx and Tx) is enabled by a software
                 * over-ride.
                 */
                mii_autoneg_adv_reg |= (ADVERTISE_PAUSE_ASYM | ADVERTISE_PAUSE_CAP);
                self.phy.autoneg_advertised |= (ADVERTISED_Pause | ADVERTISED_Asym_Pause);
            }
            _ => {
                error!("Flow control param set incorrectly\n");
                return Err(Error::Config("Flow control param set incorrectly"));
            }
        }
        self.e1e_wphy(MII_ADVERTISE, mii_autoneg_adv_reg)?;

        info!("Auto-Neg Advertising {:#X}", mii_autoneg_adv_reg);

        if (self.phy.autoneg_mask & ADVERTISE_1000_FULL) > 0 {
            self.e1e_wphy(MII_CTRL1000, mii_1000t_ctrl_reg);
        }

        Ok(())
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

        let status = &mut self.status;
        status.link_speed = speed;
        status.link_duplex = duplex;

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
    pub(crate) fn has_link(&mut self) -> bool {
        let mut link_active = false;

        match self.phy.media_type {
            MediaType::media_type_copper => {
                if self.mac.get_link_status {
                    let _ = self.check_for_copper_link();
                    link_active = !self.mac.get_link_status;
                } else {
                    link_active = true;
                }
            }
            _ => todo!(),
        }

        link_active
    }

    fn check_for_copper_link(&mut self) -> Result {
        if !self.mac.get_link_status {
            return Ok(());
        }
        self.mac.get_link_status = false;

        let link = self.phy_has_link_generic(1, 0)?;
        if link {
            info!("link up!");
            return Ok(());
        }

        let mut mac_reg = self.er32(FEXTNVM4);
        mac_reg &= !FEXTNVM4_BEACON_DURATION_MASK;
        mac_reg |= FEXTNVM4_BEACON_DURATION_8USEC;
        self.ew32(FEXTNVM4, mac_reg);
        self.ew32(0xf8, 0x40000000);
        self.ew32(FEXTNVM6, 0xe00);
        self.ew32(0xc8, 0x10);
        self.ew32(0xd0, 0x40009d);

        self.mac.get_link_status = true;
        Ok(())
    }

    fn write_smbus_addr(&mut self) -> Result {
        let mut strap = self.er32(STRAP);
        let mut freq = (strap & STRAP_SMT_FREQ_MASK) >> STRAP_SMT_FREQ_SHIFT;

        strap &= STRAP_SMBUS_ADDRESS_MASK;

        let mut phy_data = self.read_phy_reg_hv(HV_SMB_ADDR)?;

        phy_data &= !HV_SMB_ADDR_MASK;
        phy_data |= (strap >> STRAP_SMBUS_ADDRESS_SHIFT) as u16;
        phy_data |= HV_SMB_ADDR_PEC_EN | HV_SMB_ADDR_VALID;

        if self.phy.kind == PhyType::I217 {
            /* Restore SMBus frequency */
            freq -= 1;
            if freq > 0 {
                phy_data &= !HV_SMB_ADDR_FREQ_MASK;
                phy_data |= ((freq & BIT(0)) << HV_SMB_ADDR_FREQ_LOW_SHIFT) as u16;
                phy_data |= ((freq & BIT(1)) << (HV_SMB_ADDR_FREQ_HIGH_SHIFT - 1)) as u16;
            } else {
                error!("Unsupported SMB frequency in PHY");
            }
        }

        return self.write_phy_reg_hv(HV_SMB_ADDR, phy_data);
    }

    fn get_phy_lock_ich8lan(&self) -> Result<SWFlagIch8lan<'_, K>> {
        SWFlagIch8lan::new(self)
    }

    fn acquire(&self) -> Result {
        let mut extcnf_ctrl = 0;
        let mut timeout = true;
        for _ in 0..100 {
            extcnf_ctrl = self.er32(EXTCNF_CTRL);

            if extcnf_ctrl & EXTCNF_CTRL_SWFLAG == 0 {
                timeout = false;
                break;
            }

            self.udelay(1000);
        }

        if timeout {
            error!("SW has already locked the resource.");
            return Err(Error::Config("SW has already locked the resource."));
        }
        extcnf_ctrl |= EXTCNF_CTRL_SWFLAG;
        self.ew32(EXTCNF_CTRL, extcnf_ctrl);

        timeout = true;

        for _ in 0..20 {
            extcnf_ctrl = self.er32(EXTCNF_CTRL);

            if extcnf_ctrl & EXTCNF_CTRL_SWFLAG > 0 {
                timeout = false;
                break;
            }

            self.udelay(1000);
        }

        if timeout {
            error!(
                "Failed to acquire the semaphore, FW or HW has it: FWSM={:#x} EXTCNF_CTRL={:#x})",
                self.er32(FWSM),
                extcnf_ctrl
            );
            extcnf_ctrl &= !EXTCNF_CTRL_SWFLAG;
            self.ew32(EXTCNF_CTRL, extcnf_ctrl);

            return Err(Error::Config("Failed to acquire the semaphore"));
        }

        Ok(())
    }

    fn release(&self) {
        let mut extcnf_ctrl = self.er32(EXTCNF_CTRL);

        if (extcnf_ctrl & EXTCNF_CTRL_SWFLAG) > 0 {
            extcnf_ctrl &= !EXTCNF_CTRL_SWFLAG;
            self.ew32(EXTCNF_CTRL, extcnf_ctrl);
        } else {
            error!("Semaphore unexpectedly released by sw/fw/hw");
        }
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
            // let mmio = kfun.bar_n_remap(0);
            let layout = Layout::new::<Hw<K>>();
            let ptr = alloc::alloc(layout) as *mut Hw<K>;
            let hw = &mut *ptr;
            hw.kernel = kfun;
            // hw.mmio = mmio;
            hw.flash = None;
            hw.status = Default::default();
            hw.phy = Default::default();
            hw.mac = Default::default();
            hw.nvm = Default::default();
            hw.fc = Default::default();
            hw.mng_cookie = Default::default();
            hw.is_down = false;
            hw.mac.kind = board.mac;
            // hw.flags = board.flags;
            // hw.flags2 = board.flags2;
            hw.variant = board.variant;
            hw.status.max_hw_frame_size = board.max_hw_frame_size;
            hw.status.pba = board.pba;
            hw.is_print_ew = false;

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
            DevSpec::Ich8lan(i) => i.get_bus_info(hw),
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

pub(crate) struct SWFlagIch8lan<'a, K: KernelFunc> {
    hw: &'a Hw<K>,
}

impl<'a, K: KernelFunc> SWFlagIch8lan<'a, K> {
    fn new(hw: &'a Hw<K>) -> Result<Self> {
        Ok(Self { hw })
    }
}

impl<'a, K: KernelFunc> Drop for SWFlagIch8lan<'a, K> {
    fn drop(&mut self) {
        let mut extcnf_ctrl = self.hw.er32(EXTCNF_CTRL);

        if (extcnf_ctrl & EXTCNF_CTRL_SWFLAG) > 0 {
            extcnf_ctrl &= !EXTCNF_CTRL_SWFLAG;
            self.hw.ew32(EXTCNF_CTRL, extcnf_ctrl);
        } else {
            error!("Semaphore unexpectedly released by sw/fw/hw");
        }
    }
}
