obj-m := e1000_for_linux.o

PWD := $(shell pwd)
ARCH := riscv
KDIR ?= /lib/modules/$(shell uname -r)/build

default:
	@[ -e ../Makefile ] || ln -s linux/Makefile ../
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD)/../ modules
clean:
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD)/../ clean
	@[ -e ../Makefile ] && rm ../Makefile
