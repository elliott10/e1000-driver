// SPDX-License-Identifier: GPL-2.0
//! Rust e1000 network device.
use kernel::prelude::*;
use kernel::{
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    sync::{Arc, ArcBorrow, CondVar, Mutex, UniqueArc},
};
use kernel::{driver, irq, PointerWrapper};
use kernel::{pci, define_pci_id_table};
use kernel::net::{self, Device, NapiPoller, Napi, SkBuff, NetdevTx, RtnlLinkStats64};
use core::sync::atomic::AtomicPtr;

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

struct NetData {
    tx_ring: usize,
    rx_ring: usize,
    irq: AtomicPtr<irq::Registration<E1000Driver>>,
}

#[vtable]
impl net::DeviceOperations for E1000Driver {
        /// The pointer type that will be used to hold driver-defined data type.
        type Data = Arc<NetData>;

        /// Corresponds to `ndo_open` in `struct net_device_ops`.
        fn open(dev: &Device, data: <Self::Data as PointerWrapper>::Borrowed<'_>) -> Result {
        todo!()

        }
    
        /// Corresponds to `ndo_stop` in `struct net_device_ops`.
        fn stop(dev: &Device, data: <Self::Data as PointerWrapper>::Borrowed<'_>) -> Result {
        todo!()

        }
    
        /// Corresponds to `ndo_start_xmit` in `struct net_device_ops`.
        fn start_xmit(
            skb: &SkBuff,
            dev: &Device,
            data: <Self::Data as PointerWrapper>::Borrowed<'_>,
        ) -> NetdevTx {
        todo!()

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
    regs: net::Registration<E1000Driver>,
}

impl driver::DeviceRemoval for DrvData {
    fn device_remove(&self) {
        pr_info!("DrvData remove device driver for e1000\n");
    }
}

const DEVICE_ID_INTEL_82540EM: u32 = 0x100e;
const VENDOR_ID_INTEL_82540EM: u32 = 0x8086;
impl pci::Driver for E1000Driver {
    type Data  = Arc<DrvData>;
    
    fn probe(pci_dev: &mut pci::Device, id_info: Option<&Self::IdInfo>) -> Result<Self::Data> {
        pr_info!("PCI Driver probing {:?}", id_info);

        
        todo!()
    }
    fn remove(_data: &Self::Data){
        todo!()
    }
    define_pci_id_table! {u32, [
        (pci::DeviceId::new(VENDOR_ID_INTEL_82540EM, DEVICE_ID_INTEL_82540EM), None),
    ]}
}

struct RustE1000dev {
    dev: Pin<Box<driver::Registration<pci::Adapter<E1000Driver>>>>,
}

impl kernel::Module for RustE1000dev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust e1000 device driver (init)\n");

        let dev = driver::Registration::<pci::Adapter<E1000Driver>>::new_pinned(name, module)?;
        Ok(RustE1000dev {
            dev
        })
    }
}

impl Drop for RustE1000dev {
    fn drop(&mut self) {
        pr_info!("Rust e1000 device driver (exit)\n");
    }
}
