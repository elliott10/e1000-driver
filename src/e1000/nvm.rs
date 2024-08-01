use core::mem::size_of;
use core::time::Duration;

use super::defines::*;
use super::err::{Error, Result};
use super::kfn;
use super::lock::SemaphoreBackend;
use super::mac::MacType;
use super::reg::*;
use super::variant::*;
use alloc::vec::Vec;

struct FlashCtrl {
    value: u16,
}

impl FlashCtrl {
    fn set_count(&mut self, count: usize) {
        let c = (count & 0b11) as u16;
        self.value &= !(0b11 << HSFCTL_FLDBCOUNT_SHIFT);
        self.value |= c << HSFCTL_FLDBCOUNT_SHIFT;
    }

    fn set_cycle(&mut self, cycle: u16) {
        let c = cycle & 0b11;

        self.value &= !(0b11 << HSFCTL_FLCYCLE_SHIFT);
        self.value |= c << HSFCTL_FLCYCLE_SHIFT;
    }
}
impl From<u16> for FlashCtrl {
    fn from(value: u16) -> Self {
        Self { value }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct NvmOps {
    pub semaphore: NvmSemaphore,
    pub read_write: NvmReadWrite,
}

#[derive(Debug, Clone)]
pub(crate) enum NvmReadWrite {
    E82571,
    Ich8lan,
}

impl Default for NvmSemaphore {
    fn default() -> Self {
        Self::E82571
    }
}
impl Default for NvmReadWrite {
    fn default() -> Self {
        Self::E82571
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NvmSemaphore {
    E82571,
    Spt,
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
    pub kind: NvmType,
    pub reg: Reg,
    pub ops: NvmOps,
    pub mac_kind: MacType,
    pub nvm_override: NvmOverride,

    pub flash_bank_size: u32,
    pub flash_base_addr: u32,

    pub word_size: u16,
    pub delay_usec: u16,
    pub address_bits: u16,
    pub opcode_bits: u16,
    pub page_size: u16,
}

impl NvmInfo {
    pub fn acquire(&self) -> Result<NvmGuard<'_>> {
        match &self.ops.semaphore {
            NvmSemaphore::Spt => ich8lan::NvmSemaphore::new(self.reg).acquire(),
            NvmSemaphore::E82571 => e82571::NvmSemaphore::new(self.reg).acquire(),
        }?;
        unsafe { Ok(NvmGuard { data: self }) }
    }
    pub fn read_nvm(&self, offset: u16, words: u16, variant: &Variant) -> Result<Vec<u16>> {
        match &self.ops.read_write {
            NvmReadWrite::E82571 => self.read_nvm_eerd(offset, words),
            NvmReadWrite::Ich8lan => self.read_nvm_ich8lan(offset, words, variant),
        }
    }

    fn read_nvm_ich8lan(&self, offset: u16, words: u16, variant: &Variant) -> Result<Vec<u16>> {
        if let VariantKind::Ich8lan(v) = &variant.kind {
            if offset > self.word_size || (words > (self.word_size - offset)) || words == 0 {
                return Err(Error::Nvm("nvm parameter(s) out of bounds"));
            }
            #[cfg(not(oom_handle))]
            let mut data: Vec<u16> = Vec::try_with_capacity(words as usize)?;
            #[cfg(oom_handle)]
            let mut data: Vec<u16> = Vec::with_capacity(words as usize);
            for _ in 0..words {
                #[cfg(not(oom_handle))]
                data.try_push(0).unwrap();
                #[cfg(oom_handle)]
                data.push(0);
            }

            let res = self.valid_nvm_bank_detect();
            let bank = match res {
                Ok(bank) => bank,
                Err(_) => 0,
            };

            let mut act_offset = if (bank) > 0 { self.flash_bank_size } else { 0 };
            act_offset += offset as u32;

            let mut i = 0usize;
            let mut offset_to_read = 0;
            let words = words as usize;
            let offset = offset as usize;
            let act_offset = act_offset as usize;
            let mut dword = 0u32;

            while i < words {
                if (words - i == 1) {
                    if (v.shadow_ram[offset + i].modified) {
                        data[i] = v.shadow_ram[offset + i].value;
                    } else {
                        offset_to_read = act_offset + i - ((act_offset + i) % 2);

                        dword = self.read_flash_dword(offset_to_read)?;

                        if ((act_offset + i) % 2 == 0) {
                            data[i] = (dword & 0xFFFF) as u16;
                        } else {
                            data[i] = ((dword >> 16) & 0xFFFF) as u16;
                        }
                    }
                } else {
                    offset_to_read = act_offset + i;
                    if (!(v.shadow_ram[offset + i].modified)
                        || !(v.shadow_ram[offset + i + 1].modified))
                    {
                        dword = self.read_flash_dword(offset_to_read)?;
                    }
                    if (v.shadow_ram[offset + i].modified) {
                        data[i] = v.shadow_ram[offset + i].value;
                    } else {
                        data[i] = (dword & 0xFFFF) as u16;
                    }
                    if (v.shadow_ram[offset + i].modified) {
                        data[i + 1] = v.shadow_ram[offset + i + 1].value;
                    } else {
                        data[i + 1] = (dword >> 16 & 0xFFFF) as u16;
                    }
                }

                i += 2;
            }

            Ok(data)
        } else {
            return Err(Error::Nvm("not support"));
        }
    }

    fn read_nvm_eerd(&self, offset: u16, words: u16) -> Result<Vec<u16>> {
        if offset > self.word_size || (words > (self.word_size - offset)) || words == 0 {
            return Err(Error::Nvm("nvm parameter(s) out of bounds"));
        }
        #[cfg(not(oom_handle))]
        let mut data: Vec<u16> = Vec::try_with_capacity(words as usize)?;
        #[cfg(oom_handle)]
        let mut data: Vec<u16> = Vec::with_capacity(words as usize);
        let mut eerd = 0u32;

        for i in 0..words {
            eerd = (((offset + i) as u32) << NVM_RW_ADDR_SHIFT) + NVM_RW_REG_START;

            self.reg.ew32(EERD, eerd);

            self.poll_eerd_eewr_done(NVM_POLL_READ)?;

            let d = self.reg.er32(EERD) >> NVM_RW_REG_DATA;

            #[cfg(not(oom_handle))]
            data.try_push(d as u16)?;
            #[cfg(oom_handle)]
            data.push(d as u16);
        }

        Ok(data)
    }
    /**
     *  e1000_read_flash_data32_ich8lan - Read dword from NVM
     *  @hw: pointer to the HW structure
     *  @offset: The offset (in bytes) of the dword to read.
     *  @data: Pointer to the dword to store the value read.
     *
     *  Reads a byte or word from the NVM using the flash access registers.
     **/
    pub(crate) fn read_flash_data32(&self, offset: usize) -> Result<u32> {
        if (offset as u32 > ICH_FLASH_LINEAR_ADDR_MASK) {
            return Err(Error::Nvm("not support"));
        }
        let flash_linear_addr =
            ((ICH_FLASH_LINEAR_ADDR_MASK & offset as u32) + self.flash_base_addr);

        for _ in 0..ICH_FLASH_CYCLE_REPEAT_COUNT {
            kfn().delay(Duration::from_micros(1));
            /* Steps */
            self.flash_cycle_init()?;
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            let mut hsflctl: FlashCtrl =
                ((self.reg.er32flash(ICH_FLASH_HSFSTS)? >> 16) as u16).into();

            /* 0b/1b corresponds to 1 or 2 byte size, respectively. */
            hsflctl.set_count(size_of::<u32>() - 1);
            hsflctl.set_cycle(ICH_CYCLE_READ);
            /* In SPT, This register is in Lan memory space, not flash.
             * Therefore, only 32 bit access is supported
             */
            self.reg
                .ew32flash(ICH_FLASH_HSFSTS, (hsflctl.value as u32) << 16);
            self.reg.ew32flash(ICH_FLASH_FADDR, flash_linear_addr);

            let r = self.flash_cycle_ich8lan(ICH_FLASH_READ_COMMAND_TIMEOUT);

            /* Check if FCERR is set to 1, if set to 1, clear it
             * and try the whole sequence a few more times, else
             * read in (shift in) the Flash Data0, the order is
             * least significant byte first msb to lsb
             */
            if (r.is_ok()) {
                return self.reg.er32flash(ICH_FLASH_FDATA0);
            } else {
                /* If we've gotten here, then things are probably
                 * completely hosed, but if the error condition is
                 * detected, it won't hurt to give it another try...
                 * ICH_FLASH_CYCLE_REPEAT_COUNT times.
                 */
                let hsfsts = self.reg.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCERR) > 0 {
                    /* Repeat for some time before giving up. */
                    continue;
                } else if (hsfsts & HSFSTS_FLCDONE) == 0 {
                    error!("Timeout error - flash cycle did not complete.");
                    break;
                }
            }
        }

        return Err(Error::Nvm("timeout"));
    }
    /**
     *  flash_cycle_ich8lan - Starts flash cycle (read/write/erase)
     *  @hw: pointer to the HW structure
     *  @timeout: maximum time to wait for completion
     *
     *  This function starts a flash cycle and waits for its completion.
     **/
    pub(crate) fn flash_cycle_ich8lan(&self, timeout: u32) -> Result {
        let mut hsflctl: u16 = 0;
        let mut hsfsts: u16 = 0;

        /* Start a cycle by writing 1 in Flash Cycle Go in Hw Flash Control */
        if (self.mac_kind >= MacType::e1000_pch_spt) {
            hsflctl = (self.reg.er32flash(ICH_FLASH_HSFSTS)? >> 16) as u16;
        } else {
            hsflctl = self.reg.er16flash(ICH_FLASH_HSFCTL)?;
        }
        hsflctl |= HSFCTL_FLCGO;

        if (self.mac_kind >= MacType::e1000_pch_spt) {
            self.reg
                .ew32flash(ICH_FLASH_HSFSTS, (hsflctl as u32) << 16)?;
        } else {
            self.reg.ew16flash(ICH_FLASH_HSFCTL, hsflctl)?;
        }

        /* wait till FDONE bit is set to 1 */
        for i in 0..timeout {
            hsfsts = self.reg.er16flash(ICH_FLASH_HSFSTS)?;
            if hsfsts & HSFSTS_FLCDONE != 0 {
                break;
            }
            kfn().delay(Duration::from_micros(1));
        }

        if (hsfsts & HSFSTS_FLCDONE) > 0 && (hsfsts & HSFSTS_FLCERR) == 0 {
            return Ok(());
        }

        return Err(Error::Nvm("timeout"));
    }
    /**
     *  flash_cycle_init_ich8lan - Initialize flash
     *
     *  This function does initial flash setup so that a new read/write/erase cycle
     *  can be started.
     **/
    pub(crate) fn flash_cycle_init(&self) -> Result {
        let mut hsfsts = self.reg.er16flash(ICH_FLASH_HSFSTS)?;

        /* Check if the flash descriptor is valid */
        if (hsfsts & HSFSTS_FLDESVALID) == 0 {
            let msg = "Flash descriptor invalid.  SW Sequencing must be used.";
            error!("{}", msg);
            return Err(Error::Nvm(msg));
        }

        /* Clear FCERR and DAEL in hw status by writing 1 */

        hsfsts |= HSFSTS_FLCERR | HSFSTS_DAEL;

        if (self.mac_kind >= MacType::e1000_pch_spt) {
            self.reg.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
        } else {
            self.reg.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
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
            if (self.mac_kind >= MacType::e1000_pch_spt) {
                self.reg.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
            } else {
                self.reg.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
            }

            Ok(())
        } else {
            /* Otherwise poll for sometime so the current
             * cycle has a chance to end before giving up.
             */
            for _ in 0..ICH_FLASH_READ_COMMAND_TIMEOUT {
                hsfsts = self.reg.er16flash(ICH_FLASH_HSFSTS)?;
                if (hsfsts & HSFSTS_FLCINPROG) == 0 {
                    /* Successful in waiting for previous cycle to timeout,
                     * now set the Flash Cycle Done.
                     */
                    hsfsts |= HSFSTS_FLCDONE;
                    if (self.mac_kind >= MacType::e1000_pch_spt) {
                        self.reg.ew32flash(ICH_FLASH_HSFSTS, hsfsts as u32 & 0xFFFF);
                    } else {
                        self.reg.ew16flash(ICH_FLASH_HSFSTS, hsfsts);
                    }
                    break;
                }
                kfn().delay(Duration::from_micros(1));
            }

            let msg = "Flash controller busy, cannot get access";
            error!("{}", msg);
            Err(Error::Nvm(msg))
        }
    }
    pub(crate) fn read_flash_dword(&self, mut offset: usize) -> Result<u32> {
        /* Must convert word offset into bytes. */
        offset <<= 1;
        return self.read_flash_data32(offset);
    }

    pub(crate) fn valid_nvm_bank_detect(&self) -> Result<u32> {
        info!("valid_nvm_bank_detect");

        let bank1_offset = self.flash_bank_size as usize;
        let act_offset = ICH_NVM_SIG_WORD;

        let mut bank = 0;

        /* Check bank 0 */
        let mut nvm_dword = self.read_flash_dword(act_offset)?;

        let mut sig_byte = ((nvm_dword & 0xFF00) >> 8) as u8;
        if (sig_byte & ICH_NVM_VALID_SIG_MASK) == ICH_NVM_SIG_VALUE {
            return Ok(bank);
        }

        /* Check bank 1 */
        nvm_dword = self.read_flash_dword(act_offset + bank1_offset)?;

        sig_byte = ((nvm_dword & 0xFF00) >> 8) as u8;
        if ((sig_byte & ICH_NVM_VALID_SIG_MASK) == ICH_NVM_SIG_VALUE) {
            bank = 1;
            return Ok(bank);
        }

        error!("ERROR: No valid NVM bank present");
        return Err(Error::Nvm("ERROR: No valid NVM bank present"));
    }
    fn poll_eerd_eewr_done(&self, ee_reg: u8) -> Result {
        let mut reg;

        for _ in 0..100000 {
            if (ee_reg == NVM_POLL_READ) {
                reg = self.reg.er32(EERD);
            } else {
                reg = self.reg.er32(EEWR);
            }

            if (reg & NVM_RW_REG_DONE > 0) {
                return Ok(());
            }

            kfn().delay(Duration::from_micros(5));
        }

        Err(Error::Nvm("poll fail"))
    }
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
pub(crate) struct NvmGuard<'a> {
    data: &'a NvmInfo,
}

impl core::ops::Deref for NvmGuard<'_> {
    type Target = NvmInfo;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl core::ops::DerefMut for NvmGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr = self.data as *const NvmInfo;
            let ptr = ptr as *mut NvmInfo;
            &mut *ptr
        }
    }
}

impl Drop for NvmGuard<'_> {
    fn drop(&mut self) {
        match &self.data.ops.semaphore {
            NvmSemaphore::Spt => ich8lan::NvmSemaphore::new(self.data.reg).release(),
            NvmSemaphore::E82571 => e82571::NvmSemaphore::new(self.data.reg).release(),
        }
    }
}
