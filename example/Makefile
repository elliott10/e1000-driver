.PHONY: all run clean run-nvme

TARGET      := riscv64imac-unknown-none-elf
KERNEL_FILE := target/$(TARGET)/release/example
KERNEL_BIN	:= kernel-qemu
DEBUG_FILE  ?= $(KERNEL_FILE)
FWDPORT 	:= 6200
SERVER_PORT := 6201

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

all:
	@rm -f kernel-qemu
	cargo build --release
	cp $(KERNEL_FILE) kernel-qemu

run: all
	@qemu-system-riscv64 \
		-machine virt \
		-nographic \
		-kernel $(KERNEL_BIN) \
		-netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
		-device e1000,netdev=net0,bus=pcie.0
# -netdev user,id=net0,hostfwd=udp::$(FWDPORT)-:2000 -object filter-dump,id=net0,netdev=net0,file=packets.pcap \
# -device virtio-net-device,netdev=net0

ping: 
	python3 ping.py $(FWDPORT)

server:
	python3 server.py $(SERVER_PORT)

.PHONY: all run ping server