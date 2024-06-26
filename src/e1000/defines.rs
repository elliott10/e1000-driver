#![allow(non_snake_case, non_upper_case_globals)]

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

pub(crate) const IMS_ENABLE_MASK: u32 =
    IMS_RXT0 | IMS_TXDW | IMS_RXDMT0 | IMS_RXSEQ | IMS_LSC | IMS_RXQ0;

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

pub(crate) const FEXTNVM7_DISABLE_PB_READ: u32 = 0x00040000; // Bit for disabling packet buffer read
pub(crate) const FEXTNVM7_SIDE_CLK_UNGATE: u32 = 0x00000004;
pub(crate) const FEXTNVM7_DISABLE_SMB_PERST: u32 = 0x00000020;
pub(crate) const FEXTNVM9_IOSFSB_CLKGATE_DIS: u32 = 0x00000800;
pub(crate) const FEXTNVM9_IOSFSB_CLKREQ_DIS: u32 = 0x00001000;
pub(crate) const FEXTNVM11_DISABLE_PB_READ: u32 = 0x00000200; // Disable packet buffer read
pub(crate) const FEXTNVM11_DISABLE_MULR_FIX: u32 = 0x00002000;

pub(crate) const RXDCTL_THRESH_UNIT_DESC: u32 = 0x01000000;

pub(crate) const TXDCTL_PTHRESH: u32 = 0x0000003F; /* TXDCTL Prefetch Threshold mask */
pub(crate) const TXDCTL_HTHRESH: u32 = 0x00003F00; /* TXDCTL Host Threshold mask */
pub(crate) const TXDCTL_WTHRESH: u32 = 0x003F0000; /* TXDCTL Writeback Threshold mask */
pub(crate) const TXDCTL_GRAN: u32 = 0x01000000; /* TXDCTL Granularity flag */
pub(crate) const TXDCTL_FULL_TX_DESC_WB: u32 = 0x01010000; /* GRAN=1, WTHRESH=1 combined value */
pub(crate) const TXDCTL_MAX_TX_DESC_PREFETCH: u32 = 0x0100001F; /* GRAN=1, PTHRESH=31 combined value */
/* Enable the counting of desc. still to be processed. */
pub(crate) const TXDCTL_COUNT_DESC: u32 = 0x00400000;
/* Adjustments made for more clarity and Rust idiomatic style */

pub(crate) const TXDCTL_DMA_BURST_ENABLE: u32 = TXDCTL_GRAN | TXDCTL_COUNT_DESC | // Descriptor granularity enabled
    (1 << 16) |      // WTHRESH is one more than the desired to avoid stalls, set to 1
    (1 << 8)  |      // HTHRESH set to 1
    0x1f; // PTHRESH set to maximum for efficiency

pub(crate) const RXDCTL_DMA_BURST_ENABLE: u32 = 0x01000000 | // Descriptor granularity enabled
    (4 << 16) |  // Optimal Writeback Threshold for bus efficiency, set to 4
    (4 << 8)   |  // Prefetch Threshold set to 4 for performance
    0x20; // HTHRESH set to a value that suits the hardware requirements

/* Translate the transmit control definitions into Rust constants */

pub(crate) const TCTL_EN: u32 = 0x00000002; // Enable transmission
pub(crate) const TCTL_PSP: u32 = 0x00000008; // Pad short packets to minimum frame size
pub(crate) const TCTL_CT: u32 = 0x00000ff0; // Collision threshold mask
pub(crate) const TCTL_COLD: u32 = 0x003ff000; // Collision distance mask
pub(crate) const TCTL_RTLC: u32 = 0x01000000; // Retransmit on late collision
pub(crate) const TCTL_MULR: u32 = 0x10000000; // Multiple requests support

/* Declare constants for collision-related configuration in Rust */

pub(crate) const COLLISION_THRESHOLD: u32 = 15;
pub(crate) const CT_SHIFT: u32 = 4;
pub(crate) const COLLISION_DISTANCE: u32 = 63;
pub(crate) const COLD_SHIFT: u32 = 12;

pub(crate) const TARC0_CB_MULTIQ_3_REQ: u32 = 0x30000000;
pub(crate) const TARC0_CB_MULTIQ_2_REQ: u32 = 0x20000000;

pub(crate) fn DMA_BIT_MASK(n: u32) -> u64 {
    match n {
        64 => u64::MAX,
        _ => (1u64 << n as u64) - 1,
    }
}

pub(crate) fn TDLEN(n: usize) -> usize {
    if n < 4 {
        0x03808 + (n * 0x100)
    } else {
        0x0E008 + (n * 0x40)
    }
}

pub(crate) fn TDH(n: usize) -> usize {
    if n < 4 {
        0x03810 + (n * 0x100)
    } else {
        0x0E010 + (n * 0x40)
    }
}

pub(crate) fn TDT(n: usize) -> usize {
    if n < 4 {
        0x03818 + (n * 0x100)
    } else {
        0x0E018 + (n * 0x40)
    }
}

pub(crate) fn TXDCTL(n: usize) -> usize {
    if n < 4 {
        0x03828 + (n * 0x100)
    } else {
        0x0E028 + (n * 0x40)
    }
}

/* Receive Descriptor Base Address Low, High, Length, Head, Tail, and Control */
pub fn RDBAL(n: usize) -> usize {
    if n < 4 {
        0x02800 + (n * 0x100)
    } else {
        0x0C000 + (n * 0x40)
    }
}

pub fn RDBAH(n: usize) -> usize {
    if n < 4 {
        0x02804 + (n * 0x100)
    } else {
        0x0C004 + (n * 0x40)
    }
}

