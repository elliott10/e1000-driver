use super::super::regs::*;
use super::DevSpec;
use super::NvmInfo;
use super::NvmType;
use super::{DevSpecParam, Hw};
use crate::defines::*;
use crate::err::*;
use crate::BoardVariant;
use crate::KernelFunc;
use crate::MacType;
use crate::MediaType;
use crate::PhyInfo;
use crate::E1000;
use alloc::vec::Vec;
use core::mem::size_of;

const E1000_ICH_RAR_ENTRIES: u16 = 7;
const E1000_PCH2_RAR_ENTRIES: u16 = 5; /* RAR[0], SHRA[0-3] */
const E1000_PCH_LPT_RAR_ENTRIES: u16 = 12; /* RAR[0], SHRA[0-10] */

const NVM_SIZE_MULTIPLIER: u32 = 4096; // multiplier for NVMS field
const E1000_FLASH_BASE_ADDR: usize = 0xE000; // offset of NVM access regs
const E1000_CTRL_EXT_NVMVS: u32 = 0x3; // NVM valid sector

// TARC0 constants with bit masks
const E1000_TARC0_CB_MULTIQ_3_REQ: u32 = 0x30000000;
const E1000_TARC0_CB_MULTIQ_2_REQ: u32 = 0x20000000;

const PCIE_ICH8_SNOOP_ALL: u32 = PCIE_NO_SNOOP_ALL;

pub(crate) const ICH_FLASH_GFPREG: usize = 0x0000; // GFP Configuration Register
pub(crate) const ICH_FLASH_HSFSTS: usize = 0x0004; // Host Status and Control Register
pub(crate) const ICH_FLASH_HSFCTL: usize = 0x0006; // Host Control Register
pub(crate) const ICH_FLASH_FADDR: usize = 0x0008; // Flash Address Register
pub(crate) const ICH_FLASH_FDATA0: usize = 0x0010; // Flash Data Register 0
pub(crate) const ICH_FLASH_PR0: usize = 0x0074; // Protected Range Register 0
                                                // Requires up to 10 seconds when MNG might be accessing part.
pub(crate) const ICH_FLASH_READ_COMMAND_TIMEOUT: u32 = 10000000;
pub(crate) const ICH_FLASH_WRITE_COMMAND_TIMEOUT: u32 = 10000000;
pub(crate) const ICH_FLASH_ERASE_COMMAND_TIMEOUT: u32 = 10000000;

// Mask for linear address in flash operations.
pub(crate) const ICH_FLASH_LINEAR_ADDR_MASK: u32 = 0x00FFFFFF;

// Number of times a flash cycle should be repeated.
pub(crate) const ICH_FLASH_CYCLE_REPEAT_COUNT: u32 = 10;

const HSFSTS_FLDESVALID: u16 = 1 << 14;
const HSFSTS_FLCERR: u16 = 1 << 1;
const HSFSTS_DAEL: u16 = 1 << 2;

const HSFSTS_FLCINPROG: u16 = 1 << 5;
const HSFSTS_FLCDONE: u16 = 1;

const HSFCTL_FLCGO: u16 = 1;
const HSFCTL_FLCYCLE_SHIFT: u16 = 1;

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ShadowRam {
    pub(crate) value: u16,
    pub(crate) modified: bool,
}
const E1000_ICH8_SHADOW_RAM_WORDS: usize = 2048;

#[derive(Debug, Clone)]
pub(crate) struct DevSpecIch8lan {
    pub(crate) kmrn_lock_loss_workaround_enabled: bool,
    // pub(crate) shadow_ram: [ShadowRam; E1000_ICH8_SHADOW_RAM_WORDS],
    pub(crate) nvm_k1_enabled: bool,
    pub(crate) eee_disable: bool,
    pub(crate) eee_lp_ability: u16,
    // pub ulp_state: E1000UlpState,
}

impl Default for DevSpecIch8lan {
    fn default() -> Self {
        Self {
            kmrn_lock_loss_workaround_enabled: Default::default(),
            // TODO 会引起内核崩溃
            // shadow_ram: [ShadowRam::default(); E1000_ICH8_SHADOW_RAM_WORDS],
            nvm_k1_enabled: Default::default(),
            eee_disable: Default::default(),
            eee_lp_ability: Default::default(),
        }
    }
}

