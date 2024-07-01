#![allow(non_snake_case)]

use core::{
    cell::{RefCell, UnsafeCell},
    sync::atomic::AtomicBool,
    time::Duration,
};

use super::{
    reg::*,
    variant::{e82571, ich8lan, Variant},
};
use crate::e1000::lock::SemaphoreBackend;
use crate::{defines::*, err::*, kfn, mac::MacType};
use crate::{hw::MsType, MediaType, PhyType};

// IGP01E1000 Specific Registers
pub(crate) const PHY_PORT_CONFIG: u16 = 0x10; // Port Config
pub(crate) const PHY_PORT_STATUS: u16 = 0x11; // Status
pub(crate) const PHY_PORT_CTRL: u16 = 0x12; // Control
pub(crate) const PHY_LINK_HEALTH: u16 = 0x13; // PHY Link Health
pub(crate) const PHY_POWER_MGMT_IGP02: u16 = 0x19; // Power Management
pub(crate) const PHY_PAGE_SELECT_IGP01: u16 = 0x1F; // Page Select
pub(crate) const BM_PHY_PAGE_SELECT: u32 = 22; // Page Select for BM
pub(crate) const PHY_REG_MASK: u16 = 0x1F;

// BM/HV Specific Registers
pub(crate) const BM_PORT_CTRL_PAGE: u32 = 769;
pub(crate) const BM_WUC_PAGE: u16 = 800;
pub(crate) const BM_WUC_ADDRESS_OPCODE: u8 = 0x11;
pub(crate) const BM_WUC_DATA_OPCODE: u8 = 0x12;
pub(crate) const BM_WUC_ENABLE_PAGE: u32 = BM_PORT_CTRL_PAGE;
pub(crate) const BM_WUC_ENABLE_REG: u16 = 17;

// MDI Control
pub(crate) const MDIC_REG_MASK: u32 = 0x001F0000;
pub(crate) const MDIC_REG_SHIFT: u32 = 16;
pub(crate) const MDIC_PHY_SHIFT: u32 = 21;
pub(crate) const MDIC_OP_WRITE: u32 = 0x04000000;
pub(crate) const MDIC_OP_READ: u32 = 0x08000000;
pub(crate) const MDIC_READY: u32 = 0x10000000;
pub(crate) const MDIC_ERROR: u32 = 0x40000000;

pub(crate) const HV_INTC_FC_PAGE_START: u16 = 768;
pub(crate) const I82578_ADDR_REG: u32 = 29;
pub(crate) const I82577_ADDR_REG: u32 = 16;
pub(crate) const I82577_CFG_REG: u32 = 22;
pub(crate) const I82577_CFG_ASSERT_CRS_ON_TX: u16 = 1 << 15;
pub(crate) const I82577_CFG_ENABLE_DOWNSHIFT: u16 = 3 << 10; // auto downshift
pub(crate) const I82577_CTRL_REG: u32 = 23;

// 82577 specific PHY registers
pub(crate) const I82577_PHY_CTRL_2: u32 = 18;
pub(crate) const I82577_PHY_LBK_CTRL: u32 = 19;
pub(crate) const I82577_PHY_STATUS_2: u32 = 26;
pub(crate) const I82577_PHY_DIAG_STATUS: u32 = 31;

// I82577 PHY Status 2
pub(crate) const I82577_PHY_STATUS2_REV_POLARITY: u16 = 0x0400;
pub(crate) const I82577_PHY_STATUS2_MDIX: u16 = 0x0800;
pub(crate) const I82577_PHY_STATUS2_SPEED_MASK: u16 = 0x0300;
pub(crate) const I82577_PHY_STATUS2_SPEED_1000MBPS: u16 = 0x0200;

// I82577 PHY Control 2
pub(crate) const I82577_PHY_CTRL2_MANUAL_MDIX: u16 = 0x0200;
pub(crate) const I82577_PHY_CTRL2_AUTO_MDI_MDIX: u16 = 0x0400;
pub(crate) const I82577_PHY_CTRL2_MDIX_CFG_MASK: u16 = 0x0600;

