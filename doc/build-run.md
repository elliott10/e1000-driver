# Rust重构Intel e1000e网卡驱动工作

### 支持了Rust网卡驱动相关API的Kernel重编译的部署
网卡驱动e1000e进行了Rust语言重构的代码仓库：
https://github.com/elliott10/e1000-driver.git

Rust for Linux内核代码仓库：
https://github.com/elliott10/linux/commits/rust-v6.1.66/

### 内核支持Rust
内核配置文件需要用物理机自带的，如`/boot/config-6.1.19-xxx`, 复制到内核源码`arch/x86/configs/qcl_defconfig`
```
make ARCH=x86_64 LLVM=1 O=build qcl_defconfig
make ARCH=x86_64 LLVM=1 O=build menuconfig # 自定义内核配置，如打开Rust Support；
make ARCH=x86_64 LLVM=1 O=build
make ARCH=x86_64 LLVM=1 O=build modules_install INSTALL_MOD_PATH=modules_install INSTALL_MOD_STRIP=1
```

### 内核及驱动模块部署
* 将编译生成的`build/arch/x86_64/boot/bzImage`内核文件部署到物理机器上`/boot`;
* 内核模块`build/modules_install/lib/modules/6.1.66+`文件部署到物理机器上`/lib/modules`;
* **重新生成`initramfs`**
内核和模块文件生成后，在物理机器上执行如下命令重新创建`initramfs`：
```
NEW_KERN_VERSION="6.1.66+"
dracut /boot/initramfs-${NEW_KERN_VERSION}.img ${NEW_KERN_VERSION}
```
### 更新Grub内核启动项
最后在文件`/boot/efi/EFI/openEuler/grub.cfg`的menuentry项中添加对应的`linux`和`initrd`项；
去除Grub密码，则需要注释`grub.cfg`文件中的`password_pbkdf2`和`superusers`项；

### 运行
在进行了Linux 6.1.66内核升级；对网卡驱动e1000e进行了Rust语言重构；运行加载Rust重构的网卡驱动，进行网络功能测试，测试结果功能显示正常。如下图
![e1000e-run](https://github.com/rcore-os/e1000-driver/assets/8327091/22f44766-cd30-4813-a700-31e42e0fee70)
