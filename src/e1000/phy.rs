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
use crate::{
    defines::*,
    err::*,
    kfn,
    mac::{MacInfo, MacType},
    E1000,
};
use crate::{e1000::lock::SemaphoreBackend, hw::FcMode};
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

    pub(crate) fn phy_has_link_generic(
        &mut self,
        iterations: u32,
        usec_interval: u32,
    ) -> Result<bool> {
        let mut success = false;

        for i in 0..iterations {
            if self.e1e_rphy(MII_BMSR).is_err() {
                kfn().delay(Duration::from_micros(usec_interval as u64));
            }

            let phy_status = self.e1e_rphy(MII_BMSR)?;
            if phy_status & BMSR_LSTATUS != 0 {
                success = true;
                break;
            }

            kfn().delay(Duration::from_micros(usec_interval as u64));
        }
        Ok(success)
    }
    pub(crate) fn setup_copper_link_82571(&mut self) -> Result {
        let mut ctrl = self.reg.er32(CTRL);
        ctrl |= CTRL_SLU;
        ctrl &= !(CTRL_FRCSPD | CTRL_FRCDPX);
        self.reg.ew32(CTRL, ctrl);
        self.reg.e1e_flush();

        Ok(())
    }
    pub(crate) fn setup_copper_link_pch_lpt(&mut self) -> Result {
        let mut ctrl = self.reg.er32(CTRL);
        ctrl |= CTRL_SLU;
        ctrl &= !(CTRL_FRCSPD | CTRL_FRCDPX);
        self.reg.ew32(CTRL, ctrl);
        self.reg.e1e_flush();

        self.copper_link_setup_82577()
    }

    pub(crate) fn set_eee_pchlan(&self) -> Result {
        let mut lpa;
        let mut pcs_status;
        let mut adv_addr;

        match self.kind {
            PhyType::T82579 => {
                lpa = I82579_EEE_LP_ABILITY;
                pcs_status = I82579_EEE_PCS_STATUS;
                adv_addr = I82579_EEE_ADVERTISEMENT;
            }
            PhyType::I217 => {
                lpa = I217_EEE_LP_ABILITY;
                pcs_status = I217_EEE_PCS_STATUS;
                adv_addr = I217_EEE_ADVERTISEMENT;
            }
            _ => return Ok(()),
        }

        let mut lpi_ctrl = self.e1e_rphy(I82579_LPI_CTRL)?;
        /* Clear bits that enable EEE in various speeds */
        lpi_ctrl &= !I82579_LPI_CTRL_ENABLE_MASK;

        self.e1e_wphy(I82579_LPI_CTRL, lpi_ctrl)
    }
    fn copper_link_setup_82577(&mut self) -> Result {
        let mut phy_data = self.e1e_rphy(I82577_CFG_REG)?;

        /* Enable CRS on Tx. This must be set for half-duplex operation. */
        phy_data |= I82577_CFG_ASSERT_CRS_ON_TX;

        /* Enable downshift */
        phy_data |= I82577_CFG_ENABLE_DOWNSHIFT;

        self.e1e_wphy(I82577_CFG_REG, phy_data)?;

        /* Set MDI/MDIX mode */
        phy_data = self.e1e_rphy(I82577_PHY_CTRL_2)?;
        phy_data &= !I82577_PHY_CTRL2_MDIX_CFG_MASK;
        /* Options:
         *   0 - Auto (default)
         *   1 - MDI mode
         *   2 - MDI-X mode
         */
        match self.mdix {
            1 => {}
            2 => {
                phy_data |= I82577_PHY_CTRL2_MANUAL_MDIX;
            }
            _ => {
                phy_data |= I82577_PHY_CTRL2_AUTO_MDI_MDIX;
            }
        }
        self.e1e_wphy(I82577_PHY_CTRL_2, phy_data)?;

        self.set_master_slave_mode()
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
        self.original_ms_type = if (phy_data & CTL1000_ENABLE_MASTER) > 0 {
            if (phy_data & CTL1000_AS_MASTER) > 0 {
                MsType::ForceMaster
            } else {
                MsType::ForceSlave
            }
        } else {
            MsType::Auto
        };

        match self.ms_type {
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

    pub(crate) fn copper_link_autoneg(&mut self, current_mode: FcMode) -> Result {
        /* Perform some bounds checking on the autoneg advertisement
         * parameter.
         */
        self.autoneg_advertised &= self.autoneg_mask;

        /* If autoneg_advertised is zero, we assume it was not defaulted
         * by the calling code so we set to advertise full capability.
         */
        if self.autoneg_advertised == 0 {
            self.autoneg_advertised = self.autoneg_mask;
        }

        info!("Reconfiguring auto-neg advertisement params");
        self.phy_setup_autoneg(current_mode)?;

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
        if (self.autoneg_wait_to_complete) {
            let r = self.wait_autoneg();
            if r.is_err() {
                info!("Error while waiting for autoneg to complete");
                r?;
            }
        }

        info!("Auto-Neg Complete");
        // self.mac.get_link_status = true;

        Ok(())
    }
    fn phy_setup_autoneg(&mut self, current_mode: FcMode) -> Result {
        let mut mii_1000t_ctrl_reg = 0;
        let mut mii_autoneg_adv_reg = 0;

        self.autoneg_advertised &= self.autoneg_mask;

        /* Read the MII Auto-Neg Advertisement Register (Address 4). */
        mii_autoneg_adv_reg = self.e1e_rphy(MII_ADVERTISE)?;

        if (self.autoneg_mask & ADVERTISE_1000_FULL) > 0 {
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

        info!("autoneg_advertised {:#X}", self.autoneg_advertised);

        /* Do we want to advertise 10 Mb Half Duplex? */
        if (self.autoneg_advertised & ADVERTISE_10_HALF) > 0 {
            info!("Advertise 10mb Half duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_10HALF;
        }

        /* Do we want to advertise 10 Mb Full Duplex? */
        if (self.autoneg_advertised & ADVERTISE_10_FULL) > 0 {
            info!("Advertise 10mb Full duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_10FULL;
        }

        /* Do we want to advertise 100 Mb Half Duplex? */
        if (self.autoneg_advertised & ADVERTISE_100_HALF) > 0 {
            info!("Advertise 100mb Half duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_100HALF;
        }

        /* Do we want to advertise 100 Mb Full Duplex? */
        if (self.autoneg_advertised & ADVERTISE_100_FULL) > 0 {
            info!("Advertise 100mb Full duplex\n");
            mii_autoneg_adv_reg |= ADVERTISE_100FULL;
        }

        /* We do not allow the Phy to advertise 1000 Mb Half Duplex */
        if (self.autoneg_advertised & ADVERTISE_1000_HALF) > 0 {
            info!("Advertise 1000mb Half duplex request denied!\n");
        }
        /* Do we want to advertise 1000 Mb Full Duplex? */
        if (self.autoneg_advertised & ADVERTISE_1000_FULL) > 0 {
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
                self.autoneg_advertised &= !(ADVERTISED_Pause | ADVERTISED_Asym_Pause);
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
                self.autoneg_advertised |= (ADVERTISED_Pause | ADVERTISED_Asym_Pause);
            }
            FcMode::TxPause => {
                /* Tx Flow control is enabled, and Rx Flow control is
                 * disabled, by a software over-ride.
                 */
                mii_autoneg_adv_reg |= ADVERTISE_PAUSE_ASYM;
                mii_autoneg_adv_reg &= !ADVERTISE_PAUSE_CAP;
                self.autoneg_advertised |= ADVERTISED_Asym_Pause;
                self.autoneg_advertised &= !ADVERTISED_Pause;
            }
            FcMode::Full => {
                /* Flow control (both Rx and Tx) is enabled by a software
                 * over-ride.
                 */
                mii_autoneg_adv_reg |= (ADVERTISE_PAUSE_ASYM | ADVERTISE_PAUSE_CAP);
                self.autoneg_advertised |= (ADVERTISED_Pause | ADVERTISED_Asym_Pause);
            }
            _ => {
                error!("Flow control param set incorrectly\n");
                return Err(Error::Config("Flow control param set incorrectly"));
            }
        }
        self.e1e_wphy(MII_ADVERTISE, mii_autoneg_adv_reg)?;

        info!("Auto-Neg Advertising {:#X}", mii_autoneg_adv_reg);

        if (self.autoneg_mask & ADVERTISE_1000_FULL) > 0 {
            self.e1e_wphy(MII_CTRL1000, mii_1000t_ctrl_reg);
        }

        Ok(())
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
            kfn().delay(Duration::from_millis(100));
        }

        /* PHY_AUTO_NEG_TIME expiration doesn't guarantee auto-negotiation
         * has completed.
         */
        Ok(())
    }

    pub(crate) fn read_kmrn_reg(&mut self, offset: u32) -> u16 {
        let mut kmrnctrlsta =
            ((offset << KMRNCTRLSTA_OFFSET_SHIFT) & KMRNCTRLSTA_OFFSET) | KMRNCTRLSTA_REN;
        self.reg.ew32(KMRNCTRLSTA, kmrnctrlsta);
        self.reg.e1e_flush();

        self.reg.er32(KMRNCTRLSTA) as u16
    }
    pub(crate) fn write_kmrn_reg(&mut self, offset: u32, data: u16) {
        let mut kmrnctrlsta =
            ((offset << KMRNCTRLSTA_OFFSET_SHIFT) & KMRNCTRLSTA_OFFSET) | data as u32;
        self.reg.ew32(KMRNCTRLSTA, kmrnctrlsta);
        self.reg.e1e_flush();
        kfn().delay(Duration::from_micros(2));
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
