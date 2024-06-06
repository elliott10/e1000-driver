use core::mem::size_of;

use super::super::regs::*;
use super::Hw;
use super::NvmType;
use crate::err::*;
use crate::BoardVariant;
use crate::KernelFunc;
use crate::MacType;
use crate::MediaType;
use crate::E1000;

const E1000_ICH_RAR_ENTRIES: u16 = 7;
const E1000_PCH2_RAR_ENTRIES: u16 = 5; /* RAR[0], SHRA[0-3] */
const E1000_PCH_LPT_RAR_ENTRIES: u16 = 12; /* RAR[0], SHRA[0-10] */

const NVM_SIZE_MULTIPLIER: u32 = 4096; // multiplier for NVMS field
const E1000_FLASH_BASE_ADDR: usize = 0xE000; // offset of NVM access regs
const E1000_CTRL_EXT_NVMVS: u32 = 0x3; // NVM valid sector

// TARC0 constants with bit masks
const E1000_TARC0_CB_MULTIQ_3_REQ: u32 = 0x30000000;
const E1000_TARC0_CB_MULTIQ_2_REQ: u32 = 0x20000000;

#[derive(Debug, Clone, Default)]
pub(crate) struct ShadowRam {
    pub(crate) value: u16,
    pub(crate) modified: bool,
}
const E1000_ICH8_SHADOW_RAM_WORDS: usize = 2048;

#[derive(Debug, Clone)]
pub(crate) struct DevSpecIch8lan {
    pub(crate) kmrn_lock_loss_workaround_enabled: bool,
    pub(crate) shadow_ram: [ShadowRam; E1000_ICH8_SHADOW_RAM_WORDS],
    pub(crate) nvm_k1_enabled: bool,
    pub(crate) eee_disable: bool,
    pub(crate) eee_lp_ability: u16,
    // pub ulp_state: E1000UlpState,
}

pub(crate) fn get_variants<K: KernelFunc>(hw: &mut Hw<K>) -> Result {
    init_mac_params(hw)?;
    init_nvm_params(hw)?;
    Ok(())
}

fn init_mac_params<K: KernelFunc>(hw: &mut Hw<K>) -> Result {
    hw.phy.media_type = MediaType::media_type_copper;
    /* Set mta register count */
    hw.mac.mta_reg_count = 32;
    /* Set rar entry count */
    hw.mac.rar_entry_count = E1000_ICH_RAR_ENTRIES;
    if hw.mac.kind == MacType::e1000_ich8lan {
        hw.mac.rar_entry_count -= 1;
    }

    hw.mac.has_fwsm = true;

    hw.mac.arc_subsystem_valid = false;

    hw.mac.adaptive_ifs = true;

    // TODO led

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
fn init_nvm_params<K: KernelFunc>(hw: &mut Hw<K>) -> Result {
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
fn set_kmrn_lock_loss_workaround_ich8lan<K: KernelFunc>(base: &mut Hw<K>) {
    // dev_spec->kmrn_lock_loss_workaround_enabled = state;
}