pub fn RDLEN(n: usize) -> usize {
    if n < 4 {
        0x02808 + (n * 0x100)
    } else {
        0x0C008 + (n * 0x40)
    }
}

pub fn RDH(n: usize) -> usize {
    if n < 4 {
        0x02810 + (n * 0x100)
    } else {
        0x0C010 + (n * 0x40)
    }
}

pub fn RDT(n: usize) -> usize {
    if n < 4 {
        0x02818 + (n * 0x100)
    } else {
        0x0C018 + (n * 0x40)
    }
}

pub fn RXDCTL(n: usize) -> usize {
    if n < 4 {
        0x02828 + (n * 0x100)
    } else {
        0x0C028 + (n * 0x40)
    }
}

pub(crate) const fn BIT(n: usize) -> u32 {
    (1 << n) as u32
}

pub(crate) const PCICFG_DESC_RING_STATUS: i32 = 0xE4;
pub(crate) const FLUSH_DESC_REQUIRED: u16 = 0x100;

pub(crate) const TXD_DTYP_D: u32 = 0x00100000; /* Data Descriptor */
pub(crate) const TXD_POPTS_IXSM: u32 = 0x01; /* Insert IP checksum */
pub(crate) const TXD_POPTS_TXSM: u32 = 0x02; /* Insert TCP/UDP checksum */
pub(crate) const TXD_CMD_EOP: u32 = 0x01000000; /* End of Packet */
pub(crate) const TXD_CMD_IFCS: u32 = 0x02000000; /* Insert FCS (Ethernet CRC) */
pub(crate) const TXD_CMD_IC: u32 = 0x04000000; /* Insert Checksum */
pub(crate) const TXD_CMD_RS: u32 = 0x08000000; /* Report Status */
pub(crate) const TXD_CMD_RPS: u32 = 0x10000000; /* Report Packet Sent */
pub(crate) const TXD_CMD_DEXT: u32 = 0x20000000; /* Descriptor extension */
pub(crate) const TXD_CMD_VLE: u32 = 0x40000000; /* Add VLAN tag */
pub(crate) const TXD_CMD_IDE: u32 = 0x80000000; /* Enable Tidv register */
pub(crate) const TXD_STAT_DD: u32 = 0x00000001; /* Descriptor Done */
pub(crate) const TXD_STAT_EC: u32 = 0x00000002; /* Excess Collisions */
pub(crate) const TXD_STAT_LC: u32 = 0x00000004; /* Late Collisions */
pub(crate) const TXD_STAT_TU: u32 = 0x00000008; /* Transmit underrun */

pub(crate) const RXD_STAT_DD: u32 = 0x01; // Descriptor Done
pub(crate) const RXD_STAT_EOP: u32 = 0x02; // End of Packet
pub(crate) const RXD_STAT_IXSM: u32 = 0x04; // Ignore checksum
pub(crate) const RXD_STAT_VP: u32 = 0x08; // IEEE VLAN Packet
pub(crate) const RXD_STAT_UDPCS: u32 = 0x10; // UDP checksum calculated
pub(crate) const RXD_STAT_TCPCS: u32 = 0x20; // TCP checksum calculated
pub(crate) const RXD_ERR_CE: u32 = 0x01; // CRC Error
pub(crate) const RXD_ERR_SE: u32 = 0x02; // Symbol Error
pub(crate) const RXD_ERR_SEQ: u32 = 0x04; // Sequence Error
pub(crate) const RXD_ERR_CXE: u32 = 0x10; // Carrier Extension Error
pub(crate) const RXD_ERR_TCPE: u32 = 0x20; // TCP/UDP Checksum Error
pub(crate) const RXD_ERR_IPE: u32 = 0x40; // IP Checksum Error
pub(crate) const RXD_ERR_RXE: u32 = 0x80; // Rx Data Error
pub(crate) const RXD_SPC_VLAN_MASK: u32 = 0x0FFF; // VLAN ID is in lower 12 bits

// 注意重复的定义，根据实际用途可能需要调整或去除重复项
// TXD_CMD_TCP, TXD_CMD_IP, TXD_CMD_TSE 与之前的 E1000_TXD_CMD_EOP, E1000_TXD_CMD_IFCS, E1000_TXD_CMD_IC 重复
// TXD_STAT_TC 与 TXD_STAT_LC 重复定义
pub(crate) const TXD_EXTCMD_TSTAMP: u32 = 0x00000010; /* IEEE1588 Timestamp packet */

pub(crate) const TCTL_RST: u32 = 0x00000001; // Software reset
pub(crate) const TCTL_BCE: u32 = 0x00000004; // Busy check enable
pub(crate) const TCTL_CT_SHIFT: u32 = 4; // Collision threshold shift
pub(crate) const TCTL_COLD_SHIFT: u32 = 12; // Collision distance shift
pub(crate) const TCTL_SWXOFF: u32 = 0x00400000; // SW Xoff transmission
pub(crate) const TCTL_PBE: u32 = 0x00800000; // Packet Burst Enable
pub(crate) const TCTL_NRTU: u32 = 0x02000000; // No re-transmit on underrun

