#![allow(non_snake_case)]

pub(crate) const CTRL: usize = 0x00000; // Device Control - RW
pub(crate) const STATUS: usize = 0x00008; // Device Status - RO
pub(crate) const EECD: usize = 0x00010; // EEPROM/Flash Control - RW
pub(crate) const EERD: usize = 0x00014; // EEPROM Read - RW
pub(crate) const CTRL_EXT: usize = 0x00018; // Extended Device Control - RW
pub(crate) const FLA: usize = 0x0001C; // Flash Access - RW
pub(crate) const MDIC: usize = 0x00020; // MDI Control - RW
pub(crate) const SCTL: usize = 0x00024; // SerDes Control - RW
pub(crate) const FCAL: usize = 0x00028; // Flow Control Address Low - RW
pub(crate) const FCAH: usize = 0x0002C; // Flow Control Address High - RW
                                        // 注意：FEXT 和 E1000_FCAH 有相同的值，需要确认是否是故意的
pub(crate) const FEXT: usize = 0x0002C; // Future Extended - RW

pub(crate) const FCT: usize = 0x00030; // Flow Control Type - RW
pub(crate) const VET: usize = 0x00038; // VLAN Ether Type - RW
pub(crate) const ICR: usize = 0x000C0; // Interrupt Cause Read - R/clr
pub(crate) const ITR: usize = 0x000C4; // Interrupt Throttling Rate - RW
pub(crate) const ICS: usize = 0x000C8; // Interrupt Cause Set - WO
pub(crate) const IMS: usize = 0x000D0; // Interrupt Mask Set - RW
pub(crate) const IMC: usize = 0x000D8; // Interrupt Mask Clear - WO
pub(crate) const IAM: usize = 0x000E0; // Interrupt Acknowledge Auto Mask
pub(crate) const IVAR: usize = 0x000E4; // Interrupt Vector Allocation Register - RW
pub(crate) const PBA: usize = 0x01000; /* Packet Buffer Allocation - RW */
pub(crate) const PBS: usize = 0x01008; /* Packet Buffer Size */
pub(crate) const EEWR: usize = 0x0102C; /* EEPROM Write Register - RW */

pub(crate) const PCS_LCTL: usize = 0x04208; // PCS Link Control - RW
pub(crate) const PCS_LSTAT: usize = 0x0420C; // PCS Link Status - RO
pub(crate) const PCS_ANADV: usize = 0x04218; // AN advertisement - RW
pub(crate) const PCS_LPAB: usize = 0x0421C; // Link Partner Ability - RW
pub(crate) const RXCSUM: usize = 0x05000; // Rx Checksum Control - RW
pub(crate) const RFCTL: usize = 0x05008; // Receive Filter Control
pub(crate) const MTA: usize = 0x05200; // Multicast Table Array - RW Array
pub(crate) const RA: usize = 0x05400; // Receive Address - RW Array
pub(crate) const VFTA: usize = 0x05600; // VLAN Filter Table Array - RW Array
pub(crate) const WUC: usize = 0x05800; // Wakeup Control - RW
pub(crate) const WUFC: usize = 0x05808; // Wakeup Filter Control - RW
pub(crate) const WUS: usize = 0x05810; // Wakeup Status - RO
pub(crate) const MANC: usize = 0x05820; // Management Control - RW
pub(crate) const FFLT: usize = 0x05F00; // Flexible Filter Length Table - RW Array
pub(crate) const HOST_IF: usize = 0x08800; // Host Interface
pub(crate) const SW_FW_SYNC: usize = 0x05B5C; /* SW-FW Synchronization - RW */
pub(crate) const GCR: usize = 0x05B00; /* PCI-Ex Control */
pub(crate) const GCR2: usize = 0x05B64; /* PCI-Ex Control #2 */
pub(crate) const FACTPS: usize = 0x05B30; /* Function Active and Power State to MNG */
pub(crate) const SWSM: usize = 0x05B50; /* SW Semaphore */
pub(crate) const FWSM: usize = 0x05B54; /* FW Semaphore */
pub(crate) const EXFWSM: usize = 0x05B58; /* Extended FW Semaphore */

pub(crate) const EIAC_82574: usize = 0x000DC;
pub(crate) const EIAC_MASK_82574: u32 = 0x01F00000;

pub(crate) const STRAP: usize = 0x0000C;
pub(crate) const STRAP_SMBUS_ADDRESS_MASK: u32 = 0x00FE0000;
pub(crate) const STRAP_SMBUS_ADDRESS_SHIFT: u32 = 17;
pub(crate) const STRAP_SMT_FREQ_MASK: u32 = 0x00003000;
pub(crate) const STRAP_SMT_FREQ_SHIFT: u32 = 12;

pub(crate) const FEXTNVM: u16 = 0x00028; // Future Extended NVM - RW
pub(crate) const FEXTNVM3: u16 = 0x0003C; // Future Extended NVM 3 - RW
pub(crate) const FEXTNVM4: u16 = 0x00024; // Future Extended NVM 4 - RW
pub(crate) const FEXTNVM5: u16 = 0x00014; // Future Extended NVM 5 - RW
pub(crate) const FEXTNVM6: u16 = 0x00010; // Future Extended NVM 6 - RW
pub(crate) const FEXTNVM7: u16 = 0x000E4; // Future Extended NVM 7 - RW
pub(crate) const FEXTNVM8: usize = 0x5BB0; // Future Extended NVM 8 - RW
pub(crate) const FEXTNVM9: usize = 0x5BB4; // Future Extended NVM 9 - RW
pub(crate) const FEXTNVM11: usize = 0x5BBC; // Future Extended NVM 11 - RW
pub(crate) const FEXTNVM12: usize = 0x5BC0; // Future Extended NVM 12 - RW

