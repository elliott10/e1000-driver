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

## Rust e1000 driver for Linux kernel module
```
cd src/linux
make ARCH=<cpu arch> KDIR=<path to linux>
# e.g. make ARCH=x86_64 KDIR=/home/rust/linux/build
```

## Reference
* Linux source code
* [xv6: Implementation of net](https://github.com/mit-pdos/xv6-riscv-fall19/tree/net)
* [MIT 6.828/2019/networking](https://pdos.csail.mit.edu/6.828/2019/lec/l-networking.pdf)
* [Intel Gigabit Ethernet 82540EP/EM](https://pdos.csail.mit.edu/6.828/2019/readings/hardware/8254x_GBe_SDM.pdf)
* [OSDev: Intel 8254x](https://wiki.osdev.org/Intel_8254x)
* [Rust for Linux](https://github.com/fujita/linux/tree/rust-e1000)
* [Kernel threads: Rust e1000 driver (Intel Ethernet adapter)](https://lore.kernel.org/rust-for-linux/20220919.103820.680182888079022491.fujita@lima-default/)