pub(crate) const RCTL: usize = 0x00100; // RX Control - RW
pub(crate) const RDTR1: usize = 0x02820; // RX Delay Timer (1) - RW
pub(crate) const RDBAL1: usize = 0x02900; // RX Descriptor Base Address Low (1) - RW
pub(crate) const RDBAH1: usize = 0x02904; // RX Descriptor Base Address High (1) - RW
pub(crate) const RDLEN1: usize = 0x02908; // RX Descriptor Length (1) - RW
pub(crate) const RDH1: usize = 0x02910; // RX Descriptor Head (1) - RW
pub(crate) const RDT1: usize = 0x02918; // RX Descriptor Tail (1) - RW
pub(crate) const FCTTV: usize = 0x00170; // Flow Control Transmit Timer Value - RW
pub(crate) const TXCW: usize = 0x00178; // TX Configuration Word - RW
pub(crate) const RXCW: usize = 0x00180; // RX Configuration Word - RO
pub(crate) const TCTL: usize = 0x00400; // TX Control - RW
pub(crate) const TCTL_EXT: usize = 0x00404; // Extended TX Control - RW
pub(crate) const TIPG: usize = 0x00410; // TX Inter-packet gap - RW
pub(crate) const TBT: usize = 0x00448; // TX Burst Timer - RW
pub(crate) const AIT: usize = 0x00458; // Adaptive Interframe Spacing Throttle - RW

/* Receive Control Flags */
pub(crate) const RCTL_EN: u32 = 0x00000002; // enable
pub(crate) const RCTL_SBP: u32 = 0x00000004; // store bad packet
pub(crate) const RCTL_UPE: u32 = 0x00000008; // unicast promiscuous enable
pub(crate) const RCTL_MPE: u32 = 0x00000010; // multicast promiscuous enable
pub(crate) const RCTL_LPE: u32 = 0x00000020; // long packet enable
pub(crate) const RCTL_LBM_NO: u32 = 0x00000000; // no loopback mode
pub(crate) const RCTL_LBM_MAC: u32 = 0x00000040; // MAC loopback mode
pub(crate) const RCTL_LBM_TCVR: u32 = 0x000000C0; // tcvr loopback mode
pub(crate) const RCTL_DTYP_PS: u32 = 0x00000400; // Packet Split descriptor
pub(crate) const RCTL_RDMTS_HALF: u32 = 0x00000000; // Rx desc min threshold size
pub(crate) const RCTL_RDMTS_HEX: u32 = 0x00010000;
pub(crate) const RCTL_MO_SHIFT: u32 = 12; // multicast offset shift
pub(crate) const RCTL_MO_3: u32 = 0x00003000; // multicast offset 15:4
pub(crate) const RCTL_BAM: u32 = 0x00008000; // broadcast enable

// Rx buffer size configurations when E1000_RCTL_BSEX is not set (0)
pub(crate) const RCTL_SZ_2048: u32 = 0x00000000;
pub(crate) const RCTL_SZ_1024: u32 = 0x00010000;
pub(crate) const RCTL_SZ_512: u32 = 0x00020000;
pub(crate) const RCTL_SZ_256: u32 = 0x00030000;

// Rx buffer size configurations when E1000_RCTL_BSEX is set (1)
pub(crate) const RCTL_SZ_16384: u32 = 0x00010000;
pub(crate) const RCTL_SZ_8192: u32 = 0x00020000;
pub(crate) const RCTL_SZ_4096: u32 = 0x00030000;

// Other RCTL flags
pub(crate) const RCTL_VFE: u32 = 0x00040000; // VLAN Filter Enable
pub(crate) const RCTL_CFIEN: u32 = 0x00080000; // Canonical Form Indicator Enable
pub(crate) const RCTL_CFI: u32 = 0x00100000; // Canonical Form Indicator
pub(crate) const RCTL_DPF: u32 = 0x00400000; // Discard Pause Frames
pub(crate) const RCTL_PMCF: u32 = 0x00800000; // Pass MAC Control Frames
pub(crate) const RCTL_BSEX: u32 = 0x02000000; // Buffer Size Extension
pub(crate) const RCTL_SECRC: u32 = 0x04000000; // Strip Ethernet CRC

pub(crate) const MASTER_DISABLE_TIMEOUT: usize = 800; // Number of 100 microseconds we wait for PCI Express master disable
pub(crate) const PHY_CFG_TIMEOUT: usize = 100; // Number of milliseconds we wait for PHY configuration done after MAC reset

/* Device Control Flags */
pub(crate) const CTRL_FD: u32 = 0x00000001; // Full duplex. 0=half; 1=full
pub(crate) const CTRL_GIO_MASTER_DISABLE: u32 = 0x00000004; // Blocks new Master requests
pub(crate) const CTRL_LRST: u32 = 0x00000008; // Link reset. 0=normal,1=reset
pub(crate) const CTRL_ASDE: u32 = 0x00000020; // Auto-speed detect enable
pub(crate) const CTRL_SLU: u32 = 0x00000040; // Set link up (Force Link)
pub(crate) const CTRL_ILOS: u32 = 0x00000080; // Invert Loss-Of Signal
pub(crate) const CTRL_SPD_SEL: u32 = 0x00000300; // Speed Select Mask
pub(crate) const CTRL_SPD_10: u32 = 0x00000000; // Force 10Mb
pub(crate) const CTRL_SPD_100: u32 = 0x00000100; // Force 100Mb
pub(crate) const CTRL_SPD_1000: u32 = 0x00000200; // Force 1Gb
pub(crate) const CTRL_FRCSPD: u32 = 0x00000800; // Force Speed
pub(crate) const CTRL_FRCDPX: u32 = 0x00001000; // Force Duplex
pub(crate) const CTRL_LANPHYPC_OVERRIDE: u32 = 0x00010000; // SW control of LANPHYPC
pub(crate) const CTRL_LANPHYPC_VALUE: u32 = 0x00020000; // SW value of LANPHYPC
pub(crate) const CTRL_MEHE: u32 = 0x00080000; // Memory Error Handling Enable
pub(crate) const CTRL_SWDPIN0: u32 = 0x00040000; // SWDPIN 0 value
pub(crate) const CTRL_SWDPIN1: u32 = 0x00080000; // SWDPIN 1 value
pub(crate) const CTRL_ADVD3WUC: u32 = 0x00100000; // D3 WUC
pub(crate) const CTRL_EN_PHY_PWR_MGMT: u32 = 0x00200000; // PHY PM enable
pub(crate) const CTRL_SWDPIO0: u32 = 0x00400000; // SWDPIN 0 Input or output
pub(crate) const CTRL_RST: u32 = 0x04000000; // Global reset
pub(crate) const CTRL_RFCE: u32 = 0x08000000; // Receive Flow Control enable
pub(crate) const CTRL_TFCE: u32 = 0x10000000; // Transmit flow control enable
pub(crate) const CTRL_VME: u32 = 0x40000000; // IEEE VLAN mode enable
pub(crate) const CTRL_PHY_RST: u32 = 0x80000000; // PHY Reset