pub(crate) const PBA_ECC: usize = 0x01100;

pub(crate) const TIDV: usize = 0x03820; // Tx Interrupt Delay Value - RW
pub(crate) const TADV: usize = 0x0382C; // Tx Interrupt Absolute Delay Value - RW
pub(crate) const IOSFPC: usize = 0x00F28; // TX corrupted data register

pub(crate) const RDTR: usize = 0x02820; // Rx Delay Timer - Read-Write
pub(crate) const RADV: usize = 0x0282C; // Rx Interrupt Absolute Delay Timer - Read-Write

pub(crate) fn TARC(n: usize) -> usize {
    0x03840 + (n * 0x100)
}

pub(crate) fn TDBAL(n: usize) -> usize {
    if n < 4 {
        0x03800 + (0x100 * n)
    } else {
        0x0E000 + (0x40 * n)
    }
}
pub(crate) fn TDBAH(n: usize) -> usize {
    if n < 4 {
        0x03804 + (0x100 * n)
    } else {
        0x0E004 + (0x40 * n)
    }
}

pub(crate) fn EITR_82574(n: usize) -> usize {
    0x000E8 + (0x4 * n)
}

pub(crate) const CRCERRS: usize = 0x04000; // CRC Error Count - Read/clear
pub(crate) const ALGNERRC: usize = 0x04004; // Alignment Error Count - Read/clear
pub(crate) const SYMERRS: usize = 0x04008; // Symbol Error Count - Read/clear
pub(crate) const RXERRC: usize = 0x0400C; // Receive Error Count - Read/clear
pub(crate) const MPC: usize = 0x04010; // Missed Packet Count - Read/clear
pub(crate) const SCC: usize = 0x04014; // Single Collision Count - Read/clear
pub(crate) const ECOL: usize = 0x04018; // Excessive Collision Count - Read/clear
pub(crate) const MCC: usize = 0x0401C; // Multiple Collision Count - Read/clear
pub(crate) const LATECOL: usize = 0x04020; // Late Collision Count - Read/clear
pub(crate) const COLC: usize = 0x04028; // Collision Count - Read/clear
pub(crate) const DC: usize = 0x04030; // Defer Count - Read/clear
pub(crate) const TNCRS: usize = 0x04034; // Tx-No CRS - Read/clear
pub(crate) const SEC: usize = 0x04038; // Sequence Error Count - Read/clear
pub(crate) const CEXTERR: usize = 0x0403C; // Carrier Extension Error Count - Read/clear
pub(crate) const RLEC: usize = 0x04040; // Receive Length Error Count - Read/clear
pub(crate) const XONRXC: usize = 0x04048; // XON Rx Count - Read/clear
pub(crate) const XONTXC: usize = 0x0404C; // XON Tx Count - Read/clear
pub(crate) const XOFFRXC: usize = 0x04050; // XOFF Rx Count - Read/clear
pub(crate) const XOFFTXC: usize = 0x04054; // XOFF Tx Count - Read/clear
pub(crate) const FCRUC: usize = 0x04058; // Flow Control Rx Unsupported Count - Read/clear
pub(crate) const PRC64: usize = 0x0405C; // Packets Rx (64 bytes) - Read/clear
pub(crate) const PRC127: usize = 0x04060; // Packets Rx (65-127 bytes) - Read/clear
pub(crate) const PRC255: usize = 0x04064; // Packets Rx (128-255 bytes) - Read/clear
pub(crate) const PRC511: usize = 0x04068; // Packets Rx (256-511 bytes) - Read/clear
pub(crate) const PRC1023: usize = 0x0406C; // Packets Rx (512-1023 bytes) - Read/clear
pub(crate) const PRC1522: usize = 0x04070; // Packets Rx (1024-1522 bytes) - Read/clear
pub(crate) const GPRC: usize = 0x04074; // Good Packets Rx Count - Read/clear
pub(crate) const BPRC: usize = 0x04078; // Broadcast Packets Rx Count - Read/clear
pub(crate) const MPRC: usize = 0x0407C; // Multicast Packets Rx Count - Read/clear
pub(crate) const GPTC: usize = 0x04080; // Good Packets Tx Count - Read/clear
pub(crate) const GORCL: usize = 0x04088; // Good Octets Rx Count Low - Read/clear
pub(crate) const GORCH: usize = 0x0408C; // Good Octets Rx Count High - Read/clear
pub(crate) const GOTCL: usize = 0x04090; // Good Octets Tx Count Low - Read/clear
pub(crate) const GOTCH: usize = 0x04094; // Good Octets Tx Count High - Read/clear
pub(crate) const RNBC: usize = 0x040A0; // Rx No Buffers Count - Read/clear
pub(crate) const RUC: usize = 0x040A4; // Rx Undersize Count - Read/clear
pub(crate) const RFC: usize = 0x040A8; // Rx Fragment Count - Read/clear
pub(crate) const ROC: usize = 0x040AC; // Rx Oversize Count - Read/clear
pub(crate) const RJC: usize = 0x040B0; // Rx Jabber Count - Read/clear


pub(crate) const MPTC: usize = 0x040F0;         // Multicast Packets Tx Count - Read/clear
pub(crate) const BPTC: usize = 0x040F4;         // Broadcast Packets Tx Count - Read/clear
pub(crate) const TSCTC: usize   = 0x040F8;        // TCP Segmentation Context Tx Count - Read/clear
pub(crate) const TSCTFC: usize = 0x040FC;       // TCP Segmentation Context Tx Fail Count - Read/clear


pub(crate) const TPR: usize = 0x040D0;         // Total Packets Rx - Read/clear
pub(crate) const TPT: usize = 0x040D4;         // Total Packets Tx - Read/clear