// I82577 PHY Diagnostics Status
pub(crate) const I82577_DSTATUS_CABLE_LENGTH: u16 = 0x03FC;
pub(crate) const I82577_DSTATUS_CABLE_LENGTH_SHIFT: u8 = 2;

pub(crate) const MAX_PHY_ADDR: u32 = 8;

pub(crate) const BM_WUC_ENABLE_BIT: u16 = 1 << 2;
pub(crate) const BM_WUC_HOST_WU_BIT: u16 = 1 << 4;
pub(crate) const BM_WUC_ME_WU_BIT: u16 = 1 << 5;

pub(crate) const fn PHY_REG(page: u32, reg: u32) -> u32 {
    // Use bitwise shift and AND operation to combine page and register
    (page << PHY_PAGE_SHIFT) | (reg & MAX_PHY_REG_ADDRESS)
}

pub(crate) const BM_PORT_GEN_CFG: u32 = PHY_REG(BM_PORT_CTRL_PAGE, 17);

pub(crate) const PHY_UPPER_SHIFT: u32 = 21;

pub(crate) fn BM_PHY_REG(page: u16, reg: u16) -> u32 {
    // Combine page and register using bitwise operations
    ((reg as u32 & MAX_PHY_REG_ADDRESS) as u32)
        | (((page & 0xFFFF) as u32) << PHY_PAGE_SHIFT)
        | (((reg as u32 & !MAX_PHY_REG_ADDRESS) as u32) << (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT))
}

pub(crate) fn BM_PHY_REG_PAGE(offset: u32) -> u16 {
    // Extract the page from the given offset
    ((offset >> PHY_PAGE_SHIFT) as u16) & 0xFFFF
}

pub(crate) fn BM_PHY_REG_NUM(offset: u32) -> u32 {
    // Extract the register number from the given offset
    (offset & MAX_PHY_REG_ADDRESS)
        | ((offset >> (PHY_UPPER_SHIFT - PHY_PAGE_SHIFT)) & !MAX_PHY_REG_ADDRESS)
}

pub(crate) const CV_SMB_CTRL: u32 = PHY_REG(769, 23);

pub(crate) const CV_SMB_CTRL_FORCE_SMBUS: u16 = 0x0001;

/* KMRN Mode Control */
pub(crate) const HV_KMRN_MODE_CTRL: u32 = PHY_REG(769, 16);
pub(crate) const HV_KMRN_MDIO_SLOW: u16 = 0x0400;

/* SMBus Address Phy Register */
pub(crate) const HV_SMB_ADDR: u32 = PHY_REG(768, 26);
pub(crate) const HV_SMB_ADDR_MASK: u16 = 0x007F;
pub(crate) const HV_SMB_ADDR_PEC_EN: u16 = 0x0200;
pub(crate) const HV_SMB_ADDR_VALID: u16 = 0x0080;
pub(crate) const HV_SMB_ADDR_FREQ_MASK: u16 = 0x1100;
pub(crate) const HV_SMB_ADDR_FREQ_LOW_SHIFT: u32 = 8;
pub(crate) const HV_SMB_ADDR_FREQ_HIGH_SHIFT: u32 = 12;
pub(crate) const HV_OEM_BITS: u32 = PHY_REG(768, 25);
// Assuming these are meant to be bit masks for register fields
pub(crate) const HV_OEM_BITS_LPLU: u16 = 0x0004; // Low Power Link Up
pub(crate) const HV_OEM_BITS_GBE_DIS: u16 = 0x0040; // Gigabit Disable
pub(crate) const HV_OEM_BITS_RESTART_AN: u16 = 0x0400; // Restart Auto-negotiation

// For the register definition, if you're representing a struct or enum to hold register addresses and fields,
// it would depend on how you structure your hardware abstraction layer. However, given just the address and bit position,
// here's a simplified placeholder using tuples, assuming you have a way to work with register addresses and bitfields.
// A more sophisticated approach would involve a Register or Bitfield type with methods for accessing/modifying fields.
pub(crate) const HV_OEM_BITS_REGISTER: u32 = PHY_REG(768, 25); // Tuple representing the register address and bit position
pub(crate) const HV_PM_CTRL: u32 = PHY_REG(770, 17);

