// Simple PCI-Express for qemu and its e1000 ethernet
use core::slice::from_raw_parts_mut;
use volatile::Volatile;
use log::*;

// E1000 registers which were mapped
pub const E1000_REGS: u32 = 0x40000000;
// Qemu virt PCIe config space
pub const ECAM: u32 = 0x30000000;

pub fn pci_init() {
    // Look at each PCI device on bus 0
    for dev in 0..32 {
        let bus = 0;
        let func = 0;
        let offset = 0;

        let off: u32 = (bus << 16) | (dev << 11) | (func << 8) | offset;
        let base = ECAM + off;
        let pci_base = base as *mut Volatile<u32>;
        let deve_id = unsafe{ (*pci_base).read() };
        trace!("PCI device id: {:#x} @ {:#x}", deve_id, base);

        // E1000 ID = 100e:8086
        if deve_id == 0x100e8086 {
            info!("PCI Found device id: {:#x}", deve_id);
            let pci_config = unsafe { from_raw_parts_mut(base as *mut Volatile<u32>, 0xff / 4) };

            // Enable I/O access, memory access, mastering
            pci_config[1].write(0x7);
            //sync

            for bar in 0..6 {
                let old: u32 = pci_config[4 + bar].read();

                // Writing all 1's to the BAR causes it to be replaced with its size.
                pci_config[4 + bar].write(0xffffffff);
                //sync

                pci_config[4 + bar].write(old);
            }

            // To reveal e1000 registers at E1000_REGS;
            pci_config[4 + 0].write(E1000_REGS);

            //e1000_init((*mut u32)E1000_REGS);
        }
    }
}
