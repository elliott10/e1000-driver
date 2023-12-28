//! Rust e1000 network device.
#![allow(unused)]

use core::slice::from_raw_parts_mut;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use kernel::prelude::*;
use kernel::{
    bindings, c_str, define_pci_id_table, device, dma, driver,
    file::{self, File},
    io_buffer::{IoBufferReader, IoBufferWriter},
    irq,
    net::{self, Device, Napi, NapiPoller, NetdevTx, RtnlLinkStats64, SkBuff},
    pci,
    pci::MappedResource,
    spinlock_init,
    sync::{Arc, ArcBorrow, CondVar, SpinLock, UniqueArc},
    ForeignOwnable,
};

#[macro_use]
mod linux;
mod e1000;
mod utils;
pub use e1000::*;
pub use linux::volatile::*;

/// Expand Vec and slice functions
pub trait Ext<T> {
    /// Vec function with_capacity
    fn with_capacity(capacity: usize) -> Self
    where
        Self: Sized,
    {
        unimplemented!()
    }
    /// Vec function push
    fn push(&mut self, value: T) {
        unimplemented!()
    }
    /// slice function to_vec
    fn to_vec(&self) -> Vec<T> {
        unimplemented!()
    }
}

impl<T> Ext<T> for Vec<T> {
    fn with_capacity(capacity: usize) -> Self {
        alloc::vec::Vec::try_with_capacity(capacity).unwrap()
    }

    fn push(&mut self, value: T) {
        self.try_push(value);
    }
}

impl<T: Clone> Ext<T> for [T] {
    fn to_vec(&self) -> Vec<T> {
        self.try_to_vec().unwrap()
    }
}

const RXBUFFER: u32 = 2048;
/// Intel E1000 ID
const VENDOR_ID_INTEL: u32 = 0x8086;
const DEVICE_ID_INTEL_I219: u32 = 0x15fc;
const DEVICE_ID_INTEL_82540EM: u32 = 0x100e;
const DEVICE_ID_INTEL_82574L: u32 = 0x10d3;
//const MAC_HWADDR: [u8; 6] = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
//const MAC_HWADDR: [u8; 6] = [0x90, 0xe2, 0xfc, 0xb5, 0x36, 0x95];
const MAC_HWADDR: [u8; 6] = [0x52, 0x54, 0x00, 0x6c, 0xf8, 0x88];

module! {
    type: RustE1000dev,
    name: "rust_e1000dev",
    author: "Luoyuan Xiao",
    description: "Rust e1000 device driver",
    license: "GPL",
}

struct E1000Driver;

impl E1000Driver {
    fn handle_rx_irq(dev: &net::Device, napi: &Napi, data: &NetData) {
        let mut packets = 0;
        let mut bytes = 0;

        let recv_vec = {
            let mut dev_e1k = data.dev_e1000.lock_irqdisable();
            dev_e1k.as_mut().unwrap().e1000_recv()
        };

        if let Some(vec) = recv_vec {
            packets = vec.len();
            for (_i, packet) in vec.iter().enumerate() {
                let mut len = packet.len();
                //bytes += len;

                let skb = dev.alloc_skb_ip_align(RXBUFFER).unwrap();
                let skb_buf =
                    unsafe { from_raw_parts_mut(skb.head_data().as_ptr() as *mut u8, len) };
                skb_buf.copy_from_slice(&packet);

                //len -= 4; // ?
                skb.put(len as u32);
                let protocol = skb.eth_type_trans(dev);
                skb.protocol_set(protocol);

                // Send the skb up the stack
                napi.gro_receive(&skb);

                bytes += len;

                //pr_info!("Sk Buff protocol: {:#x}, [01] = {:x}{:x}\n", protocol, skb_buf[0], skb_buf[1]);
            }
            info!(
                "handle_rx_irq, received packets: {}, bytes: {}\n",
                packets,
                bytes
            );
        } else {
            pr_warn!("None packets were received\n");
        }

        data.stats
            .rx_bytes
            .fetch_add(bytes as u64, Ordering::Relaxed);
        data.stats
            .rx_packets
            .fetch_add(packets as u64, Ordering::Relaxed);
    }

    fn handle_tx_irq() {
        // check status E1000_TXD_STAT_DD
    }
}

struct IrqData {
    dev_e1000: Arc<SpinLock<Option<E1000Device<'static, Kernfn<u8>>>>>,
    res: Arc<MappedResource>,
    napi: Arc<net::Napi>,
}

impl irq::Handler for E1000Driver {
    type Data = Box<IrqData>;

    fn handle_irq(data: &IrqData) -> irq::Return {
        info!("handle_irq\n");
        let intr = {
            let mut dev_e1k = data.dev_e1000.lock_irqdisable();
            dev_e1k.as_mut().unwrap().e1000_intr()
        };
        /*
        let intr = unsafe {
            let ptr = data.res.ptr.wrapping_add(0xC0); // ICR
            bindings::readl(ptr as *const u32 as _)
        };
        */
        info!("irq::Handler E1000_ICR = {:#x}\n", intr);

        if (intr & (1<<7)) == 0 {
            pr_warn!("No valid e1000 interrupt was found\n");
            return irq::Return::None;
        }

        data.napi.schedule();

        irq::Return::Handled
    }
}

