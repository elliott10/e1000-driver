#![allow(non_snake_case)]

pub(crate) const CTRL: u32 = 0x00000; // Device Control - RW
pub(crate) const STATUS: u32 = 0x00008; // Device Status - RO
pub(crate) const EECD: u32 = 0x00010; // EEPROM/Flash Control - RW
pub(crate) const EERD: u32 = 0x00014; // EEPROM Read - RW
pub(crate) const CTRL_EXT: u32 = 0x00018; // Extended Device Control - RW
pub(crate) const FLA: u32 = 0x0001C; // Flash Access - RW
pub(crate) const MDIC: u32 = 0x00020; // MDI Control - RW
pub(crate) const SCTL: u32 = 0x00024; // SerDes Control - RW
pub(crate) const FCAL: u32 = 0x00028; // Flow Control Address Low - RW
pub(crate) const FCAH: u32 = 0x0002C; // Flow Control Address High - RW
                                      // 注意：FEXT 和 E1000_FCAH 有相同的值，需要确认是否是故意的
pub(crate) const FEXT: u32 = 0x0002C; // Future Extended - RW

pub(crate) const FCT: u32 = 0x00030; // Flow Control Type - RW
pub(crate) const VET: u32 = 0x00038; // VLAN Ether Type - RW
pub(crate) const ICR: u32 = 0x000C0; // Interrupt Cause Read - R/clr
pub(crate) const ITR: u32 = 0x000C4; // Interrupt Throttling Rate - RW
pub(crate) const ICS: u32 = 0x000C8; // Interrupt Cause Set - WO
pub(crate) const IMS: u32 = 0x000D0; // Interrupt Mask Set - RW
pub(crate) const IMC: u32 = 0x000D8; // Interrupt Mask Clear - WO
pub(crate) const IAM: u32 = 0x000E0; // Interrupt Acknowledge Auto Mask
pub(crate) const IVAR: u32 = 0x000E4; // Interrupt Vector Allocation Register - RW
pub(crate) const PBA: u32 = 0x01000; /* Packet Buffer Allocation - RW */
pub(crate) const PBS: u32 = 0x01008; /* Packet Buffer Size */
pub(crate) const EEWR: u32 = 0x0102C; /* EEPROM Write Register - RW */

pub(crate) const PCS_LCTL: u32 = 0x04208; // PCS Link Control - RW
pub(crate) const PCS_LSTAT: u32 = 0x0420C; // PCS Link Status - RO
pub(crate) const PCS_ANADV: u32 = 0x04218; // AN advertisement - RW
pub(crate) const PCS_LPAB: u32 = 0x0421C; // Link Partner Ability - RW
pub(crate) const RXCSUM: u32 = 0x05000; // Rx Checksum Control - RW
pub(crate) const RFCTL: u32 = 0x05008; // Receive Filter Control
pub(crate) const MTA: u32 = 0x05200; // Multicast Table Array - RW Array
pub(crate) const RA: u32 = 0x05400; // Receive Address - RW Array
pub(crate) const VFTA: u32 = 0x05600; // VLAN Filter Table Array - RW Array
pub(crate) const WUC: u32 = 0x05800; // Wakeup Control - RW
pub(crate) const WUFC: u32 = 0x05808; // Wakeup Filter Control - RW
pub(crate) const WUS: u32 = 0x05810; // Wakeup Status - RO
pub(crate) const MANC: u32 = 0x05820; // Management Control - RW
pub(crate) const FFLT: u32 = 0x05F00; // Flexible Filter Length Table - RW Array
pub(crate) const HOST_IF: u32 = 0x08800; // Host Interface
pub(crate) const SW_FW_SYNC: u32 = 0x05B5C; /* SW-FW Synchronization - RW */
pub(crate) const GCR: u32 = 0x05B00; /* PCI-Ex Control */
pub(crate) const GCR2: u32 = 0x05B64; /* PCI-Ex Control #2 */
pub(crate) const FACTPS: u32 = 0x05B30; /* Function Active and Power State to MNG */
pub(crate) const SWSM: u32 = 0x05B50; /* SW Semaphore */
pub(crate) const FWSM: u32 = 0x05B54; /* FW Semaphore */
pub(crate) const EXFWSM: u32 = 0x05B58; /* Extended FW Semaphore */

pub(crate) const EIAC_82574: u32 = 0x000DC;
pub(crate) const EIAC_MASK_82574: u32 = 0x01F00000;