/* Extended Device Control Flags */
pub(crate) const CTRL_EXT_LPCD: u32 = 0x00000004; // LCD Power Cycle Done
pub(crate) const CTRL_EXT_SDP3_DATA: u32 = 0x00000080; // Value of SW Definable Pin 3
pub(crate) const CTRL_EXT_FORCE_SMBUS: u32 = 0x00000800; // Force SMBus mode
pub(crate) const CTRL_EXT_EE_RST: u32 = 0x00002000; // Reinitialize from EEPROM
pub(crate) const CTRL_EXT_SPD_BYPS: u32 = 0x00008000; // Speed Select Bypass
pub(crate) const CTRL_EXT_RO_DIS: u32 = 0x00020000; // Relaxed Ordering disable
pub(crate) const CTRL_EXT_DMA_DYN_CLK_EN: u32 = 0x00080000; // DMA Dynamic Clock Gating
pub(crate) const CTRL_EXT_LINK_MODE_MASK: u32 = 0x00C00000;
pub(crate) const CTRL_EXT_LINK_MODE_PCIE_SERDES: u32 = 0x00C00000;
pub(crate) const CTRL_EXT_EIAME: u32 = 0x01000000;
pub(crate) const CTRL_EXT_DRV_LOAD: u32 = 0x10000000; // Driver loaded bit for FW
pub(crate) const CTRL_EXT_IAME: u32 = 0x08000000; // Int ACK Auto-mask
pub(crate) const CTRL_EXT_PBA_CLR: u32 = 0x80000000; // PBA Clear
pub(crate) const CTRL_EXT_LSECCK: u32 = 0x00001000;
pub(crate) const CTRL_EXT_PHYPDEN: u32 = 0x00100000;
pub(crate) const AUTO_READ_DONE_TIMEOUT: usize = 10; // Timeout value in milliseconds, now set to 10

/* NVM Control Flags */
pub(crate) const EECD_SK: u32 = 0x00000001; /* NVM Clock */
pub(crate) const EECD_CS: u32 = 0x00000002; /* NVM Chip Select */
pub(crate) const EECD_DI: u32 = 0x00000004; /* NVM Data In */
pub(crate) const EECD_DO: u32 = 0x00000008; /* NVM Data Out */
pub(crate) const EECD_REQ: u32 = 0x00000040; /* NVM Access Request */
pub(crate) const EECD_GNT: u32 = 0x00000080; /* NVM Access Grant */
pub(crate) const EECD_PRES: u32 = 0x00000100; /* NVM Present */
pub(crate) const EECD_SIZE: u32 = 0x00000200; /* NVM Size (0=64 word 1=256 word) */

pub(crate) const PBA_ECC_COUNTER_MASK: u32 = 0xFFF00000;
pub(crate) const PBA_ECC_COUNTER_SHIFT: u32 = 20;
pub(crate) const PBA_ECC_CORR_EN: u32 = 0x00000001;
pub(crate) const PBA_ECC_STAT_CLR: u32 = 0x00000002;
pub(crate) const PBA_ECC_INT_EN: u32 = 0x00000004;

/* Header split receive */
pub(crate) const RFCTL_NFSW_DIS: u32 = 0x00000040;
pub(crate) const RFCTL_NFSR_DIS: u32 = 0x00000080;
pub(crate) const RFCTL_ACK_DIS: u32 = 0x00001000;
pub(crate) const RFCTL_EXTEN: u32 = 0x00008000;
pub(crate) const RFCTL_IPV6_EX_DIS: u32 = 0x00010000;
pub(crate) const RFCTL_NEW_IPV6_EXT_DIS: u32 = 0x00020000;

pub(crate) const MNG_IAMT_MODE: u32 = 0x3;
pub(crate) const MNG_DHCP_COOKIE_LENGTH: usize = 0x10;
pub(crate) const MNG_DHCP_COOKIE_OFFSET: usize = 0x6F0;
pub(crate) const MNG_DHCP_COMMAND_TIMEOUT: u32 = 10;
pub(crate) const MNG_DHCP_TX_PAYLOAD_CMD: usize = 64;
pub(crate) const MNG_DHCP_COOKIE_STATUS_PARSING: u8 = 0x1;
pub(crate) const MNG_DHCP_COOKIE_STATUS_VLAN: u8 = 0x2;

pub(crate) const VFTA_ENTRY_SHIFT: u16 = 5;
pub(crate) const VFTA_ENTRY_MASK: u16 = 0x7F;
pub(crate) const VFTA_ENTRY_BIT_SHIFT_MASK: u16 = 0x1F;

/* 802.1q VLAN Packet Size */
pub(crate) const VLAN_FILTER_TBL_SIZE: usize = 128; // VLAN Filter Table (4096 bits)
pub(crate) const RAH_AV: u32 = 0x80000000;

// SerDes Control
pub(crate) const GEN_POLL_TIMEOUT: usize = 640;

