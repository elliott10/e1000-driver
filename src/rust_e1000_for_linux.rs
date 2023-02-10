// SPDX-License-Identifier: GPL-2.0
//! Rust e1000 network device.
use core::sync::atomic::AtomicPtr;
use kernel::net::{self, Device, Napi, NapiPoller, NetdevTx, RtnlLinkStats64, SkBuff};
use kernel::prelude::*;
use kernel::{bindings, c_str, define_pci_id_table, device, dma, driver, irq, pci, PointerWrapper};
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    pci::MappedResource,
    sync::{Arc, ArcBorrow, CondVar, Mutex, UniqueArc},
};

#[macro_use]
mod log {
    macro_rules! info {
    ($($arg:tt)*) => (
        $crate::pr_info!($($arg)*);
    )
}
    macro_rules! debug (
    ($($arg:tt)*) => (
        $crate::pr_debug!($($arg)*)
    )
);
    macro_rules! warn (
    ($($arg:tt)*) => (
        $crate::pr_warn!($($arg)*)
    )
);
    macro_rules! error (
    ($($arg:tt)*) => (
        $crate::pr_err!($($arg)*)
    )
);
}

mod utils;
mod volatile;
pub use volatile::*;

mod e1000;
pub use e1000::*;
mod e1000_const;
pub use e1000_const::*;

module! {
    type: RustE1000dev,
    name: "rust_e1000dev",
    author: "Luoyuan Xiao",
    description: "Rust e1000 device driver",
    license: "GPL",
}

struct E1000Driver;

impl E1000Driver {
    fn myfn() {}
}

impl irq::Handler for E1000Driver {
    type Data = Box<u32>;

    fn handle_irq(_data: &u32) -> irq::Return {
        irq::Return::None
    }
}

// fn request_irq(irq: u32, data: Box<u32>) -> Result<irq::Registration<Example>> {
//     irq::Registration::try_new(irq, data, irq::flags::SHARED, fmt!("example_{irq}"))
// }

struct Poller;

impl NapiPoller for Poller {
    /// The pointer type that will be used to hold driver-defined data type.
    /// This must be same as DeviceOperations::Data.
    type Data = Box<NetData>;

    /// Corresponds to NAPI poll method.
    fn poll(
        napi: &Napi,
        budget: i32,
        dev: &Device,
        data: <Self::Data as PointerWrapper>::Borrowed<'_>,
    ) -> i32 {
        todo!()
    }
}

const E1000_REGS: u32 = 0x40000000; //?

struct Kernfn<T> {
    //dev: &'a dyn device::RawDevice,
    dev: Arc<device::Device>,
    alloc_coherent: Vec<dma::Allocation<T>>,
}

impl<T> e1000::KernelFunc for Kernfn<T> {
    const PAGE_SIZE: usize = 1 << 12;

    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        let alloc = dma::Allocation::<T>::try_new(
            &*self.dev,
            pages * Self::PAGE_SIZE,
            bindings::GFP_KERNEL,
        )
        .unwrap();

        let vaddr = alloc.cpu_addr as usize;
        let paddr = alloc.dma_handle as usize;
        self.alloc_coherent.try_push(alloc);
        pr_info!("Allocated vaddr: {:#x}, paddr: {:#x}\n", vaddr, paddr);

        (vaddr, paddr)
    }

    fn dma_free_coherent(&mut self, vaddr: usize, pages: usize) {
        pr_info!("Deallocating addr: {:#x}\n", vaddr);
        self.alloc_coherent.retain(|alloc| {
            if alloc.cpu_addr as usize == vaddr {
                drop(alloc);
                false
            } else {
                true
            }
        });
    }
}

struct NetData {
    dev: Arc<device::Device>,
    res: Arc<MappedResource>,
    //dev_e1000: Arc<E1000Device<'static, Kernfn<u8>>>,
    tx_ring: usize,
    rx_ring: usize,
    irq: u32,
    irq_handler: AtomicPtr<irq::Registration<E1000Driver>>,
}

#[vtable]
impl net::DeviceOperations for E1000Driver {
    /// The pointer type that will be used to hold driver-defined data type.
    type Data = Box<NetData>;

