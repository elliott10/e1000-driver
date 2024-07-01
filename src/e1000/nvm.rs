use core::time::Duration;

use super::reg::*;
use super::variant::*;
use crate::defines::*;
use crate::e1000::lock::SemaphoreBackend;
use crate::err::{Error, Result};
use crate::kfn;
use alloc::vec::Vec;

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
            NvmSemaphore::Spt => todo!(),
            NvmSemaphore::E82571 => e82571::NvmSemaphore::new(self.reg).acquire(),
        }?;
        unsafe { Ok(NvmGuard { data: self }) }
    }
    pub fn read_nvm(&self, offset: u16, words: u16) -> Result<Vec<u16>> {
        match &self.ops.read_write {
            NvmReadWrite::E82571 => self.read_nvm_eerd(offset, words),
            NvmReadWrite::Ich8lan => todo!(),
        }
    }

    fn read_nvm_eerd(&self, offset: u16, words: u16) -> Result<Vec<u16>> {
        if offset > self.word_size || (words > (self.word_size - offset)) || words == 0 {
            return Err(Error::Nvm("nvm parameter(s) out of bounds"));
        }
        let mut data: Vec<u16> = Vec::try_with_capacity(words as usize)?;

        let mut eerd = 0u32;

        for i in 0..words {
            eerd = (((offset + i) as u32) << NVM_RW_ADDR_SHIFT) + NVM_RW_REG_START;

            self.reg.ew32(EERD, eerd);

            self.poll_eerd_eewr_done(NVM_POLL_READ)?;

            let d = self.reg.er32(EERD) >> NVM_RW_REG_DATA;

            data.try_push(d as u16)?;
        }

        Ok(data)
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
            NvmSemaphore::Spt => todo!(),
            NvmSemaphore::E82571 => e82571::NvmSemaphore::new(self.data.reg).release(),
        }
    }
}