// Generic MII registers
pub(crate) const MII_BMCR: u32 = 0x00; // Basic mode control register
pub(crate) const MII_BMSR: u32 = 0x01; // Basic mode status register
pub(crate) const MII_PHYSID1: u32 = 0x02; // PHYS ID 1
pub(crate) const MII_PHYSID2: u32 = 0x03; // PHYS ID 2
pub(crate) const MII_ADVERTISE: u32 = 0x04; // Advertisement control register
pub(crate) const MII_LPA: u32 = 0x05; // Link partner ability register
pub(crate) const MII_EXPANSION: u32 = 0x06; // Expansion register
pub(crate) const MII_CTRL1000: u32 = 0x09; // 1000BASE-T control
pub(crate) const MII_STAT1000: u32 = 0x0a; // 1000BASE-T status
pub(crate) const MII_MMD_CTRL: u8 = 0x0d; // MMD Access Control Register
pub(crate) const MII_MMD_DATA: u8 = 0x0e; // MMD Access Data Register
pub(crate) const MII_ESTATUS: u32 = 0x0f; // Extended Status
pub(crate) const MII_DCOUNTER: u8 = 0x12; // Disconnect counter
pub(crate) const MII_FCSCOUNTER: u8 = 0x13; // False carrier counter
pub(crate) const MII_NWAYTEST: u8 = 0x14; // N-way auto-neg test register
pub(crate) const MII_RERRCOUNTER: u8 = 0x15; // Receive error counter
pub(crate) const MII_SREVISION: u8 = 0x16; // Silicon revision
pub(crate) const MII_RESV1: u8 = 0x17; // Reserved...
pub(crate) const MII_LBRERROR: u8 = 0x18; // Lpback, rx, bypass error
pub(crate) const MII_PHYADDR: u8 = 0x19; // PHY address
pub(crate) const MII_RESV2: u8 = 0x1a; // Reserved...
pub(crate) const MII_TPISTATUS: u8 = 0x1b; // TPI status for 10mbps
pub(crate) const MII_NCONFIG: u8 = 0x1c; // Network interface config

// Basic mode control register
pub(crate) const BMCR_RESV: u16 = 0x003f; // Unused...
pub(crate) const BMCR_SPEED1000: u16 = 0x0040; // MSB of Speed (1000)
pub(crate) const BMCR_CTST: u16 = 0x0080; // Collision test
pub(crate) const BMCR_FULLDPLX: u16 = 0x0100; // Full duplex
pub(crate) const BMCR_ANRESTART: u16 = 0x0200; // Auto negotiation restart
pub(crate) const BMCR_ISOLATE: u16 = 0x0400; // Isolate data paths from MII
pub(crate) const BMCR_PDOWN: u16 = 0x0800; // Enable low power state
pub(crate) const BMCR_ANENABLE: u16 = 0x1000; // Enable auto negotiation
pub(crate) const BMCR_SPEED100: u16 = 0x2000; // Select 100Mbps
pub(crate) const BMCR_LOOPBACK: u16 = 0x4000; // TXD loopback bits
pub(crate) const BMCR_RESET: u16 = 0x8000; // Reset to default state
pub(crate) const BMCR_SPEED10: u16 = 0x0000; // Select 10Mbps

pub(crate) const MANC2H: usize = 0x05860; // Management Control To Host - RW
pub(crate) const HALF_DUPLEX: u16 = 1;
pub(crate) const FULL_DUPLEX: u16 = 2;

pub(crate) const ICS_LSC: u32 = ICR_LSC; // Link Status Change
pub(crate) const ICS_RXSEQ: u32 = ICR_RXSEQ; // Rx sequence error
pub(crate) const ICS_RXDMT0: u32 = ICR_RXDMT0; // Rx desc min. threshold
pub(crate) const ICS_OTHER: u32 = ICR_OTHER; // Other Interrupt

pub(crate) const IVAR_INT_ALLOC_VALID: u32 = 0x8;

pub(crate) const STATUS_PCIM_STATE: u32 = 0x40000000; // PCIm function state

pub(crate) const BMSR_ERCAP: u16 = 0x0001; // Extended register capability
pub(crate) const BMSR_JCD: u16 = 0x0002; // Jabber detected
pub(crate) const BMSR_LSTATUS: u16 = 0x0004; // Link status
pub(crate) const BMSR_ANEGCAPABLE: u16 = 0x0008; // Able to do auto-negotiation
pub(crate) const BMSR_RFAULT: u16 = 0x0010; // Remote fault detected
pub(crate) const BMSR_ANEGCOMPLETE: u16 = 0x0020; // Auto-negotiation complete
pub(crate) const BMSR_RESV: u16 = 0x00c0; // Unused...
pub(crate) const BMSR_ESTATEN: u16 = 0x0100; // Extended Status in R15
pub(crate) const BMSR_100HALF2: u16 = 0x0200; // Can do 100BASE-T2 HDX
pub(crate) const BMSR_100FULL2: u16 = 0x0400; // Can do 100BASE-T2 FDX
pub(crate) const BMSR_10HALF: u16 = 0x0800; // Can do 10mbps, half-duplex
pub(crate) const BMSR_10FULL: u16 = 0x1000; // Can do 10mbps, full-duplex
pub(crate) const BMSR_100HALF: u16 = 0x2000; // Can do 100mbps, half-duplex
pub(crate) const BMSR_100FULL: u16 = 0x4000; // Can do 100mbps, full-duplex
pub(crate) const BMSR_100BASE4: u16 = 0x8000; // Can do 100mbps, 4k packets