pub(crate) const I82579_LPI_CTRL: u32 = PHY_REG(772, 20);

pub(crate) const I82579_LPI_CTRL_ENABLE_MASK: u16 = 0x6000;

#[derive(Debug)]
pub(crate) struct PhyInfo2 {
    pub mac_kind: MacType,
    pub kind: PhyType,
    pub reg: Reg,
    pub ops: PhyOps,

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

    pub ms_type: MsType,
    pub original_ms_type: MsType,
    pub access_shared_resource: AtomicBool,
}

impl PhyInfo2 {
    pub fn acquire(&self) -> Result<PhyGuard<'_>> {
        match &self.ops.semaphore {
            PhySemaphore::Ich8lan => todo!(),
            PhySemaphore::B82571 => e82571::PhySemaphore::new(self.reg).acquire(),
        }?;
        unsafe { Ok(PhyGuard { phy: self }) }
    }

    pub fn e1e_rphy_locked(&mut self, offset: u32) -> Result<u16> {
        match &self.ops.read_write {
            PhyReadWrite::Bm2 => self.read_phy_reg_bm2(offset),
            PhyReadWrite::Hv => todo!(),
        }
    }
    pub fn e1e_wphy_locked(&mut self, offset: u32, value: u16) -> Result {
        match &self.ops.read_write {
            PhyReadWrite::Bm2 => self.write_phy_reg_bm2(offset, value),
            PhyReadWrite::Hv => todo!(),
        }
    }
    pub fn e1e_rphy(&self, offset: u32) -> Result<u16> {
        let mut g = self.acquire()?;
        g.e1e_rphy_locked(offset)
    }
    pub fn e1e_wphy(&self, offset: u32, value: u16) -> Result {
        let mut g = self.acquire()?;
        g.e1e_wphy_locked(offset, value)
    }
    fn read_phy_reg_bm2(&mut self, offset: u32) -> Result<u16> {
        let page = (offset >> IGP_PAGE_SHIFT) as u16;

        /* Page 800 works differently than the rest so it has its own func */
        if (page == BM_WUC_PAGE) {
            // TODO
            // ret_val = e1000_access_phy_wakeup_reg_bm(hw, offset, data, true,
            // 					 false);
            // goto release;
        }

        self.addr = 1;

        if (offset > MAX_PHY_MULTI_PAGE_REG) {
            /* Page is shifted left, PHY expects (page x 32) */
            self.write_phy_reg_mdic(BM_PHY_PAGE_SELECT, page)?;
        }

        self.read_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset)
    }
    fn write_phy_reg_bm2(&mut self, offset: u32, value: u16) -> Result {
        let page = (offset >> IGP_PAGE_SHIFT) as u16;

        /* Page 800 works differently than the rest so it has its own func */
        if (page == BM_WUC_PAGE) {
            // TODO
            // ret_val = e1000_access_phy_wakeup_reg_bm(hw, offset, data, true,
            // 					 false);
            // goto release;
        }

        self.addr = 1;

        if (offset > MAX_PHY_MULTI_PAGE_REG) {
            /* Page is shifted left, PHY expects (page x 32) */
            self.write_phy_reg_mdic(BM_PHY_PAGE_SELECT, page)?;
        }

        self.write_phy_reg_mdic(MAX_PHY_REG_ADDRESS & offset, value)
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

        self.addr = 1;

        self.write_phy_reg_mdic(IGP01E1000_PHY_PAGE_SELECT, page)
    }
    pub fn write_phy_reg_mdic(&mut self, offset: u32, value: u16) -> Result {
        if (offset > MAX_PHY_REG_ADDRESS) {
            error!("PHY Address {} is out of range", offset);
            return Err(Error::Phy("PHY Address"));
        }
        /* Set up Op-code, Phy Address, and register offset in the MDI
         * Control register.  The MAC will take care of interfacing with the
         * PHY to retrieve the desired data.
         */
        let mut mdic = ((value as u32)
            | (offset << MDIC_REG_SHIFT)
            | (self.addr << MDIC_PHY_SHIFT)
            | (MDIC_OP_WRITE));

        self.reg.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT * 3 {
            kfn().delay(Duration::from_micros(50));
            mdic = self.reg.er32(MDIC);
            if (mdic & MDIC_READY) > 0 {
                break;
            }
        }

        if (mdic & MDIC_READY) == 0 {
            error!("MDI Read PHY Reg Address {} did not complete", offset);
            return Err(Error::Phy("PHY"));
        }
        if (mdic & MDIC_ERROR) > 0 {
            error!("MDI Read PHY Reg Address {} Error", offset);
            return Err(Error::Phy("PHY"));
        }
        if (((mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT) != offset) {
            error!(
                "MDI Read offset error - requested {}, returned {}",
                offset,
                (mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT,
            );

            return Err(Error::Phy("PHY"));
        }

        /* Allow some time after each MDIC transaction to avoid
         * reading duplicate data in the next MDIC transaction.
         */
        if (self.mac_kind == MacType::e1000_pch2lan) {
            kfn().delay(Duration::from_micros(100));
        }

        Ok(())
    }

    pub fn read_phy_reg_mdic(&mut self, offset: u32) -> Result<u16> {
        if (offset > MAX_PHY_REG_ADDRESS) {
            error!("PHY Address {} is out of range", offset);
            return Err(Error::Phy("PHY Address"));
        }
        let reg = &self.reg;
        /* Set up Op-code, Phy Address, and register offset in the MDI
         * Control register.  The MAC will take care of interfacing with the
         * PHY to retrieve the desired data.
         */
        let mut mdic =
            ((offset << MDIC_REG_SHIFT) | (self.addr << MDIC_PHY_SHIFT) | (MDIC_OP_READ));

        reg.ew32(MDIC, mdic);

        for _ in 0..GEN_POLL_TIMEOUT * 3 {
            kfn().delay(Duration::from_micros(50));
            mdic = reg.er32(MDIC);
            if (mdic & MDIC_READY) > 0 {
                break;
            }
        }

        if (mdic & MDIC_READY) == 0 {
            error!("MDI Read PHY Reg Address {} did not complete", offset);
            return Err(Error::Phy("MDI Read PHY Reg Address did not complete"));
        }
        if (mdic & MDIC_ERROR) > 0 {
            error!("MDI Read PHY Reg Address {} Error", offset);
            return Err(Error::Phy("MDI Read PHY Reg Address Error"));
        }
        if (((mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT) != offset) {
            error!(
                "MDI Read offset error - requested {}, returned {}",
                offset,
                (mdic & MDIC_REG_MASK) >> MDIC_REG_SHIFT,
            );

            return Err(Error::Phy("PHY"));
        }

        /* Allow some time after each MDIC transaction to avoid
         * reading duplicate data in the next MDIC transaction.
         */
        if (self.mac_kind == MacType::e1000_pch2lan) {
            kfn().delay(Duration::from_micros(100));
        }

        Ok((mdic & 0xFFFF) as u16)
    }

    pub(crate) fn copper_options(&mut self) {
        if self.media_type == MediaType::media_type_copper {
            self.mdix = AUTO_ALL_MODES;
            self.disable_polarity_correction = false;
            self.ms_type = MsType::HwDefault;
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PhyOps {
    pub semaphore: PhySemaphore,
    pub read_write: PhyReadWrite,
}

#[derive(Debug, Clone)]
pub(crate) enum PhySemaphore {
    Ich8lan,
    B82571,
}

#[derive(Debug, Clone)]
pub(crate) enum PhyReadWrite {
    Bm2,
    Hv,
}

pub(crate) struct PhyGuard<'a> {
    phy: &'a PhyInfo2,
}

impl core::ops::Deref for PhyGuard<'_> {
    type Target = PhyInfo2;

    fn deref(&self) -> &Self::Target {
        self.phy
    }
}

impl core::ops::DerefMut for PhyGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr = self.phy as *const PhyInfo2;
            let ptr = ptr as *mut PhyInfo2;
            &mut *ptr
        }
    }
}

impl Drop for PhyGuard<'_> {
    fn drop(&mut self) {
        match &self.phy.ops.semaphore {
            PhySemaphore::Ich8lan => todo!(),
            PhySemaphore::B82571 => e82571::PhySemaphore::new(self.phy.reg).release(),
        }
    }
}
