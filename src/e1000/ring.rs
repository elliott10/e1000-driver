use core::{
    marker,
    mem::size_of,
    ptr::{self, null, slice_from_raw_parts, slice_from_raw_parts_mut},
    slice::from_raw_parts_mut,
};

use alloc::vec::Vec;

use super::KernelFunc;

pub(crate) const MBUF_SIZE: usize = 2048;

pub(crate) struct Ring<T, K>
where
    T: TransDesc,
    K: KernelFunc,
{
    kernel: K,
    pub(crate) size: usize,
    pub(crate) dma: u64,
    ring: *mut T,
    pub(crate) mbufs: Vec<usize>,
    pub(crate) buff_info: Vec<BuffInfo>,
    pub(crate) next_to_use: usize,
    pub(crate) next_to_clean: usize,
    pub(crate) ims_val: u32,
    _marker: marker::PhantomData<T>,
}

impl<T, K> Ring<T, K>
where
    T: TransDesc,
    K: KernelFunc,
{
    pub(crate) fn new(kernel: K, size: usize) -> Self {
        Self {
            kernel,
            size,
            dma: 0,
            next_to_use: 0,
            next_to_clean: 0,
            ring: ptr::null_mut(),
            mbufs: Vec::new(),
            buff_info: Vec::new(),
            ims_val: 0,
            _marker: Default::default(),
        }
    }
    pub(crate) fn next_to_use(&mut self) -> &mut T {
        let ring = unsafe { from_raw_parts_mut(self.ring, self.size) };
        let ring = &mut ring[self.next_to_use];
        ring
    }

    fn ring_pages(&self) -> usize {
        (self.data_len() + (K::PAGE_SIZE - 1)) / K::PAGE_SIZE
    }

    pub(crate) fn data_len(&self) -> usize {
        self.size * size_of::<T>()
    }
    pub(crate) fn one_copy_data_from(&mut self, index: usize, data: &[u8]) -> usize {
        let len = data.len().min(MBUF_SIZE);
        let buf =
            unsafe { &mut *slice_from_raw_parts_mut(self.mbufs[index] as *mut u8, MBUF_SIZE) };
        buf[..len].copy_from_slice(&data[..len]);
        len
    }
    pub(crate) fn init(&mut self) {
        let alloc_ring_pages = self.ring_pages();

        let (vaddr, dma) = self.kernel.dma_alloc_coherent(alloc_ring_pages);

        self.dma = dma;
        let ring = unsafe { from_raw_parts_mut(vaddr as *mut T, self.size) };
        ring.fill(T::default());
        let mut mbufs = Vec::try_with_capacity(ring.len()).unwrap();
        // 一起申请所有内存
        let alloc_buffer_pages = ((self.size * MBUF_SIZE) + (K::PAGE_SIZE - 1)) / K::PAGE_SIZE;
        let (mut mbufs_vaddr, mut mbufs_dma) = self.kernel.dma_alloc_coherent(alloc_buffer_pages);

        for i in 0..self.size {
            ring[i].set_status(0);
            ring[i].set_addr(mbufs_dma.to_le());
            self.buff_info.try_push(BuffInfo::default()).unwrap();
            mbufs.try_push(mbufs_vaddr).unwrap();
            mbufs_dma += MBUF_SIZE as u64;
            mbufs_vaddr += MBUF_SIZE;
        }
        self.mbufs = mbufs;
        self.ring = unsafe { ring.as_mut_ptr() };
    }
}

impl<T, K> AsRef<[T]> for Ring<T, K>
where
    T: TransDesc,
    K: KernelFunc,
{
    fn as_ref(&self) -> &[T] {
        unsafe { &*slice_from_raw_parts(self.ring, self.size) }
    }
}

impl<T, K> AsMut<[T]> for Ring<T, K>
where
    T: TransDesc,
    K: KernelFunc,
{
    fn as_mut(&mut self) -> &mut [T] {
        unsafe { &mut *slice_from_raw_parts_mut(self.ring, self.size) }
    }
}

impl<T, K> Drop for Ring<T, K>
where
    T: TransDesc,
    K: KernelFunc,
{
    fn drop(&mut self) {
        if self.mbufs.is_empty() {
            return;
        }

        if self.dma == 0 {
            return;
        }
        self.kernel
            .dma_free_coherent(self.ring as usize, self.dma, self.ring_pages());
    }
}

pub trait TransDesc: Default + Clone {
    fn set_status(&mut self, status: u32);
    fn set_addr(&mut self, addr: u64);
}

// #[derive(Debug, Default, Clone)]
// #[repr(C, align(16))]
// pub struct TxDesc {
//     pub addr: u64,
//     pub length: u16,
//     pub cso: u8,
//     pub cmd: u8,
//     pub status: u8,
//     pub css: u8,
//     pub special: u16,
// }
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct TxDesc {
    pub addr: u64,
    pub lower: u32,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}
impl TransDesc for TxDesc {
    fn set_status(&mut self, status: u32) {
        self.status = status as u8;
    }

    fn set_addr(&mut self, addr: u64) {
        self.addr = addr;
    }
}

impl TxDesc {}

/// Receive Descriptor - Extended
#[derive(Debug, Default, Clone)]
#[repr(C)]
pub struct RxDesc {
    pub addr: u64, /* Address of the descriptor's data buffer */
    pub rsvd: u64,
    pub mrq: u32,
    pub ip_id: u16,
    pub csum: u16,
    pub status_error: u32,
    pub length: u16, /* Length of data DMAed into data buffer */
    pub vlan: u16,
}
impl TransDesc for RxDesc {
    fn set_status(&mut self, status: u32) {
        self.status_error = status;
    }

    fn set_addr(&mut self, addr: u64) {
        self.addr = addr;
    }
}

#[derive(Default)]
pub(crate) struct BuffInfo {
    pub(crate) length: usize,
    pub(crate) time_stamp: u64,
    pub(crate) next_to_watch: usize,
}