pub(crate) const ADVERTISE_SLCT: u16 = 0x001f; // Selector bits
pub(crate) const ADVERTISE_CSMA: u16 = 0x0001; // Only selector supported
pub(crate) const ADVERTISE_10HALF: u16 = 0x0020; // Try for 10mbps half-duplex
pub(crate) const ADVERTISE_1000XFULL: u16 = 0x0020; // Try for 1000BASE-X full-duplex
pub(crate) const ADVERTISE_10FULL: u16 = 0x0040; // Try for 10mbps full-duplex
pub(crate) const ADVERTISE_1000XHALF: u16 = 0x0040; // Try for 1000BASE-X half-duplex
pub(crate) const ADVERTISE_100HALF: u16 = 0x0080; // Try for 100mbps half-duplex
pub(crate) const ADVERTISE_1000XPAUSE: u16 = 0x0080; // Try for 1000BASE-X pause
pub(crate) const ADVERTISE_100FULL: u16 = 0x0100; // Try for 100mbps full-duplex
pub(crate) const ADVERTISE_1000XPSE_ASYM: u16 = 0x0100; // Try for 1000BASE-X asym pause
pub(crate) const ADVERTISE_100BASE4: u16 = 0x0200; // Try for 100mbps 4k packets
pub(crate) const ADVERTISE_PAUSE_CAP: u16 = 0x0400; // Try for pause
pub(crate) const ADVERTISE_PAUSE_ASYM: u16 = 0x0800; // Try for asymetric pause
pub(crate) const ADVERTISE_RESV: u16 = 0x1000; // Unused...
pub(crate) const ADVERTISE_RFAULT: u16 = 0x2000; // Say we can detect faults
pub(crate) const ADVERTISE_LPACK: u16 = 0x4000; // Ack link partners response
pub(crate) const ADVERTISE_NPAGE: u16 = 0x8000; // Next page bit

pub(crate) const ADVERTISE_FULL: u16 = ADVERTISE_100FULL | ADVERTISE_10FULL | ADVERTISE_CSMA;
pub(crate) const ADVERTISE_ALL: u16 =
    ADVERTISE_10HALF | ADVERTISE_10FULL | ADVERTISE_100HALF | ADVERTISE_100FULL;

pub(crate) const EXPANSION_NWAY: u16 = 0x0001; // Can do N-way auto-negotiation
pub(crate) const EXPANSION_LCWP: u16 = 0x0002; // Got new RX page code word
pub(crate) const EXPANSION_ENABLENPAGE: u16 = 0x0004; // Enables npage words
pub(crate) const EXPANSION_NPCAPABLE: u16 = 0x0008; // Link partner supports npage
pub(crate) const EXPANSION_MFAULTS: u16 = 0x0010; // Multiple faults detected
pub(crate) const EXPANSION_RESV: u16 = 0xffe0; // Unused...

pub(crate) const ESTATUS_1000_XFULL: u16 = 0x8000; // Can do 1000BaseX Full
pub(crate) const ESTATUS_1000_XHALF: u16 = 0x4000; // Can do 1000BaseX Half
pub(crate) const ESTATUS_1000_TFULL: u16 = 0x2000; // Can do 1000BT Full
pub(crate) const ESTATUS_1000_THALF: u16 = 0x1000; // Can do 1000BT Half

pub(crate) const ADVERTISE_1000FULL: u16 = 0x0200; // Advertise 1000BASE-T full duplex
pub(crate) const ADVERTISE_1000HALF: u16 = 0x0100; // Advertise 1000BASE-T half duplex
pub(crate) const CTL1000_PREFER_MASTER: u16 = 0x0400; // Prefer to operate as master
pub(crate) const CTL1000_AS_MASTER: u16 = 0x0800; // Operate as master
pub(crate) const CTL1000_ENABLE_MASTER: u16 = 0x1000; // Enable master/slave functionality
                                                      // Given the macro definition and understanding it's not practically used but for informational purposes:

pub(crate) const M88E1000_PHY_SPEC_CTRL: u32 = 0x10; // PHY Specific Control Register
pub(crate) const M88E1000_PHY_SPEC_STATUS: u32 = 0x11; // PHY Specific Status Register
pub(crate) const M88E1000_EXT_PHY_SPEC_CTRL: u8 = 0x14; // Extended PHY Specific Control Register

pub(crate) const M88E1000_PSSR_REV_POLARITY: u16 = 0x0002; // 1 indicates polarity reversed
pub(crate) const M88E1000_PSSR_DOWNSHIFT: u16 = 0x0020; // 1 indicates downshifted
pub(crate) const M88E1000_PSSR_MDIX: u16 = 0x0040; // 1 for MDIX, 0 for MDI

pub(crate) const IGP01E1000_PHY_PORT_CONFIG: u8 = 0x10; // Port Configuration Register
pub(crate) const IGP01E1000_PHY_PORT_STATUS: u8 = 0x11; // Status Register
pub(crate) const IGP01E1000_PHY_PORT_CTRL: u8 = 0x12; // Control Register
pub(crate) const IGP01E1000_PHY_LINK_HEALTH: u32 = 0x13; // PHY Link Health Register
pub(crate) const IGP02E1000_PHY_POWER_MGMT: u8 = 0x19; // Power Management Register
pub(crate) const IGP01E1000_PHY_PAGE_SELECT: u32 = 0x1F; // Page Select Register
pub(crate) const BM_PHY_PAGE_SELECT: u8 = 22; // Page Select for BM (Base Management)
pub(crate) const IGP_PAGE_SHIFT: u16 = 5; // Shift amount for IGP page addressing

pub(crate) const IGP01E1000_PLHR_SS_DOWNGRADE: u16 = 0x8000; // Signal Speed Downgrade in PHY Link Health Register

pub(crate) const M88E1000_PSCR_AUTO_X_MODE: u16 = 0x0060; // Bitmask for Auto MDI/MDI-X mode

