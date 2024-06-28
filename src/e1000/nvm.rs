use super::{reg::Reg, variant::e82571};
use crate::e1000::lock::Semaphore;

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
