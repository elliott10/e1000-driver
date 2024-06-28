use crate::err::*;

pub(crate) trait Semaphore {
    fn acquire(&self) -> Result;
    fn release(&self);
}