pub(crate) const ADVERTISED_Pause: u16 = 1 << 13;
pub(crate) const ADVERTISED_Asym_Pause: u16 = 1 << 14;

pub(crate) const RXCSUM_IPOFLD: u32 = 0x00000100; /* TCP / UDP checksum offload */
pub(crate) const RXCSUM_TUOFL: u32 = 0x00000200; /* TCP / UDP checksum offload */
pub(crate) const PBECCSTS_ECC_ENABLE: u32 = 0x00010000;

pub(crate) const PHY_CTRL_D0A_LPLU: u32 = 0x00000002;
pub(crate) const PHY_CTRL_NOND0A_LPLU: u32 = 0x00000004;
pub(crate) const PHY_CTRL_NOND0A_GBE_DISABLE: u32 = 0x00000008;
pub(crate) const PHY_CTRL_GBE_DISABLE: u32 = 0x00000040;

pub(crate) const KMRNCTRLSTA_OFFSET: u32 = 0x001F0000;
pub(crate) const KMRNCTRLSTA_OFFSET_SHIFT: u32 = 16;
pub(crate) const KMRNCTRLSTA_REN: u32 = 0x00200000;
pub(crate) const KMRNCTRLSTA_CTRL_OFFSET: u32 = 0x1;
pub(crate) const KMRNCTRLSTA_DIAG_OFFSET: u8 = 0x3;
pub(crate) const KMRNCTRLSTA_TIMEOUTS: u8 = 0x4;
pub(crate) const KMRNCTRLSTA_INBAND_PARAM: u8 = 0x9;
pub(crate) const KMRNCTRLSTA_IBIST_DISABLE: u16 = 0x0200;
pub(crate) const KMRNCTRLSTA_DIAG_NELPBK: u16 = 0x1000;
pub(crate) const KMRNCTRLSTA_K1_CONFIG: u8 = 0x7;
pub(crate) const KMRNCTRLSTA_K1_ENABLE: u16 = 0x0002;
pub(crate) const KMRNCTRLSTA_HD_CTRL: u32 = 0x10;

pub(crate) const FEXTNVM4_BEACON_DURATION_MASK: u32 = 0x7;
pub(crate) const FEXTNVM4_BEACON_DURATION_8USEC: u32 = 0x7;
pub(crate) const FEXTNVM4_BEACON_DURATION_16USEC: u32 = 0x3;

pub(crate) const I217_EEE_PCS_STATUS: u16 = 0x9401; // IEEE MMD Register 3.1
pub(crate) const I217_EEE_CAPABILITY: u16 = 0x8000; // IEEE MMD Register 3.20
pub(crate) const I217_EEE_ADVERTISEMENT: u16 = 0x8001; // IEEE MMD Register 7.60
pub(crate) const I217_EEE_LP_ABILITY: u16 = 0x8002; // IEEE MMD Register 7.61

pub(crate) const I82579_EEE_PCS_STATUS: u16 = 0x182E; // IEEE MMD Register 3.1 >> 8
pub(crate) const I82579_EEE_CAPABILITY: u16 = 0x0410; // IEEE MMD Register 3.20
pub(crate) const I82579_EEE_ADVERTISEMENT: u16 = 0x040E; // IEEE MMD Register 7.60
pub(crate) const I82579_EEE_LP_ABILITY: u16 = 0x040F; // IEEE MMD Register 7.61

pub(crate) const LPA_PAUSE_CAP: u16 = 0x0400; /* Can pause                   */
pub(crate) const LPA_PAUSE_ASYM: u16 = 0x0800; /* Can pause asymetrically     */

use core::arch::asm;

pub(crate) fn fence() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence iorw, iorw");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
pub(crate) fn fence_w() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ow, ow");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
pub(crate) fn fence_r() {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        asm!("fence ir, ir");
    }

    #[cfg(not(target_arch = "riscv64"))]
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}

pub(crate) const FWSM_WLOCK_MAC_MASK: u32 = 0x0380;
pub(crate) const FWSM_WLOCK_MAC_SHIFT: u32 = 7;
pub(crate) const FWSM_ULP_CFG_DONE: u32 = 0x00000400;
pub(crate) const EXFWSM_DPG_EXIT_DONE: u32 = 0x00000001;

/* Shared Receive Address Registers */
pub(crate) const fn SHRAL_PCH_LPT(i: usize) -> usize {
    0x05408 + (i * 8)
}

pub(crate) fn SHRAH_PCH_LPT(i: usize) -> usize {
    0x0540C + (i * 8)
}

pub(crate) fn MDEF(n: usize) -> usize {
    0x05890 + (4 * n)
}

pub(crate) const ADVERTISE_10_HALF: u16 = 0x0001;
pub(crate) const ADVERTISE_10_FULL: u16 = 0x0002;
pub(crate) const ADVERTISE_100_HALF: u16 = 0x0004;
pub(crate) const ADVERTISE_100_FULL: u16 = 0x0008;
pub(crate) const ADVERTISE_1000_HALF: u16 = 0x0010;
pub(crate) const ADVERTISE_1000_FULL: u16 = 0x0020;

pub(crate) const E1000_ALL_SPEED_DUPLEX: u16 = ADVERTISE_10_HALF
    | ADVERTISE_10_FULL
    | ADVERTISE_100_HALF
    | ADVERTISE_100_FULL
    | ADVERTISE_1000_FULL;
pub(crate) const E1000_ALL_NOT_GIG: u16 =
    ADVERTISE_10_HALF | ADVERTISE_10_FULL | ADVERTISE_100_HALF | ADVERTISE_100_FULL;
