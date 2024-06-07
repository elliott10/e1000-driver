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

pub(crate) const NVM_PBA_OFFSET_0: u16 = 8;
pub(crate) const NVM_PBA_OFFSET_1: u16 = 9;
pub(crate) const NVM_PBA_PTR_GUARD: u16 = 0xFAFA;
pub(crate) const NVM_WORD_SIZE_BASE_SHIFT: u16 = 6;

pub(crate) const NVM_RW_REG_DATA: u32 = 16; // Offset to data in NVM r/w regs
pub(crate) const NVM_RW_REG_DONE: u32 = 2; // Offset to READ/WRITE done bit
pub(crate) const NVM_RW_REG_START: u32 = 1; // Start operation
pub(crate) const NVM_RW_ADDR_SHIFT: u32 = 2; // Shift to the address bits
pub(crate) const NVM_POLL_WRITE: u8 = 1; // Flag for polling write complete
pub(crate) const NVM_POLL_READ: u8 = 0; // Flag for polling read complete
pub(crate) const FLASH_UPDATES: usize = 2000; // Number of iterations for polling

pub(crate) const NVM_COMPAT: u16 = 0x0003;
pub(crate) const NVM_ID_LED_SETTINGS: u16 = 0x0004;
pub(crate) const NVM_FUTURE_INIT_WORD1: u16 = 0x0019;
pub(crate) const NVM_COMPAT_VALID_CSUM: u16 = 0x0001;
pub(crate) const NVM_FUTURE_INIT_WORD1_VALID_CSUM: u16 = 0x0040;

pub(crate) const NVM_INIT_CONTROL2_REG: u16 = 0x000F;
pub(crate) const NVM_INIT_CONTROL3_PORT_B: u16 = 0x0014;
pub(crate) const NVM_INIT_3GIO_3: u16 = 0x001A;
pub(crate) const NVM_INIT_CONTROL3_PORT_A: u16 = 0x0024;
pub(crate) const NVM_CFG: u16 = 0x0012;
pub(crate) const NVM_ALT_MAC_ADDR_PTR: u16 = 0x0037;
pub(crate) const NVM_CHECKSUM_REG: u16 = 0x003F;

pub(crate) const NVM_CFG_DONE_PORT_0: u32 = 0x40000; // MNG config cycle done
pub(crate) const NVM_CFG_DONE_PORT_1: u32 = 0x80000; // ...for second port

// Mask bits for fields in Word 0x0f of the NVM
pub(crate) const NVM_WORD0F_PAUSE_MASK: u16 = 0x3000;
pub(crate) const NVM_WORD0F_PAUSE: u16 = 0x1000;
pub(crate) const NVM_WORD0F_ASM_DIR: u16 = 0x2000;

// Mask bits for fields in Word 0x1a of the NVM
pub(crate) const NVM_WORD1A_ASPM_MASK: u16 = 0x000C;

// Mask bits for fields in Word 0x03 of the EEPROM
pub(crate) const NVM_COMPAT_LOM: u16 = 0x0800;

pub(crate) const AUTO_ALL_MODES: u8 = 0;
pub(crate) const EEPROM_APME: u32 = 0x0400;

/* length of string needed to store PBA number */
pub(crate) const PBANUM_LENGTH: usize = 11;

pub(crate) const MANC_SMBUS_EN: u32 = 0x00000001; /* SMBus Enabled - RO */
pub(crate) const MANC_ASF_EN: u32 = 0x00000002; /* ASF Enabled - RO */
pub(crate) const MANC_ARP_EN: u32 = 0x00002000; /* Enable ARP Request Filtering */
pub(crate) const MANC_RCV_TCO_EN: u32 = 0x00020000; /* Receive TCO Packets Enabled */
pub(crate) const MANC_BLK_PHY_RST_ON_IDE: u32 = 0x00040000; /* Block phy resets */
pub(crate) const MANC_EN_MAC_ADDR_FILTER: u32 = 0x00100000; /* Enable MAC address filtering */
pub(crate) const MANC_EN_MNG2HOST: u32 = 0x00200000; /* Enable MNG packets to host memory */

pub(crate) const MANC2H_PORT_623: u32 = 0x00000020; /* Port 0x26f */
pub(crate) const MANC2H_PORT_664: u32 = 0x00000040; /* Port 0x298 */
pub(crate) const MDEF_PORT_623: u32 = 0x00000800; /* Port 0x26f */
pub(crate) const MDEF_PORT_664: u32 = 0x00000400; /* Port 0x298 */

pub(crate) const FACTPS_MNGCG: u32 = 0x20000000;

pub(crate) const FWSM_MODE_MASK: u32 = 0xE;
pub(crate) const FWSM_MODE_SHIFT: u8 = 1;

pub(crate) const VLAN_ETH_DATA_LEN: usize = 1500; /* Max. octets in payload */
pub(crate) const VLAN_ETH_FRAME_LEN: usize = 1518; /* Max. octets in frame sans FCS */
pub(crate) const VLAN_MAX_DEPTH: usize = 8; /* Max. number of nested VLAN tags parsed */
pub(crate) const ETH_FCS_LEN: usize = 4; /* Octets in the FCS */

pub(crate) const FC_PAUSE_TIME: u16 = 0x0680; //  858微秒

pub(crate) const FCRTH_RTH: u32 = 0x0000FFF8; // Mask Bits[15:3] for RTH
pub(crate) const FCRTL_RTL: u32 = 0x0000FFF8; // Mask Bits[15:3] for RTL
pub(crate) const FCRTL_XONE: u32 = 0x80000000; // Enable XON frame transmission



