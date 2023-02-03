# E1000 Driver
E1000 driver in Rust for the Intel 82540EP/EM Gigabit Ethernet.

## Support features
* E1000 driver for RISCV on Qemu is supported
* Initialize simple PCI-Express for e1000 device

  - Todo: networking protocol support: IP, ARP, UDP

## How to use

Initialize the E1000 driver
```
let mut e1000dev = e1000::E1000Device::new(base);
```

Sending network packets
```
e1000dev.e1000_transmit(packet);
```

Receiving network packets
```
e1000dev.e1000_recv();
```

## Example

```
cd examples/riscv
make run
```

## Reference
* Linux source code
* [xv6: Implementation of net](https://github.com/mit-pdos/xv6-riscv-fall19/tree/net)
