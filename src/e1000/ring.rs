use alloc::vec::Vec;
use core::{
    alloc::Layout,
    marker::{self, PhantomData},
    mem::{self, size_of},
    ops::{Deref, DerefMut},
    ptr::{self, null, slice_from_raw_parts, slice_from_raw_parts_mut},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use crate::{err::Result, DMA};

const DEFAULT_RING_SIZE: usize = 256;

pub(crate) struct Ring<T>
where
    T: Descriptor,
{
    elems: Vec<RingElem<T>>,
    pub buff_len: usize,
    pub next_to_use: usize,
    pub next_to_clean: usize,
    pub ims_val: u32,
    dma_desc: DMA,
    dma_buff: DMA,
}

impl<T> Ring<T>
where
    T: Descriptor,
{
    pub fn new(buff_len: usize) -> Result<Self> {
        let size = DEFAULT_RING_SIZE;
        let mut elems = Vec::try_with_capacity(size)?;
        let mut layout = Layout::array::<T>(size).unwrap();
        layout = layout.align_to(4096).unwrap();
        let dma_desc: DMA = DMA::try_new(layout.size())?;
        let dma_buff: DMA = DMA::try_new(buff_len * size)?;
        let mut buff_addr: u64 = dma_buff.dma_addr();

        unsafe {
            let desc_ptr = dma_desc.cpu_addr() as *mut T;

            for i in 0..size {
                let desc = &mut *desc_ptr.offset(i as _);
                desc.set_dma_addr(buff_addr);
                let mut elem = RingElem {
                    desc_ptr: desc,
                    length: buff_len,
                };

                elems.try_push(elem)?;
                buff_addr += buff_len as u64;
            }
        }
        Ok(Self {
            elems,
            buff_len,
            next_to_use: 0,
            next_to_clean: 0,
            ims_val: 0,
            dma_desc,
            dma_buff,
        })
    }
}

impl<T> Deref for Ring<T>
where
    T: Descriptor,
{
    type Target = [RingElem<T>];

    fn deref(&self) -> &Self::Target {
        &self.elems
    }
}

impl<T> DerefMut for Ring<T>
where
    T: Descriptor,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elems
    }
}

pub(crate) trait Descriptor {
    fn set_dma_addr(&mut self, dma_addr: u64);
}

pub(crate) struct RingElem<T>
where
    T: Descriptor,
{
    desc_ptr: *mut T,
    pub length: usize,
}

impl<T> RingElem<T>
where
    T: Descriptor,
{
    pub fn desc<'a>(&'a self) -> &'a T {
        unsafe { &*self.desc_ptr }
    }

    pub fn desc_mut<'a>(&'a mut self) -> &'a mut T {
        unsafe { &mut *self.desc_ptr }
    }
}

#[repr(C)]
pub struct TxDesc {
    pub addr: u64,
    pub length: u16, /* Data buffer length */
    pub cso: u8,     /* Checksum offset */
    pub cmd: u8,     /* Descriptor control */
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

impl Descriptor for TxDesc {
    fn set_dma_addr(&mut self, dma_addr: u64) {
        self.addr = dma_addr;
    }
}

/// Receive Descriptor - Extended
#[derive(Clone, Copy)]
#[repr(C)]
pub union RxDescExt {
    pub read: RxDescExtRead,
    pub wb: RxDescExtWB,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RxDescExtRead {
    pub addr: u64, /* Address of the descriptor's data buffer */
    pub reserved: u64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RxDescExtWB {
    pub mrq: u32,
    pub ip_id: u16,
    pub csum: u16,
    pub status_error: u32,
    pub length: u16, /* Length of data DMAed into data buffer */
    pub vlan: u16,
}
impl Descriptor for RxDescExt {
    fn set_dma_addr(&mut self, dma_addr: u64) {
        self.read.addr = dma_addr;
    }
}
