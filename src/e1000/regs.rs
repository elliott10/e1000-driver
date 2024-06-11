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
pub(crate) const STRAP_SMBUS_ADDRESS_MASK: usize = 0x00FE0000;
pub(crate) const STRAP_SMBUS_ADDRESS_SHIFT: usize = 17;
pub(crate) const STRAP_SMT_FREQ_MASK: usize = 0x00003000;
pub(crate) const STRAP_SMT_FREQ_SHIFT: usize = 12;

pub(crate) const FEXTNVM: u16 = 0x00028;     // Future Extended NVM - RW
pub(crate) const FEXTNVM3: u16 = 0x0003C;    // Future Extended NVM 3 - RW
pub(crate) const FEXTNVM4: u16 = 0x00024;    // Future Extended NVM 4 - RW
pub(crate) const FEXTNVM5: u16 = 0x00014;    // Future Extended NVM 5 - RW
pub(crate) const FEXTNVM6: u16 = 0x00010;    // Future Extended NVM 6 - RW
pub(crate) const FEXTNVM7: u16 = 0x000E4;    // Future Extended NVM 7 - RW
pub(crate) const FEXTNVM8: u32 = 0x5BB0;     // Future Extended NVM 8 - RW
pub(crate) const FEXTNVM9: u32 = 0x5BB4;     // Future Extended NVM 9 - RW
pub(crate) const FEXTNVM11: usize = 0x5BBC;    // Future Extended NVM 11 - RW
pub(crate) const FEXTNVM12: usize = 0x5BC0;    // Future Extended NVM 12 - RW

pub(crate) const PBA_ECC: usize = 0x01100;

pub(crate) fn TARC(n: usize) -> usize {
    0x03840 + (n * 0x100)
}