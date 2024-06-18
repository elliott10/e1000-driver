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

const PCIE_ICH8_SNOOP_ALL: u32 =	PCIE_NO_SNOOP_ALL;


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

            let snoop = 
	/* ICH8 has opposite polarity of no_snoop bits.
	 * By default, we should use snoop behavior.
	 */
	if (hw.mac.kind == MacType:: e1000_ich8lan){
		PCIE_ICH8_SNOOP_ALL
    }
	else{
		!(PCIE_NO_SNOOP_ALL)
    };
	self.set_pcie_no_snoop(hw, snoop);
    	/* Enable workaround for packet loss issue on TGP PCH
	 * Do not gate DMA clock from the modPHY block
	 */
	if (hw.mac.kind >=MacType:: e1000_pch_tgp) {
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
        if (hw.er32(TCTL) & TCTL_MULR) > 0{
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
}