pub(crate) const STRAP: u32 = 0x0000C;
pub(crate) const STRAP_SMBUS_ADDRESS_MASK: u32 = 0x00FE0000;
pub(crate) const STRAP_SMBUS_ADDRESS_SHIFT: u32 = 17;
pub(crate) const STRAP_SMT_FREQ_MASK: u32 = 0x00003000;
pub(crate) const STRAP_SMT_FREQ_SHIFT: u32 = 12;

pub(crate) const FEXTNVM: u32 = 0x00028; // Future Extended NVM - RW
pub(crate) const FEXTNVM3: u32 = 0x0003C; // Future Extended NVM 3 - RW
pub(crate) const FEXTNVM4: u32 = 0x00024; // Future Extended NVM 4 - RW
pub(crate) const FEXTNVM5: u32 = 0x00014; // Future Extended NVM 5 - RW
pub(crate) const FEXTNVM6: u32 = 0x00010; // Future Extended NVM 6 - RW
pub(crate) const FEXTNVM7: u32 = 0x000E4; // Future Extended NVM 7 - RW
pub(crate) const FEXTNVM8: u32 = 0x5BB0; // Future Extended NVM 8 - RW
pub(crate) const FEXTNVM9: u32 = 0x5BB4; // Future Extended NVM 9 - RW
pub(crate) const FEXTNVM11: u32 = 0x5BBC; // Future Extended NVM 11 - RW
pub(crate) const FEXTNVM12: u32 = 0x5BC0; // Future Extended NVM 12 - RW

pub(crate) const PBA_ECC: u32 = 0x01100;

pub(crate) const TIDV: u32 = 0x03820; // Tx Interrupt Delay Value - RW
pub(crate) const TADV: u32 = 0x0382C; // Tx Interrupt Absolute Delay Value - RW
pub(crate) const IOSFPC: u32 = 0x00F28; // TX corrupted data register

pub(crate) const RDTR: u32 = 0x02820; // Rx Delay Timer - Read-Write
pub(crate) const RADV: u32 = 0x0282C; // Rx Interrupt Absolute Delay Timer - Read-Write

pub(crate) const PBECCSTS: u32 = 0x0100C; /* Packet Buffer ECC Status - RW */

pub(crate) const PHY_CTRL: u32 = 0x00F10; /* PHY Control Register in CSR */

pub(crate) fn TARC(n: u32) -> u32 {
    0x03840 + (n * 0x100)
}

pub(crate) fn TDBAL(n: u32) -> u32 {
    if n < 4 {
        0x03800 + (0x100 * n)
    } else {
        0x0E000 + (0x40 * n)
    }
}
pub(crate) fn TDBAH(n: u32) -> u32 {
    if n < 4 {
        0x03804 + (0x100 * n)
    } else {
        0x0E004 + (0x40 * n)
    }
}

pub(crate) fn EITR_82574(n: u32) -> u32 {
    0x000E8 + (0x4 * n)
}

