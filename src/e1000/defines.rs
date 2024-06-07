pub(crate) const ICR_TXDW: u32 = 0x00000001; // Transmit desc written back
pub(crate) const ICR_LSC: u32 = 0x00000004; // Link Status Change
pub(crate) const ICR_RXSEQ: u32 = 0x00000008; // Rx sequence error
pub(crate) const ICR_RXDMT0: u32 = 0x00000010; // Rx desc min. threshold (0)
pub(crate) const ICR_RXO: u32 = 0x00000040; // Receiver Overrun
pub(crate) const ICR_RXT0: u32 = 0x00000080; // Rx timer intr (ring 0)
pub(crate) const ICR_MDAC: u32 = 0x00000200; // MDIO Access Complete
pub(crate) const ICR_SRPD: u32 = 0x00010000; // Small Receive Packet Detected
pub(crate) const ICR_ACK: u32 = 0x00020000; // Receive ACK Frame Detected
pub(crate) const ICR_MNG: u32 = 0x00040000; // Manageability Event Detected
pub(crate) const ICR_ECCER: u32 = 0x00400000; // Uncorrectable ECC Error
pub(crate) const ICR_INT_ASSERTED: u32 = 0x80000000; // Interrupt Asserted
pub(crate) const ICR_RXQ0: u32 = 0x00100000; // Rx Queue 0 Interrupt
pub(crate) const ICR_RXQ1: u32 = 0x00200000; // Rx Queue 1 Interrupt
pub(crate) const ICR_TXQ0: u32 = 0x00400000; // Tx Queue 0 Interrupt
pub(crate) const ICR_TXQ1: u32 = 0x00800000; // Tx Queue 1 Interrupt
pub(crate) const ICR_OTHER: u32 = 0x01000000; // Other Interrupt

pub(crate) const IMS_TXDW: u32 = ICR_TXDW; // Transmit desc written back
pub(crate) const IMS_LSC: u32 = ICR_LSC; // Link Status Change
pub(crate) const IMS_RXSEQ: u32 = ICR_RXSEQ; // Rx sequence error
pub(crate) const IMS_RXDMT0: u32 = ICR_RXDMT0; // Rx desc min. threshold
pub(crate) const IMS_RXO: u32 = ICR_RXO; // Receiver Overrun
pub(crate) const IMS_RXT0: u32 = ICR_RXT0; // Rx timer intr
pub(crate) const IMS_MDAC: u32 = ICR_MDAC; // MDIO Access Complete
pub(crate) const IMS_SRPD: u32 = ICR_SRPD; // Small Receive Packet
pub(crate) const IMS_ACK: u32 = ICR_ACK; // Receive ACK Frame Detected
pub(crate) const IMS_MNG: u32 = ICR_MNG; // Manageability Event
pub(crate) const IMS_ECCER: u32 = ICR_ECCER; // Uncorrectable ECC Error
pub(crate) const IMS_RXQ0: u32 = ICR_RXQ0; // Rx Queue 0 Interrupt
pub(crate) const IMS_RXQ1: u32 = ICR_RXQ1; // Rx Queue 1 Interrupt
pub(crate) const IMS_TXQ0: u32 = ICR_TXQ0; // Tx Queue 0 Interrupt
pub(crate) const IMS_TXQ1: u32 = ICR_TXQ1; // Tx Queue 1 Interrupt
pub(crate) const IMS_OTHER: u32 = ICR_OTHER; // Other Interrupt

pub(crate) const IMS_ENABLE_MASK: u32 = IMS_RXT0 | IMS_TXDW | IMS_RXDMT0 | IMS_RXSEQ | IMS_LSC;

pub(crate) const IMS_OTHER_MASK: u32 = IMS_LSC | IMS_RXO | IMS_MDAC | IMS_SRPD | IMS_ACK | IMS_MNG;

/* Device Status */
pub(crate) const STATUS_FD: u32 = 0x00000001; // Full duplex.0=half,1=full
pub(crate) const STATUS_LU: u32 = 0x00000002; // Link up.0=no,1=link
pub(crate) const STATUS_FUNC_MASK: u32 = 0x0000000C; // PCI Function Mask
pub(crate) const STATUS_FUNC_SHIFT: u32 = 2;
pub(crate) const STATUS_FUNC_1: u32 = 0x00000004; // Function 1
pub(crate) const STATUS_TXOFF: u32 = 0x00000010; // transmission paused
pub(crate) const STATUS_SPEED_MASK: u32 = 0x000000C0;
pub(crate) const STATUS_SPEED_10: u32 = 0x00000000; // Speed 10Mb/s
pub(crate) const STATUS_SPEED_100: u32 = 0x00000040; // Speed 100Mb/s
pub(crate) const STATUS_SPEED_1000: u32 = 0x00000080; // Speed 1000Mb/s
pub(crate) const STATUS_LAN_INIT_DONE: u32 = 0x00000200; // Lan Init Completion by NVM
pub(crate) const STATUS_PHYRA: u32 = 0x00000400; // PHY Reset Asserted
pub(crate) const STATUS_GIO_MASTER_ENABLE: u32 = 0x00080000; // Master Req status

pub(crate) const RAR_ENTRIES: usize = 15;

/* SW Semaphore Register */
pub(crate) const SWSM_SMBI: u32 = 0x00000001; /* Driver Semaphore bit */
pub(crate) const SWSM_SWESMBI: u32 = 0x00000002; /* FW Semaphore bit */
pub(crate) const SWSM_DRV_LOAD: u32 = 0x00000008; /* Driver Loaded Bit */

pub(crate) const EECD_ADDR_BITS: u32 = 0x00000400;
pub(crate) const EECD_AUTO_RD: u32 = 0x00000200; // NVM Auto Read done
pub(crate) const EECD_SIZE_EX_MASK: u32 = 0x00007800; // NVM Size
pub(crate) const EECD_SIZE_EX_SHIFT: u32 = 11;
pub(crate) const EECD_FLUPD: u32 = 0x00080000; // Update FLASH
pub(crate) const EECD_AUPDEN: u32 = 0x00100000; // Enable Autonomous FLASH update
pub(crate) const EECD_SEC1VAL: u32 = 0x00400000; // Sector One Valid

pub(crate) const NVM_PBA_OFFSET_0: usize = 8;
pub(crate) const NVM_PBA_OFFSET_1: usize = 9;
pub(crate) const NVM_PBA_PTR_GUARD: u16 = 0xFAFA;
pub(crate) const NVM_WORD_SIZE_BASE_SHIFT: u16 = 6;

pub(crate) const AUTO_ALL_MODES: u8 = 0;
pub(crate) const EEPROM_APME: u32 = 0x0400;

/* length of string needed to store PBA number */
pub(crate) const PBANUM_LENGTH: usize = 11;
