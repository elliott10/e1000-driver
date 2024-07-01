use core::cell::UnsafeCell;

use crate::err::*;

pub(crate) trait SemaphoreBackend {
    fn acquire(&self) -> Result;
    fn release(&self);
}

pub(crate) struct Semaphore<B: SemaphoreBackend> {
    pub(crate) backend: B,
}

impl<B: SemaphoreBackend> Drop for Semaphore<B> {
    fn drop(&mut self) {
        self.backend.release()
    }
}
