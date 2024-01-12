obj-m := e1000_for_linux.o

PWD := $(shell pwd)
ARCH ?= x86_64
KDIR ?= ~/kernel_dev/linux-rust-v6.1.66/

default:
	@[ -e ../Makefile ] || ln -s linux/Makefile ../
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD)/../ modules
clean:
	$(MAKE) ARCH=$(ARCH) LLVM=1 -C $(KDIR) M=$(PWD)/../ clean
	@[ -e ../Makefile ] && rm ../Makefile