    /// Corresponds to `ndo_open` in `struct net_device_ops`.
    fn open(dev: &Device, data: &NetData) -> Result {
        pr_info!("Ethernet E1000 open\n");

        let mut kfn = Kernfn {
            dev: data.dev.clone(),
            alloc_coherent: Vec::new(),
        };
        let regs = data.res.ptr;
        let mut e1000_device = E1000Device::<Kernfn<u8>>::new(&mut kfn, regs).unwrap();

        let ping_frame: Box<[u8]> = Box::try_new([
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x52, 0x54, 0x00, 0x12, 0x34, 0x55, 0x08, 0x06,
            0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x52, 0x54, 0x00, 0x12, 0x34, 0x55,
            0xc0, 0xa8, 0x00, 0x7b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x00, 0x42,
        ])?; //ping 192.168.0.66
        e1000_device.e1000_transmit(&ping_frame);
        e1000_device.e1000_transmit(&ping_frame);

        Ok(())
    }

    /// Corresponds to `ndo_stop` in `struct net_device_ops`.
    fn stop(dev: &Device, data: <Self::Data as PointerWrapper>::Borrowed<'_>) -> Result {
        pr_warn!("net::DeviceOperations::stop() unimplemented!\n");
        Ok(())
    }

    /// Corresponds to `ndo_start_xmit` in `struct net_device_ops`.
    fn start_xmit(
        skb: &SkBuff,
        dev: &Device,
        data: <Self::Data as PointerWrapper>::Borrowed<'_>,
    ) -> NetdevTx {
        pr_info!("start xmit");

        net::NetdevTx::Ok
    }

    /// Corresponds to `ndo_get_stats64` in `struct net_device_ops`.
    fn get_stats64(
        _dev: &Device,
        _data: <Self::Data as PointerWrapper>::Borrowed<'_>,
        _storage: &mut RtnlLinkStats64,
    ) {
    }
}

struct DrvData {
    regist: net::Registration<E1000Driver>,
    bar_res: Arc<MappedResource>,
    irq: u32,
}

impl driver::DeviceRemoval for DrvData {
    fn device_remove(&self) {
        pr_info!("DrvData remove device driver from PCI\n");
    }
}

const DEVICE_ID_INTEL_82540EM: u32 = 0x100e;
const VENDOR_ID_INTEL_82540EM: u32 = 0x8086;
const MAC_HWADDR: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x55];

impl pci::Driver for E1000Driver {
    type Data = Box<DrvData>;

    fn probe(pci_dev: &mut pci::Device, id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        pr_info!("PCI Driver probing {:?}\n", id_info);

        pci_dev.enable_device();
        pci_dev.set_master();

        let bar_mask = pci_dev.select_bars(bindings::IORESOURCE_MEM as u64);
        pci_dev.request_selected_regions(bar_mask, c_str!("e1000"))?;

        let res0 = pci_dev.iter_resource().nth(0).unwrap();
        let bar_res = pci_dev.map_resource(&res0, res0.len())?;

        let irq = pci_dev.irq();
        pr_info!(
            "PCI MappedResource addr: {:#x}, len: {}, irq: {}\n",
            bar_res.ptr,
            res0.len(),
            irq
        );

        dma::set_mask(pci_dev, 0xffffffff)?;
        dma::set_coherent_mask(pci_dev, 0xffffffff)?;

        let mut regist = net::Registration::<E1000Driver>::try_new(pci_dev)?;
        let net_dev = regist.dev_get();
        net_dev.eth_hw_addr_set(&MAC_HWADDR);
        let dev = Arc::try_new(device::Device::from_dev(pci_dev))?;
        let bar_res = Arc::try_new(bar_res)?;
        let net_data = Box::try_new(NetData {
            dev,
            res: bar_res.clone(),
            tx_ring: 0,
            rx_ring: 0,
            irq,
            irq_handler: AtomicPtr::new(core::ptr::null_mut()),
        })?;
        regist.register(net_data)?; // ip link show

        Ok(Box::try_new(DrvData {
            regist,
            bar_res: bar_res.clone(),
            irq,
        })?)
    }
    fn remove(_data: &Self::Data) {
        pr_info!("PCI Driver remove\n");
    }
    define_pci_id_table! {u32, [
        (pci::DeviceId::new(VENDOR_ID_INTEL_82540EM, DEVICE_ID_INTEL_82540EM), Some(0x1)),
    ]}
}

struct RustE1000dev {
    dev: Pin<Box<driver::Registration<pci::Adapter<E1000Driver>>>>,
}

impl kernel::Module for RustE1000dev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust e1000 device driver (init)\n");

        let dev = driver::Registration::<pci::Adapter<E1000Driver>>::new_pinned(name, module)?;
        Ok(RustE1000dev { dev })
    }
}

impl Drop for RustE1000dev {
    fn drop(&mut self) {
        pr_info!("Rust e1000 device driver (exit)\n");
    }
}