fn request_irq(irq: u32, data: Box<IrqData>) -> Result<irq::Registration<E1000Driver>> {
    irq::Registration::try_new(irq, data, irq::flags::SHARED, fmt!("e1000_{irq}"))
}

struct Poller;

impl NapiPoller for Poller {
    /// The pointer type that will be used to hold driver-defined data type.
    /// This must be same as DeviceOperations::Data.
    type Data = Box<NetData>;

    /// Corresponds to NAPI poll method.
    fn poll(napi: &Napi, budget: i32, dev: &net::Device, data: &NetData) -> i32 {
        info!("NapiPoller poll\n");

        E1000Driver::handle_rx_irq(dev, napi, data);
        E1000Driver::handle_tx_irq();

        napi.complete_done(1);
        1
    }
}

struct Stats64 {
    rx_bytes: AtomicU64,
    rx_packets: AtomicU64,
    tx_bytes: AtomicU64,
    tx_packets: AtomicU64,
}

impl Stats64 {
    fn new() -> Self {
        Stats64 {
            rx_bytes: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
        }
    }
}

struct Kernfn<T> {
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
        pr_info!("Allocated {} pages, vaddr: {:#x}, paddr: {:#x}\n", pages, vaddr, paddr);

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

unsafe impl Send for NetData {}
unsafe impl Sync for NetData {}

struct NetData {
    dev: Arc<device::Device>,
    res: Arc<MappedResource>,
    dev_e1000: Arc<SpinLock<Option<E1000Device<'static, Kernfn<u8>>>>>,
    stats: Stats64,
    napi: Arc<net::Napi>,
    irq: Option<u32>,
    irq_handler: AtomicPtr<irq::Registration<E1000Driver>>,
}

#[vtable]
impl net::DeviceOperations for E1000Driver {
    /// The pointer type that will be used to hold driver-defined data type.
    type Data = Box<NetData>;

    /// Corresponds to `ndo_open` in `struct net_device_ops`.
    fn open(dev: &Device, data: &NetData) -> Result {
        pr_info!("Ethernet E1000 open\n");

        let kfn = Kernfn {
            dev: data.dev.clone(),
            alloc_coherent: Vec::new(),
        };
        let regs = data.res.ptr;
        let mut e1000_device = E1000Device::<Kernfn<u8>>::new(kfn, regs).unwrap();

        pr_info!("e1000 device is initialized\n");
        {
            let mut dev_e1k = data.dev_e1000.lock();
            *dev_e1k = Some(e1000_device);
        }

        let irq_data = Box::try_new(IrqData {
            dev_e1000: data.dev_e1000.clone(),
            res: data.res.clone(),
            napi: data.napi.clone(),
        })?;
        let irq_regist = request_irq(data.irq.unwrap(), irq_data)?;
        // 注意把申请的irq放入Box中，其他线程才能handle中断
        data.irq_handler
            .store(Box::into_raw(Box::try_new(irq_regist)?), Ordering::Relaxed);

        // Enable NAPI scheduling
        data.napi.enable();

        dev.netif_start_queue();

        {
            let mut dev_e1k = data.dev_e1000.lock_irqdisable();
            let e1k_fn = dev_e1k.as_mut().unwrap();

            // enable rx int
            e1k_fn.e1000_irq_enable();

            /* fire a link status change interrupt to start the watchdog */
            e1k_fn.e1000_cause_lsc_int();
            /* 或这种方式触发LSC中断
            unsafe {
                let ptr = data.res.ptr.wrapping_add(0xc8); // ICS
                bindings::writel(4, ptr as _);
            }
            */
        };

        // watchdog handler ?
        // Enable net interface
        dev.netif_carrier_on();

        Ok(())
    }

    /// Corresponds to `ndo_start_xmit` in `struct net_device_ops`.
    fn start_xmit(
        skb: &SkBuff,
        dev: &Device,
        data: <Self::Data as ForeignOwnable>::Borrowed<'_>,
    ) -> NetdevTx {
        info!("start xmit\n");

        skb.put_padto(bindings::ETH_ZLEN);
        let _size = skb.len() - skb.data_len();
        let skb_data = skb.head_data();

        info!(
            "SkBuff length: {}, head data len: {}, get size: {}\n",
            skb.len(),
            skb_data.len(),
            _size
        );

        dev.sent_queue(skb.len());

        let len = {
            let mut dev_e1k = data.dev_e1000.lock_irqdisable();
            dev_e1k.as_mut().unwrap().e1000_transmit(skb_data)
        };

        if len < 0 {
            pr_warn!("Failed to send transmit the skbuff packet: {}", len);
            return net::NetdevTx::Busy;
        }

        // when clean_tx_irq
        {
            let bytes = skb.len() as u64;
            let packets = 1;

            skb.napi_consume(64);

            data.stats.tx_bytes.fetch_add(bytes, Ordering::Relaxed);
            data.stats.tx_packets.fetch_add(packets, Ordering::Relaxed);

            dev.completed_queue(packets as u32, bytes as u32);
        }

        net::NetdevTx::Ok
    }

