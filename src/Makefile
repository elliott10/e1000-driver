obj-m := rust_e1000_for_linux.o

PWD := $(shell pwd)
ARCH := riscv
KDIR ?= /lib/modules/$(shell uname -r)/build

KDIR := /home/xx/linux/linux

default:
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD) modules
clean:
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD) clean
