use super::KernelFunc;

pub(crate) struct Ring<K>
where
    K: KernelFunc,
{
    kernel: K,
}

impl<K> Ring<K>
where
    K: KernelFunc,
{
    pub(crate) fn new(kernel: K) -> Self {
        Self { kernel }
    }
}

impl<K> Drop for Ring<K>
where
    K: KernelFunc,
{
    fn drop(&mut self) {}
}

/// [E1000 3.3.3]
/// The dma descriptor for transmitting
#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct TxDesc {
    pub addr: u64,
    pub data_l: u32,
    pub length: u16,
    pub cso: u8,
    pub cmd: u8,
    pub data_u: u8,
    pub status: u8,
    pub css: u8,
    pub special: u16,
}

/// [E1000 3.2.3]
/// The dma descriptor for receiving
#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct RxDesc {
    pub addr: u64,   /* Address of the descriptor's data buffer */
    pub length: u16, /* Length of data DMAed into data buffer */
    pub csum: u16,   /* Packet checksum */
    pub status: u8,  /* Descriptor status */
    pub errors: u8,  /* Descriptor Errors */
    pub special: u16,
}
