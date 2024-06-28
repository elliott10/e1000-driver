obj-m := e1000_rs.o
e1000_rs-y := linux/e1000_main.o platform_linux.o 


KDIR ?= /lib/modules/$(shell uname -r)/build
MDIR ?= $(PWD)
INSTALL_MOD_PATH ?= $(PWD)/_install

default:
	make LLVM=1 -C $(KDIR) M=$(MDIR) modules

install:
	make LLVM=1 -C $(KDIR) M=$(MDIR) modules_install INSTALL_MOD_PATH=$(INSTALL_MOD_PATH) INSTALL_MOD_STRIP=1

rust-analyzer:
	make LLVM=1 -C $(KDIR) M=$(MDIR) rust-analyzer

clean:
	make LLVM=1 -C $(KDIR) M=$(MDIR) clean