pub(crate) const E1000_ALL_100_SPEED: u16 = ADVERTISE_100_HALF | ADVERTISE_100_FULL;
pub(crate) const E1000_ALL_10_SPEED: u16 = ADVERTISE_10_HALF | ADVERTISE_10_FULL;
pub(crate) const E1000_ALL_HALF_DUPLEX: u16 = ADVERTISE_10_HALF | ADVERTISE_100_HALF;

pub(crate) const AUTONEG_ADVERTISE_SPEED_DEFAULT: u16 = E1000_ALL_SPEED_DUPLEX;

pub(crate) const MIN_NUM_XMITS: u32 = 1000; // Minimum number of transmissions

pub(crate) const SWSM2_LOCK: u32 = 0x00000002; // Bitmask for secondary driver semaphore lock

pub(crate) const FLOW_CONTROL_ADDRESS_LOW: u32 = 0x00C28001; // Lower part of the flow control address
pub(crate) const FLOW_CONTROL_ADDRESS_HIGH: u32 = 0x00000100; // Higher part of the flow control address
pub(crate) const FLOW_CONTROL_TYPE: u32 = 0x8808; // EtherType for flow control packets (802.3x PAUSE frame)

pub(crate) const GCR_RXD_NO_SNOOP: u32 = 0x00000001;
pub(crate) const GCR_RXDSCW_NO_SNOOP: u32 = 0x00000002;
pub(crate) const GCR_RXDSCR_NO_SNOOP: u32 = 0x00000004;
pub(crate) const GCR_TXD_NO_SNOOP: u32 = 0x00000008;
pub(crate) const GCR_TXDSCW_NO_SNOOP: u32 = 0x00000010;
pub(crate) const GCR_TXDSCR_NO_SNOOP: u32 = 0x00000020;

pub(crate) const PCIE_NO_SNOOP_ALL: u32 = GCR_RXD_NO_SNOOP
    | GCR_RXDSCW_NO_SNOOP
    | GCR_RXDSCR_NO_SNOOP
    | GCR_TXD_NO_SNOOP
    | GCR_TXDSCW_NO_SNOOP
    | GCR_TXDSCR_NO_SNOOP;

pub(crate) const FFLT_DBG_DONT_GATE_WAKE_DMA_CLK: u32 = 0x1000;

pub(crate) const M88E1000_E_PHY_ID: u32 = 0x01410C50;
pub(crate) const M88E1000_I_PHY_ID: u32 = 0x01410C30;
pub(crate) const M88E1011_I_PHY_ID: u32 = 0x01410C20;
pub(crate) const IGP01E1000_I_PHY_ID: u32 = 0x02A80380;
pub(crate) const M88E1111_I_PHY_ID: u32 = 0x01410CC0;
pub(crate) const GG82563_E_PHY_ID: u32 = 0x01410CA0;
pub(crate) const IGP03E1000_E_PHY_ID: u32 = 0x02A80390;
pub(crate) const IFE_E_PHY_ID: u32 = 0x02A80330;
pub(crate) const IFE_PLUS_E_PHY_ID: u32 = 0x02A80320;
pub(crate) const IFE_C_E_PHY_ID: u32 = 0x02A80310;
pub(crate) const BME1000_E_PHY_ID: u32 = 0x01410CB0;
pub(crate) const BME1000_E_PHY_ID_R2: u32 = 0x01410CB1;
pub(crate) const I82577_E_PHY_ID: u32 = 0x01540050;
pub(crate) const I82578_E_PHY_ID: u32 = 0x004DD040;
pub(crate) const I82579_E_PHY_ID: u32 = 0x01540090;
pub(crate) const I217_E_PHY_ID: u32 = 0x015400A0;

pub(crate) const PHY_REVISION_MASK: u32 = 0xFFFFFFF0;
pub(crate) const MAX_PHY_MULTI_PAGE_REG: u16 = 0xF;

pub(crate) const KABGTXD_BGSQLBIAS: u32 = 0x00050000; // Band Gap SQL BIAS

pub(crate) const EXTCNF_CTRL_MDIO_SW_OWNERSHIP: u32 = 0x00000020;
pub(crate) const EXTCNF_CTRL_LCD_WRITE_ENABLE: u32 = 0x00000001;
pub(crate) const EXTCNF_CTRL_OEM_WRITE_ENABLE: u32 = 0x00000008;
pub(crate) const EXTCNF_CTRL_SWFLAG: u32 = 0x00000020; // Note: This seems to be a duplicate of MDIO_SW_OWNERSHIP.
pub(crate) const EXTCNF_CTRL_GATE_PHY_CFG: u32 = 0x00000080;
pub(crate) const EXTCNF_SIZE_EXT_PCIE_LENGTH_MASK: u32 = 0x00FF0000;
pub(crate) const EXTCNF_SIZE_EXT_PCIE_LENGTH_SHIFT: u32 = 16;
pub(crate) const EXTCNF_CTRL_EXT_CNF_POINTER_MASK: u32 = 0x0FFF0000;
pub(crate) const EXTCNF_CTRL_EXT_CNF_POINTER_SHIFT: u32 = 16;

/* Manageability Operation Mode mask */
pub(crate) const NVM_INIT_CTRL2_MNGM: u16 = 0x6000;



pub(crate) const ICH_FWSM_RSPCIPHY: u32 = 0x00000040; // Reset PHY on PCI Reset
pub(crate) const ICH_FWSM_FW_VALID: u32 = 0x00008000; // FW established a valid mode
pub(crate) const ICH_FWSM_PCIM2PCI: u32 = 0x01000000; // ME PCIm-to-PCI active
pub(crate) const ICH_FWSM_PCIM2PCI_COUNT: u32 = 2000;
pub(crate) const ICH_MNG_IAMT_MODE: u32 = 0x2;