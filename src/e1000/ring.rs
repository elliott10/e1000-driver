use alloc::vec::Vec;
use core::{
    marker,
    mem::size_of,
    ptr::{self, null, slice_from_raw_parts, slice_from_raw_parts_mut},
    slice::from_raw_parts_mut,
};

use crate::err::Result;

pub(crate) struct Ring<'a, Desc>
where
    Desc: Descriptor,
{
    elems: Vec<RingElem<'a, Desc>>,
    buff_len: usize,
    pub next_to_use: usize,
    pub next_to_clean: usize,
    pub ims_val: u32,
}

impl<Desc> Ring<'_, Desc>
where
    Desc: Descriptor,
{
    pub fn new(len: usize, buff_len: usize) -> Result<Self> {
        let elems = Vec::try_with_capacity(len)?;
        Ok(Self {
            elems,
            buff_len,
            next_to_use: 0,
            next_to_clean: 0,
            ims_val: 0,
        })
    }

    pub fn init(&mut self) -> Result {
        Ok(())
    }
}

pub(crate) trait Descriptor {}

pub(crate) struct RingElem<'a, Desc>
where
    Desc: Descriptor,
{
    desc: &'a mut Desc,
}
