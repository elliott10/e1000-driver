# E1000 Driver
E1000 driver in Rust for the Intel PCIE Gigabit Ethernet.

## Support features
* `e1000` and `e1000e` driver on Qemu and a Physical Industrial Computer(i217)
* Initialize simple PCI-Express for e1000 device
* Implement the e1000 driver as a linux driver module
* MSIX、MSI supported
* Bridge mode supported


## Quick start on bare metal OS

For instance, [E1000 driver in Rust on ArceOS/Starry](https://github.com/elliott10/arceos/blob/net-e1000/crates/driver_net/src/e1000.rs)

![e1000 on arceos](doc/img/arceos-starry-e1000.png)

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

test on linux kernel 6.6

```
enable kernel rust support

cd src

make ARCH=<cpu arch> KDIR=<path to linux> 
# e.g. make ARCH=x86_64 KDIR=/home/rust/linux/build
```

* `e1000-driver` runs on Linux

```
[root@test ~]# lsmod | grep e1000
e1000_rs              147456  0
[root@test ~]# ip a
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    link/loopback 00:00:00:00:00:00 brd 00:00:00:00:00:00
    inet 127.0.0.1/8 scope host lo
       valid_lft forever preferred_lft forever
    inet6 ::1/128 scope host
       valid_lft forever preferred_lft forever
2: eno1: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel master bridge0 state UP group default qlen 1000
    link/ether 88:88:88:88:87:88 brd ff:ff:ff:ff:ff:ff
3: bridge0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether 88:88:88:88:87:88 brd ff:ff:ff:ff:ff:ff
    inet 10.3.10.39/24 brd 10.3.10.255 scope global dynamic noprefixroute bridge0
       valid_lft 84793sec preferred_lft 84793sec
    inet6 240c:caa1:3:1310::43/128 scope global dynamic noprefixroute
       valid_lft 5890sec preferred_lft 5590sec
    inet6 240c:caa1:3:1310:895d:2e48:42:4059/64 scope global dynamic noprefixroute
       valid_lft 2591715sec preferred_lft 604515sec
    inet6 fe80::f0a:6045:2ebc:1eb4/64 scope link noprefixroute
       valid_lft forever preferred_lft forever
4: virbr0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc noqueue state UP group default qlen 1000
    link/ether 52:54:00:88:84:f3 brd ff:ff:ff:ff:ff:ff
    inet 192.168.122.1/24 brd 192.168.122.255 scope global virbr0
       valid_lft forever preferred_lft forever
5: virbr0-nic: <BROADCAST,MULTICAST> mtu 1500 qdisc fq_codel master virbr0 state DOWN group default qlen 1000
    link/ether 52:54:00:88:84:f3 brd ff:ff:ff:ff:ff:ff
6: vnet0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel master virbr0 state UNKNOWN group default qlen 1000
    link/ether fe:54:00:3c:3d:5b brd ff:ff:ff:ff:ff:ff
    inet6 fe80::fc54:ff:fe3c:3d5b/64 scope link
       valid_lft forever preferred_lft forever
7: vnet1: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel master bridge0 state UNKNOWN group default qlen 1000
    link/ether fe:54:00:3c:3d:5a brd ff:ff:ff:ff:ff:ff
    inet6 fe80::fc54:ff:fe3c:3d5a/64 scope link
       valid_lft forever preferred_lft forever
[root@test ~]# ping www.baidu.com
PING www.a.shifen.com (182.61.200.7) 56(84) 字节的数据。
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=1 ttl=48 时间=9.08 毫秒
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=2 ttl=48 时间=8.84 毫秒
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=3 ttl=48 时间=8.87 毫秒
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=4 ttl=48 时间=8.92 毫秒
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=5 ttl=48 时间=8.97 毫秒
64 字节，来自 182.61.200.7 (182.61.200.7): icmp_seq=6 ttl=48 时间=8.96 毫秒
^C64 字节，来自 182.61.200.7: icmp_seq=7 ttl=48 时间=8.92 毫秒

[root@test ~]# wget https://mirror.sjtu.edu.cn/openeuler/openEuler-22.03-LTS-SP4/ISO/x86_64/openEuler-22.03-LTS-SP4-x86_64-dvd.iso
--2024-07-12 14:56:09--  https://mirror.sjtu.edu.cn/openeuler/openEuler-22.03-LTS-SP4/ISO/x86_64/openEuler-22.03-LTS-SP4-x86_64-dvd.iso
正在解析主机 mirror.sjtu.edu.cn (mirror.sjtu.edu.cn)... 2001:da8:8000:7100::322:a, 111.186.58.212
正在连接 mirror.sjtu.edu.cn (mirror.sjtu.edu.cn)|2001:da8:8000:7100::322:a|:443... 已连接。
已发出 HTTP 请求，正在等待回应... 200 OK
长度：4038066176 (3.8G) [application/octet-stream]
正在保存至: “openEuler-22.03-LTS-SP4-x86_64-dvd.iso”

dvd.iso                         5%[=>                                                ] 223.02M  24.4MB/s  剩余 2m 30s
```

## Reference
* Linux source code
* [xv6: Implementation of net](https://github.com/mit-pdos/xv6-riscv-fall19/tree/net)
* [MIT 6.828/2019/networking](https://pdos.csail.mit.edu/6.828/2019/lec/l-networking.pdf)
* [Intel Gigabit Ethernet 82540EP/EM](https://pdos.csail.mit.edu/6.828/2019/readings/hardware/8254x_GBe_SDM.pdf)
* [OSDev: Intel 8254x](https://wiki.osdev.org/Intel_8254x)
* [Rust for Linux](https://github.com/fujita/linux/tree/rust-e1000)
* [Kernel threads: Rust e1000 driver (Intel Ethernet adapter)](https://lore.kernel.org/rust-for-linux/20220919.103820.680182888079022491.fujita@lima-default/)