pub(crate) const CRCERRS: u32 = 0x04000; // CRC Error Count - Read/clear
pub(crate) const ALGNERRC: u32 = 0x04004; // Alignment Error Count - Read/clear
pub(crate) const SYMERRS: u32 = 0x04008; // Symbol Error Count - Read/clear
pub(crate) const RXERRC: u32 = 0x0400C; // Receive Error Count - Read/clear
pub(crate) const MPC: u32 = 0x04010; // Missed Packet Count - Read/clear
pub(crate) const SCC: u32 = 0x04014; // Single Collision Count - Read/clear
pub(crate) const ECOL: u32 = 0x04018; // Excessive Collision Count - Read/clear
pub(crate) const MCC: u32 = 0x0401C; // Multiple Collision Count - Read/clear
pub(crate) const LATECOL: u32 = 0x04020; // Late Collision Count - Read/clear
pub(crate) const COLC: u32 = 0x04028; // Collision Count - Read/clear
pub(crate) const DC: u32 = 0x04030; // Defer Count - Read/clear
pub(crate) const TNCRS: u32 = 0x04034; // Tx-No CRS - Read/clear
pub(crate) const SEC: u32 = 0x04038; // Sequence Error Count - Read/clear
pub(crate) const CEXTERR: u32 = 0x0403C; // Carrier Extension Error Count - Read/clear
pub(crate) const RLEC: u32 = 0x04040; // Receive Length Error Count - Read/clear
pub(crate) const XONRXC: u32 = 0x04048; // XON Rx Count - Read/clear
pub(crate) const XONTXC: u32 = 0x0404C; // XON Tx Count - Read/clear
pub(crate) const XOFFRXC: u32 = 0x04050; // XOFF Rx Count - Read/clear
pub(crate) const XOFFTXC: u32 = 0x04054; // XOFF Tx Count - Read/clear
pub(crate) const FCRUC: u32 = 0x04058; // Flow Control Rx Unsupported Count - Read/clear
pub(crate) const PRC64: u32 = 0x0405C; // Packets Rx (64 bytes) - Read/clear
pub(crate) const PRC127: u32 = 0x04060; // Packets Rx (65-127 bytes) - Read/clear
pub(crate) const PRC255: u32 = 0x04064; // Packets Rx (128-255 bytes) - Read/clear
pub(crate) const PRC511: u32 = 0x04068; // Packets Rx (256-511 bytes) - Read/clear
pub(crate) const PRC1023: u32 = 0x0406C; // Packets Rx (512-1023 bytes) - Read/clear
pub(crate) const PRC1522: u32 = 0x04070; // Packets Rx (1024-1522 bytes) - Read/clear
pub(crate) const GPRC: u32 = 0x04074; // Good Packets Rx Count - Read/clear
pub(crate) const BPRC: u32 = 0x04078; // Broadcast Packets Rx Count - Read/clear
pub(crate) const MPRC: u32 = 0x0407C; // Multicast Packets Rx Count - Read/clear
pub(crate) const GPTC: u32 = 0x04080; // Good Packets Tx Count - Read/clear
pub(crate) const GORCL: u32 = 0x04088; // Good Octets Rx Count Low - Read/clear
pub(crate) const GORCH: u32 = 0x0408C; // Good Octets Rx Count High - Read/clear
pub(crate) const GOTCL: u32 = 0x04090; // Good Octets Tx Count Low - Read/clear
pub(crate) const GOTCH: u32 = 0x04094; // Good Octets Tx Count High - Read/clear
pub(crate) const RNBC: u32 = 0x040A0; // Rx No Buffers Count - Read/clear
pub(crate) const RUC: u32 = 0x040A4; // Rx Undersize Count - Read/clear
pub(crate) const RFC: u32 = 0x040A8; // Rx Fragment Count - Read/clear
pub(crate) const ROC: u32 = 0x040AC; // Rx Oversize Count - Read/clear
pub(crate) const RJC: u32 = 0x040B0; // Rx Jabber Count - Read/clear

pub(crate) const MPTC: u32 = 0x040F0; // Multicast Packets Tx Count - Read/clear
pub(crate) const BPTC: u32 = 0x040F4; // Broadcast Packets Tx Count - Read/clear
pub(crate) const TSCTC: u32 = 0x040F8; // TCP Segmentation Context Tx Count - Read/clear
pub(crate) const TSCTFC: u32 = 0x040FC; // TCP Segmentation Context Tx Fail Count - Read/clear

pub(crate) const TPR: u32 = 0x040D0; // Total Packets Rx - Read/clear
pub(crate) const TPT: u32 = 0x040D4; // Total Packets Tx - Read/clear

pub(crate) const SWSM2: u32 = 0x05B58; // Driver-only software semaphore

pub(crate) const FCRTL: u32 = 0x02160; // Flow Control Receive Threshold Low Register - Read/Write
pub(crate) const FCRTH: u32 = 0x02168; // Flow Control Receive Threshold High Register - Read/Write

pub(crate) const FFLT_DBG: u32 = 0x05F04; // Debug Register

pub(crate) const TORL: u32 = 0x040C0; // Total Octets Rx Low - Read/clear
pub(crate) const TORH: u32 = 0x040C4; // Total Octets Rx High - Read/clear
pub(crate) const TOTL: u32 = 0x040C8; // Total Octets Tx Low - Read/clear
pub(crate) const TOTH: u32 = 0x040CC; // Total Octets Tx High - Read/clear

pub(crate) const MGTPRC: u32 = 0x040B4; // Management Packets Rx Count - Read/clear
pub(crate) const MGTPDC: u32 = 0x040B8; // Management Packets Dropped Count - Read/clear
pub(crate) const MGTPTC: u32 = 0x040BC; // Management Packets Tx Count - Read/clear

pub(crate) const IAC: u32 = 0x04100; // Interrupt Assertion Count
pub(crate) const ICRXOC: u32 = 0x04124; // Interrupt Cause Receiver Overrun Count

pub(crate) const FCRTV_PCH: u32 = 0x05F40; // PCH Flow Control Refresh Timer Value