    /// Corresponds to `ndo_get_stats64` in `struct net_device_ops`.
    fn get_stats64(_dev: &Device, data: &NetData, stats: &mut RtnlLinkStats64) {
        info!("get stats64\n");

        stats.set_rx_bytes(data.stats.rx_bytes.load(Ordering::Relaxed));
        stats.set_rx_packets(data.stats.rx_packets.load(Ordering::Relaxed));
        stats.set_tx_bytes(data.stats.tx_bytes.load(Ordering::Relaxed));
        stats.set_tx_packets(data.stats.tx_packets.load(Ordering::Relaxed));
    }

    /// Corresponds to `ndo_stop` in `struct net_device_ops`.
    fn stop(dev: &Device, data: <Self::Data as ForeignOwnable>::Borrowed<'_>) -> Result {
        pr_info!("net::DeviceOperations::stop\n");
        dev.netif_carrier_off();
        let mut dev_e1k = data.dev_e1000.lock_irqdisable();
        dev_e1k.as_mut().unwrap().e1000_irq_disable();

        dev.netif_stop_queue();
        data.napi.disable();

        let irq_ptr = data.irq_handler.load(Ordering::Relaxed);
        unsafe{ drop(Box::from_raw(irq_ptr)); }

        drop(data);
        Ok(())
    }
}

struct DrvData {
    regist: net::Registration<E1000Driver>,
    bar_res: Arc<MappedResource>,
    bar_mask: i32,
    irq: Option<u32>,
}

impl driver::DeviceRemoval for DrvData {
    fn device_remove(&self) {
        pr_info!("DrvData remove device driver from PCI\n");
    }
}

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
            irq.unwrap_or_default(),
        );

        dma::set_mask(pci_dev, 0xffffffff)?;
        dma::set_coherent_mask(pci_dev, 0xffffffff)?;

        let mut regist = net::Registration::<E1000Driver>::try_new(pci_dev)?;
        let net_dev = regist.dev_get();
        net_dev.eth_hw_addr_set(&MAC_HWADDR);
        let dev = Arc::try_new(device::Device::from_dev(pci_dev))?;
        let bar_res = Arc::try_new(bar_res)?;
        /*
        let mut dev_e1000 = Pin::from(Box::try_new(unsafe { SpinLock::new(None) })?);
        spinlock_init!(dev_e1000.as_mut(), "e1000_device");
        */
        let mut lock_e1000 = unsafe { SpinLock::new(None) };
        spinlock_init!(
            unsafe { Pin::new_unchecked(&mut lock_e1000) },
            "e1000_device"
        );
        let mut dev_e1000 = Arc::try_new(lock_e1000)?;

        let napi = net::NapiAdapter::<Poller>::add_weight(&net_dev, 64)?;
        net_dev.netif_carrier_off();

        let net_data = Box::try_new(NetData {
            dev,
            res: bar_res.clone(),
            dev_e1000,
            stats: Stats64::new(),
            napi: napi.into(),
            irq,
            irq_handler: AtomicPtr::new(core::ptr::null_mut()),
        })?;
        regist.register(net_data)?; // ip link show

        Ok(Box::try_new(DrvData {
            regist,
            bar_res: bar_res.clone(),
            bar_mask,
            irq,
        })?)
    }
    fn remove(pci_dev: &mut pci::Device, data: &Self::Data) {
        pr_info!("PCI Driver remove\n");
        pci_dev.release_selected_regions(data.bar_mask);
        drop(data);
    }
    define_pci_id_table! {u32, [
        (pci::DeviceId::new(VENDOR_ID_INTEL, DEVICE_ID_INTEL_82540EM), Some(0x1)),
        (pci::DeviceId::new(VENDOR_ID_INTEL, DEVICE_ID_INTEL_82574L), Some(0x1)),
        (pci::DeviceId::new(VENDOR_ID_INTEL, DEVICE_ID_INTEL_I219), Some(0x1)),
    ]}
}

struct RustE1000dev {
    dev: Pin<Box<driver::Registration<pci::Adapter<E1000Driver>>>>,
}

impl kernel::Module for RustE1000dev {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {

        pr_info!(r"
 ____            _      __              _     _                  
|  _ \ _   _ ___| |_   / _| ___  _ __  | |   (_)_ __  _   ___  __
| |_) | | | / __| __| | |_ / _ \| '__| | |   | | '_ \| | | \ \/ /
|  _ <| |_| \__ \ |_  |  _| (_) | |    | |___| | | | | |_| |>  < 
|_| \_\\__,_|___/\__| |_|  \___/|_|    |_____|_|_| |_|\__,_/_/\_\
                                                                 
 ____       _                    
|  _ \ _ __(_)_   _____ _ __ ___ 
| | | | '__| \ \ / / _ \ '__/ __|
| |_| | |  | |\ V /  __/ |  \__ \
|____/|_|  |_| \_/ \___|_|  |___/
                                 
");
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
