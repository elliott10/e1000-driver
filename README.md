# E1000 Driver
E1000 driver in Rust for the Intel 82540EP/EM Gigabit Ethernet.

## Support features
* E1000 driver for RISCV on Qemu is supported
  - networking protocol support: IP, ARP, UDP

## Example

```
cd examples/riscv
make run
```

## Reference
* Linux source code
* [xv6: Implementation of net](https://github.com/mit-pdos/xv6-riscv-fall19/tree/net)