pub(crate) const PHY_PAGE_SHIFT: u32 = 5;
pub(crate) const MAX_PHY_REG_ADDRESS: u32 = 0x1F; // Assuming this is the maximum register address

pub(crate) const KMRNCTRLSTA: u32 = 0x00034; /* MAC-PHY interface - RW */

pub(crate) const KABGTXD: u32 = 0x03004; // AFE Band Gap Transmit Ref Data

pub(crate) const EXTCNF_CTRL: u32 = 0x00F00; // Extended Configuration Control
pub(crate) const EXTCNF_SIZE: u32 = 0x00F08; // Extended Configuration Size
pub(crate) const RCTL: u32 = 0x00100; // RX Control - RW
pub(crate) const RDTR1: u32 = 0x02820; // RX Delay Timer (1) - RW
pub(crate) const RDBAL1: u32 = 0x02900; // RX Descriptor Base Address Low (1) - RW
pub(crate) const RDBAH1: u32 = 0x02904; // RX Descriptor Base Address High (1) - RW
pub(crate) const RDLEN1: u32 = 0x02908; // RX Descriptor Length (1) - RW
pub(crate) const RDH1: u32 = 0x02910; // RX Descriptor Head (1) - RW
pub(crate) const RDT1: u32 = 0x02918; // RX Descriptor Tail (1) - RW
pub(crate) const FCTTV: u32 = 0x00170; // Flow Control Transmit Timer Value - RW
pub(crate) const TXCW: u32 = 0x00178; // TX Configuration Word - RW
pub(crate) const RXCW: u32 = 0x00180; // RX Configuration Word - RO
pub(crate) const TCTL: u32 = 0x00400; // TX Control - RW
pub(crate) const TCTL_EXT: u32 = 0x00404; // Extended TX Control - RW
pub(crate) const TIPG: u32 = 0x00410; // TX Inter-packet gap - RW
pub(crate) const TBT: u32 = 0x00448; // TX Burst Timer - RW
pub(crate) const AIT: u32 = 0x00458; // Adaptive Interframe Spacing Throttle - RW

pub(crate) const MANC2H: u32 = 0x05860; // Management Control To Host - RW

pub(crate) fn RAL(index: u32) -> u32 {
    RA + index * 8
}

pub(crate) fn RAH(index: u32) -> u32 {
    RA + index * 8 + 4
}

pub(crate) const fn PCH_RAICC(n: u32) -> u32 {
    0x05F50 + (n * 4)
}

use core::time::Duration;

use crate::kfn;
use crate::BoardInfo;
use crate::MacType;
use crate::{defines::*, err::*};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Reg {
    pub mmio: usize,
    pub flash: Option<usize>,
    pub pcim2pci_arbiter_wa: bool,
}

impl Reg {
    pub fn new(board: &BoardInfo) -> Result<Self> {
        let mmio = kfn().bar_n_remap(0);
        let mut flash = None;

        if board.flags.has_flash && board.mac < MacType::e1000_pch_spt {
            let addr = kfn().bar_n_remap(1);
            if addr > 0 {
                info!("flash: {:#X}", addr);
                flash = Some(addr);
            } else {
                error!("remap bar1 failed");
                return Err(Error::Unknown("remap bar1 failed"));
            }
        };

        Ok(Self {
            mmio,
            flash,
            pcim2pci_arbiter_wa: false,
        })
    }

    fn read<T>(&self, offset: u32) -> T {
        unsafe {
            let ptr = (self.mmio + offset as usize) as *const T;
            ptr.read_volatile()
        }
    }
    pub fn er32(&self, offset: u32) -> u32 {
        self.read(offset)
    }
    fn ew32_prepare(&self) {
        let mut i = ICH_FWSM_PCIM2PCI_COUNT;

        while ((self.er32(FWSM) & ICH_FWSM_PCIM2PCI) != 0) && i > 0 {
            kfn().sleep(Duration::from_micros(50));
            i -= 1;
        }
    }
    pub fn ew32(&self, offset: u32, value: u32) {
        if self.pcim2pci_arbiter_wa {
            self.ew32_prepare();
        }
        unsafe {
            let ptr = (self.mmio + offset as usize) as *mut u32;
            ptr.write_volatile(value);
        }
    }

    pub fn e1e_flush(&self) {
        self.er32(STATUS);
    }

    pub fn write_reg_array(&self, reg: u32, offset: u32, value: u32) {
        let address = reg + (offset << 2);
        self.ew32(address, value);
    }
}