impl DevSpecIch8lan {
    pub(crate) fn get_variants<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        info!("Ich8lan init");
        self.init_mac_params(hw)?;
        self.init_nvm_params(hw)?;
        Ok(())
    }

    fn init_mac_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        hw.phy.media_type = MediaType::media_type_copper;
        /* Set mta register count */
        hw.mac.mta_reg_count = 32;
        // /* Set rar entry count */
        hw.mac.rar_entry_count = E1000_ICH_RAR_ENTRIES;
        if hw.mac.kind == MacType::e1000_ich8lan {
            hw.mac.rar_entry_count -= 1;
        }

        hw.mac.has_fwsm = true;

        hw.mac.arc_subsystem_valid = false;

        hw.mac.adaptive_ifs = true;

        // // TODO led

        if hw.mac.kind > MacType::e1000_pch_lpt {
            hw.mac.rar_entry_count = E1000_PCH_LPT_RAR_ENTRIES;
        }

        /* Enable PCS Lock-loss workaround for ICH8 */
        // if hw.mac.kind == MacType::e1000_ich8lan{
        //     if let BoardVariant

        //     set_kmrn_lock_loss_workaround_ich8lan(hw);
        // }

        Ok(())
    }
    fn init_nvm_params<K: KernelFunc>(&mut self, hw: &mut Hw<K>) -> Result {
        let mut nvm_size = 0;

        hw.nvm.kind = NvmType::FlashSw;

        if hw.mac.kind >= MacType::e1000_pch_spt {
            hw.nvm.flash_base_addr = 0;
            let strap_value = hw.er32(STRAP);
            nvm_size = (((strap_value >> 1) & 0x1F) + 1) * NVM_SIZE_MULTIPLIER;

            hw.nvm.flash_bank_size = nvm_size / 2;

            hw.nvm.flash_bank_size /= (size_of::<u16>() as u32);

            hw.flash = Some(hw.mmio + E1000_FLASH_BASE_ADDR);
        } else {
            if hw.flash.is_none() {
                return Err(Error::Config("Flash registers not mapped"));
            }
        }

        hw.nvm.word_size = E1000_ICH8_SHADOW_RAM_WORDS as _;

        Ok(())
    }
    /**
     *  e1000e_set_kmrn_lock_loss_workaround_ich8lan - Set Kumeran workaround state
     *  @hw: pointer to the HW structure
     *  @state: boolean value used to set the current Kumeran workaround state
     *
     *  If ICH8, set the current Kumeran workaround state (enabled - true
     *  /disabled - false).
     **/
    fn set_kmrn_lock_loss_workaround_ich8lan<K: KernelFunc>(
        &mut self,
        base: &mut Hw<K>,
        state: bool,
    ) {
        self.kmrn_lock_loss_workaround_enabled = state;
    }

    pub(crate) fn init_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        self.init_hw_bits(hw);
        //TODO init led

        hw.init_rx_addrs(hw.mac.rar_entry_count);

        /* Zero out the Multicast HASH table */
        info!("Zeroing the MTA");
        for i in 0..hw.mac.mta_reg_count {
            hw.write_reg_array(MTA, i as usize, 0);
        }

        /* Set the transmit descriptor write-back policy for both queues */
        let mut txdctl = hw.er32(TXDCTL(0));
        txdctl = ((txdctl & !TXDCTL_WTHRESH) | TXDCTL_FULL_TX_DESC_WB);
        txdctl = ((txdctl & !TXDCTL_PTHRESH) | TXDCTL_MAX_TX_DESC_PREFETCH);
        hw.ew32(TXDCTL(0), txdctl);
        txdctl = hw.er32(TXDCTL(1));
        txdctl = ((txdctl & !TXDCTL_WTHRESH) | TXDCTL_FULL_TX_DESC_WB);
        txdctl = ((txdctl & !TXDCTL_PTHRESH) | TXDCTL_MAX_TX_DESC_PREFETCH);
        hw.ew32(TXDCTL(1), txdctl);

        /* ICH8 has opposite polarity of no_snoop bits.
         * By default, we should use snoop behavior.
         */
        let snoop = if (hw.mac.kind == MacType::e1000_ich8lan) {
            PCIE_ICH8_SNOOP_ALL
        } else {
            !(PCIE_NO_SNOOP_ALL)
        };
        self.set_pcie_no_snoop(hw, snoop);
        /* Enable workaround for packet loss issue on TGP PCH
         * Do not gate DMA clock from the modPHY block
         */
        if (hw.mac.kind >= MacType::e1000_pch_tgp) {
            let mut fflt_dbg = hw.er32(FFLT_DBG);
            fflt_dbg |= FFLT_DBG_DONT_GATE_WAKE_DMA_CLK;
            hw.ew32(FFLT_DBG, fflt_dbg);
        }

        let mut ctrl_ext = hw.er32(CTRL_EXT);
        ctrl_ext |= CTRL_EXT_RO_DIS;
        hw.ew32(CTRL_EXT, ctrl_ext);

        /* Clear all of the statistics registers (clear on read).  It is
         * important that we do this after we have tried to establish link
         * because the symbol error count will increment wildly if there
         * is no link.
         */
        self.clear_hw_cntrs(hw);

        Ok(())
    }

    fn init_hw_bits<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        /* Extended Device Control */
        let mut reg = hw.er32(CTRL_EXT);
        reg |= BIT(22);
        /* Enable PHY low-power state when MAC is at D3 w/o WoL */
        if (hw.mac.kind >= MacType::e1000_pchlan) {
            reg |= CTRL_EXT_PHYPDEN;
        }
        hw.ew32(CTRL_EXT, reg);

        /* Transmit Descriptor Control 0 */
        reg = hw.er32(TXDCTL(0));
        reg |= BIT(22);
        hw.ew32(TXDCTL(0), reg);

        /* Transmit Descriptor Control 1 */
        reg = hw.er32(TXDCTL(1));
        reg |= BIT(22);
        hw.ew32(TXDCTL(1), reg);

        /* Transmit Arbitration Control 0 */
        reg = hw.er32(TARC(0));
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg |= BIT(28) | BIT(29);
        }
        reg |= BIT(23) | BIT(24) | BIT(26) | BIT(27);
        hw.ew32(TARC(0), reg);

        /* Transmit Arbitration Control 1 */
        reg = hw.er32(TARC(1));
        if (hw.er32(TCTL) & TCTL_MULR) > 0 {
            reg &= !BIT(28);
        } else {
            reg |= BIT(28);
        }
        reg |= BIT(24) | BIT(26) | BIT(30);
        hw.ew32(TARC(1), reg);

        /* Device Status */
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg = hw.er32(STATUS);
            reg &= !BIT(31);
            hw.ew32(STATUS, reg);
        }

        /* work-around descriptor data corruption issue during nfs v2 udp
         * traffic, just disable the nfs filtering capability
         */
        reg = hw.er32(RFCTL);
        reg |= (RFCTL_NFSW_DIS | RFCTL_NFSR_DIS);

        /* Disable IPv6 extension header parsing because some malformed
         * IPv6 headers can hang the Rx.
         */
        if (hw.mac.kind == MacType::e1000_ich8lan) {
            reg |= (RFCTL_IPV6_EX_DIS | RFCTL_NEW_IPV6_EXT_DIS);
        }
        hw.ew32(RFCTL, reg);

        /* Enable ECC on Lynxpoint */
        if (hw.mac.kind >= MacType::e1000_pch_lpt) {
            reg = hw.er32(PBECCSTS);
            reg |= PBECCSTS_ECC_ENABLE;
            hw.ew32(PBECCSTS, reg);

            reg = hw.er32(CTRL);
            reg |= CTRL_MEHE;
            hw.ew32(CTRL, reg);
        }
    }

    /**
     *  e1000e_set_pcie_no_snoop - Set PCI-express capabilities
     *  @hw: pointer to the HW structure
     *  @no_snoop: bitmap of snoop events
     *
     *  Set the PCI-express register to snoop for events enabled in 'no_snoop'.
     **/
    fn set_pcie_no_snoop<K: KernelFunc>(&self, hw: &mut Hw<K>, no_snoop: u32) {
        if (no_snoop > 0) {
            let mut gcr = hw.er32(GCR);
            gcr &= !(PCIE_NO_SNOOP_ALL);
            gcr |= no_snoop;
            hw.ew32(GCR, gcr);
        }
    }

    fn clear_hw_cntrs<K: KernelFunc>(&self, hw: &mut Hw<K>) {
        hw.clear_hw_cntrs_base();

        hw.er32(ALGNERRC);
        hw.er32(RXERRC);
        hw.er32(TNCRS);
        hw.er32(CEXTERR);
        hw.er32(TSCTC);
        hw.er32(TSCTFC);

        hw.er32(MGTPRC);
        hw.er32(MGTPDC);
        hw.er32(MGTPTC);

        hw.er32(IAC);
        hw.er32(ICRXOC);
    }

    pub(crate) fn reset_hw<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut res = hw.disable_pcie_master();
        if res.is_err() {
            error!("PCI-E Master disable polling has failed.");
        }
        debug!("Masking off all interrupts");

        hw.ew32(IMC, 0xffffffff);

        hw.ew32(RCTL, 0);

        hw.ew32(TCTL, TCTL_PSP);
        hw.e1e_flush();

        hw.kernel.delay(10000_000);

        //TODO
        //     /* Workaround for ICH8 bit corruption issue in FIFO memory */
        // if (hw.mac.kind ==MacType:: e1000_ich8lan) {
        // 	/* Set Tx and Rx buffer allocation to 8k apiece. */
        // 	hw.ew32(PBA, PBA_8K);
        // 	/* Set Packet Buffer Size to 16k. */
        // 	hw.ew32(PBS, PBS_16K);
        // }

        // 	if (hw.mac.kind == MacType:: e1000_pchlan) {
        // 		/* Save the NVM K1 bit setting */
        // 		ret_val = self.read_nvm(hw, NVM_K1_CONFIG, 1, &kum_cfg);
        // 		if (ret_val)
        // 			return ret_val;

        // 		if (kum_cfg & NVM_K1_ENABLE){
        // 			self.nvm_k1_enabled = true;
        //         }
        // 		else{
        // 			self.nvm_k1_enabled = false;}
        // 	}

        //  let mut	ctrl =hw. er32(CTRL);

        // if (!hw->phy.ops.check_reset_block(hw)) {
        // 	/* Full-chip reset requires MAC and PHY reset at the same
        // 	 * time to make sure the interface between MAC and the
        // 	 * external PHY is reset.
        // 	 */
        // 	ctrl |= E1000_CTRL_PHY_RST;

        // 	/* Gate automatic PHY configuration by hardware on
        // 	 * non-managed 82579
        // 	 */
        // 	if ((hw->mac.type == e1000_pch2lan) &&
        // 	    !(er32(FWSM) & E1000_ICH_FWSM_FW_VALID))
        // 		e1000_gate_hw_phy_config_ich8lan(hw, true);
        // }
        // ret_val = e1000_acquire_swflag_ich8lan(hw);
        // e_dbg("Issuing a global reset to ich8lan\n");
        // ew32(CTRL, (ctrl | E1000_CTRL_RST));
        // /* cannot issue a flush here because it hangs the hardware */
        // msleep(20);

        // /* Set Phy Config Counter to 50msec */
        // if (hw->mac.type == e1000_pch2lan) {
        // 	reg = er32(FEXTNVM3);
        // 	reg &= ~E1000_FEXTNVM3_PHY_CFG_COUNTER_MASK;
        // 	reg |= E1000_FEXTNVM3_PHY_CFG_COUNTER_50MSEC;
        // 	ew32(FEXTNVM3, reg);
        // }

        // if (!ret_val)
        // 	clear_bit(__E1000_ACCESS_SHARED_RESOURCE, &hw->adapter->state);

        // if (ctrl & E1000_CTRL_PHY_RST) {
        // 	ret_val = hw->phy.ops.get_cfg_done(hw);
        // 	if (ret_val)
        // 		return ret_val;

        // 	ret_val = e1000_post_phy_reset_ich8lan(hw);
        // 	if (ret_val)
        // 		return ret_val;
        // }

        // /* For PCH, this write will make sure that any noise
        //  * will be detected as a CRC error and be dropped rather than show up
        //  * as a bad packet to the DMA engine.
        //  */
        // if (hw->mac.type == e1000_pchlan)
        // 	ew32(CRC_OFFSET, 0x65656565);

        // ew32(IMC, 0xffffffff);
        // er32(ICR);

        // reg = er32(KABGTXD);
        // reg |= E1000_KABGTXD_BGSQLBIAS;
        // ew32(KABGTXD, reg);

        Ok(())
    }

    pub(crate) fn read_nvm<K: KernelFunc>(
        &self,
        hw: &mut Hw<K>,
        offset: u16,
        words: u16,
    ) -> Result<Vec<u16>> {
        if offset > hw.nvm.word_size || (words > (hw.nvm.word_size - offset)) || words == 0 {
            return Err(Error::Nvm("nvm parameter(s) out of bounds"));
        }

        let mut data: Vec<u16> = Vec::try_with_capacity(words as usize)?;

        Ok(data)
    }

    /**
     *  e1000_flash_cycle_init_ich8lan - Initialize flash
     *  @hw: pointer to the HW structure
     *
     *  This function does initial flash setup so that a new read/write/erase cycle
     *  can be started.
     **/
    pub(crate) fn flash_cycle_init<K: KernelFunc>(&self, hw: &mut Hw<K>) -> Result {
        let mut hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;

        /* Check if the flash descriptor is valid */
        if (hsfsts & HSFSTS_FLDESVALID) == 0 {
            let msg = "Flash descriptor invalid.  SW Sequencing must be used.";
            error!("{}", msg);
            return Err(Error::Nvm(msg));
        }

        /* Clear FCERR and DAEL in hw status by writing 1 */

        hsfsts |= HSFSTS_FLCERR | HSFSTS_DAEL;

        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
        } else {
            hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
        }

        /* Either we should have a hardware SPI cycle in progress
         * bit to check against, in order to start a new cycle or
         * FDONE bit should be changed in the hardware so that it
         * is 1 after hardware reset, which can then be used as an
         * indication whether a cycle is in progress or has been
         * completed.
         */
        if (hsfsts & HSFSTS_FLCINPROG) == 0 {
            /* There is no cycle running at present,
             * so we can start a cycle.
             * Begin by setting Flash Cycle Done.
             */
            hsfsts |= HSFSTS_FLCDONE;
            if (hw.mac.kind >= MacType::e1000_pch_spt) {
                hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
            } else {
                hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
            }

            Ok(())
        } else {
            /* Otherwise poll for sometime so the current
             * cycle has a chance to end before giving up.
             */
            for _ in 0..ICH_FLASH_READ_COMMAND_TIMEOUT {
                hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCINPROG) == 0 {
                    /* Successful in waiting for previous cycle to timeout,
                     * now set the Flash Cycle Done.
                     */
                    hsfsts |= HSFSTS_FLCDONE;
                    if (hw.mac.kind >= MacType::e1000_pch_spt) {
                        hw.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
                    } else {
                        hw.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
                    }
                    break;
                }
                hw.kernel.delay(1000);
            }

            let msg = "Flash controller busy, cannot get access";
            error!("{}", msg);
            Err(Error::Nvm(msg))
        }
    }

    /**
     *  e1000_flash_cycle_ich8lan - Starts flash cycle (read/write/erase)
     *  @hw: pointer to the HW structure
     *  @timeout: maximum time to wait for completion
     *
     *  This function starts a flash cycle and waits for its completion.
     **/
    pub(crate) fn flash_cycle<K: KernelFunc>(&self, hw: &mut Hw<K>, timeout: u32) -> Result {
        let mut hsflctl: u16 = 0;
        let mut hsfsts: u16 = 0;

        /* Start a cycle by writing 1 in Flash Cycle Go in Hw Flash Control */
        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hsflctl = (hw.er32flash(ICH_FLASH_HSFSTS)? >> 16) as u16;
        } else {
            hsflctl = hw.er16flash(ICH_FLASH_HSFCTL)?;
        }
        hsflctl |= HSFCTL_FLCGO;

        if (hw.mac.kind >= MacType::e1000_pch_spt) {
            hw.ew32flash(ICH_FLASH_HSFSTS, (hsflctl as u32) << 16)?;
        } else {
            hw.ew16flash(ICH_FLASH_HSFCTL, hsflctl)?;
        }

        /* wait till FDONE bit is set to 1 */
        for i in 0..timeout {
            hsfsts = hw.er16flash(ICH_FLASH_HSFSTS)?;
            if hsfsts & HSFSTS_FLCDONE != 0 {
                break;
            }
            hw.kernel.delay(1000);
        }

        if (hsfsts & HSFSTS_FLCDONE) > 0 && (hsfsts & HSFSTS_FLCERR) == 0 {
            return Ok(());
        }

        return Err(Error::Nvm("timeout"));
    }
}
