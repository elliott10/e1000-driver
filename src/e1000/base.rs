use super::board::*;
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

impl<K> Base<K> where K: KernelFunc {}

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
