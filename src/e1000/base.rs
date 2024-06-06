use super::board::*;
use super::hw::*;
use super::regs::*;
use crate::err::{Error, Result};
use crate::*;
use core::alloc::Layout;
pub(crate) use core::borrow::*;
use core::ptr::NonNull;

pub(crate) struct Base<K>
where
    K: KernelFunc,
{
    pub(crate) kernel: K,
    pub(crate) mmio: usize,
    pub(crate) board: Board,
    pub(crate) flash: Option<usize>,
    pub(crate) stats: E1000HWStats,
}

impl<K> Base<K>
where
    K: KernelFunc,
{
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
        if self.board.flags2.pcim2pci_arbiter_wa {
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

    pub(crate) fn ral(&self, index: usize) -> usize {
        RA + index * 8
    }

    pub(crate) fn rah(&self, index: usize) -> usize {
        RA + index * 8 + 4
    }
}

pub(crate) struct BaseRc<K>(NonNull<Base<K>>)
where
    K: KernelFunc;

impl<K> BaseRc<K>
where
    K: KernelFunc,
{
    pub(crate) fn new(kfun: K, board: Board) -> Self {
        unsafe {
            let mmio = kfun.bar_n_remap(0);
            let layout = Layout::new::<Base<K>>();
            let ptr = alloc::alloc(layout) as *mut Base<K>;
            let base = &mut *ptr;
            base.kernel = kfun;
            base.mmio = mmio;
            base.board = board;
            base.flash = None;
            base.stats = E1000HWStats::default();
            Self(NonNull::new_unchecked(ptr))
        }
    }
    pub(crate) fn init(&mut self) -> Result {
        let base = self.as_mut();

        if base.board.flags.has_flash && base.board.mac.kind < E1000MacType::e1000_pch_spt {
            let addr = base.kernel.bar_n_remap(1);
            if addr > 0 {
                info!("flash: {:#X}", addr);
                base.flash = Some(addr);
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
            let layout = Layout::new::<Base<K>>();
            let ptr = self.0.as_ptr() as *mut u8;
            alloc::dealloc(ptr, layout);
        }
    }
}

impl<K> AsRef<Base<K>> for BaseRc<K>
where
    K: KernelFunc,
{
    fn as_ref(&self) -> &Base<K> {
        unsafe { self.0.as_ref() }
    }
}

impl<K> AsMut<Base<K>> for BaseRc<K>
where
    K: KernelFunc,
{
    fn as_mut(&mut self) -> &mut Base<K> {
        unsafe { self.0.as_mut() }
    }
}
impl<K: KernelFunc> Borrow<Base<K>> for BaseRc<K> {
    fn borrow(&self) -> &Base<K> {
        self.as_ref()
    }
}

impl<K: KernelFunc> BorrowMut<Base<K>> for BaseRc<K> {
    fn borrow_mut(&mut self) -> &mut Base<K> {
        self.as_mut()
    }
}
