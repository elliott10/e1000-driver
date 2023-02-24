# E1000 Driver
E1000 driver in Rust for the Intel 82540EP/EM Gigabit Ethernet.

## Support features
* E1000 driver for RISCV on Qemu is supported
* Initialize simple PCI-Express for e1000 device
* Implement the e1000 driver as a linux driver module

- _Todo: networking protocol support: IP, ARP, UDP_

## Quick start on bare metal OS

Initialize PCI and E1000 driver
```
pub struct Kernfn;
impl e1000_driver::e1000::KernelFunc for Kernfn { ... }

e1000_driver::pci::pci_init();

let mut e1000_device = e1000_driver::e1000::E1000Device::<Kernfn>::new(e1000_driver::pci::E1000_REGS as usize).unwrap();
```

Sending network packets
```
e1000_device.e1000_transmit(&frame);
```

Receiving network packets
```
let rx_buf = e1000_device.e1000_recv();
```

## Kernel module on Linux
```
cd src/linux
make ARCH=arch KDIR=/path/to/linux
# e.g. make ARCH=riscv KDIR=/home/rust/linux
```

## Reference
* Linux source code
* [xv6: Implementation of net](https://github.com/mit-pdos/xv6-riscv-fall19/tree/net)
* [Rust for Linux](https://github.com/fujita/linux/tree/rust-e1000)
