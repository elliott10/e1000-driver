use alloc::vec::Vec;
use core::{
    alloc::Layout,
    marker::{self, PhantomData},
    mem::{self, size_of},
    ops::{Deref, DerefMut},
    ptr::{self, null, slice_from_raw_parts, slice_from_raw_parts_mut},
    slice::{from_raw_parts, from_raw_parts_mut},
};

use super::{err::Result, DMA};

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
    pub dma_desc: DMA,
    dma_buff: DMA,
}

impl<T> Ring<T>
where
    T: Descriptor,
{
    pub fn new(buff_len: usize) -> Result<Self> {
        let size = DEFAULT_RING_SIZE;

        #[cfg(no_global_oom_handling)]
        let mut elems = Vec::try_with_capacity(size)?;
        #[cfg(not(no_global_oom_handling))]
        let mut elems = Vec::with_capacity(size);

        let mut layout = Layout::array::<T>(size).unwrap();
        layout = layout.align_to(4096).unwrap();
        let dma_desc: DMA = DMA::try_new(layout.size())?;
        let dma_buff: DMA = DMA::try_new(buff_len * size)?;
        let mut buff_dma_addr: u64 = dma_buff.dma_addr();
        let mut buff_cpu_addr = dma_buff.cpu_addr();

        unsafe {
            let desc_ptr = dma_desc.cpu_addr() as *mut T;

            for i in 0..size {
                let desc = &mut *desc_ptr.offset(i as _);
                desc.set_dma_addr(buff_dma_addr);
                let mut elem = RingElem {
                    desc_ptr: desc,
                    length: buff_len,
                    dma_addr: buff_dma_addr,
                    cpu_addr: buff_cpu_addr,
                };

                #[cfg(no_global_oom_handling)]
                elems.try_push(elem)?;
                #[cfg(not(no_global_oom_handling))]
                elems.push(elem);
                buff_dma_addr += buff_len as u64;
                buff_cpu_addr += buff_len;
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

    pub fn elem_copy_from_slice(&mut self, index: usize, src: &[u8]) -> usize {
        let len = src.len().min(self.buff_len);

        let buf = unsafe {
            &mut *slice_from_raw_parts_mut(
                (self.dma_buff.cpu_addr() + index * self.buff_len) as *mut u8,
                self.buff_len,
            )
        };

        buf[..len].copy_from_slice(&src[..len]);
        len
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
    pub dma_addr: u64,
    pub cpu_addr: usize,
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
pub union TxDescLower {
    pub data: u32,
    pub flags: TxDescLowerFlags,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct TxDescLowerFlags {
    pub length: u16, /* Data buffer length */
    pub cso: u8,     /* Checksum offset */
    pub cmd: u8,     /* Descriptor control */
}

#[repr(C)]
pub struct TxDesc {
    pub addr: u64,
    pub lower: TxDescLower,
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

const MAX_PS_BUFFERS: usize = 4;
const PS_PAGE_BUFFERS: usize = MAX_PS_BUFFERS - 1;

/* Receive Descriptor - Packet Split */
#[repr(C)]
pub union RxDescPacketSplit {
    pub buffer_addr: [u64; MAX_PS_BUFFERS],
    pub wb: RxDescPacketSplitWB,
}

impl Descriptor for RxDescPacketSplit {
    fn set_dma_addr(&mut self, dma_addr: u64) {
        todo!()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RxDescPacketSplitWB {
    pub mrq: u32,
    pub hi_dword: RxDescPacketSplitHiDword,
    pub status_error: u32,
    pub length0: u16,
    pub vlan: u16,
    pub header_status: u16,
    pub length: [u16; PS_PAGE_BUFFERS],
    pub reserved: u64,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union RxDescPacketSplitHiDword {
    pub rss: u32,
    pub csum_ip: RxDescPacketSplitCsumIp,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct RxDescPacketSplitCsumIp {
    pub ip_id: u16,
    pub csum: u16,
}
