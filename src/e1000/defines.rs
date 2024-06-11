#![allow(non_snake_case)]

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

pub(crate) const FEXTNVM7_DISABLE_PB_READ: u32 = 0x00040000; // Bit for disabling packet buffer read
pub(crate) const FEXTNVM7_SIDE_CLK_UNGATE: u32 = 0x00000004;
pub(crate) const FEXTNVM7_DISABLE_SMB_PERST: u32 = 0x00000020;
pub(crate) const FEXTNVM9_IOSFSB_CLKGATE_DIS: u32 = 0x00000800;
pub(crate) const FEXTNVM9_IOSFSB_CLKREQ_DIS: u32 = 0x00001000;
pub(crate) const FEXTNVM11_DISABLE_PB_READ: u32 = 0x00000200; // Disable packet buffer read
pub(crate) const FEXTNVM11_DISABLE_MULR_FIX: u32 = 0x00002000;

pub(crate) const RXDCTL_THRESH_UNIT_DESC: u32 = 0x01000000;

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

pub fn BIT(n: usize) -> u32 {
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
// 注意重复的定义，根据实际用途可能需要调整或去除重复项
// TXD_CMD_TCP, TXD_CMD_IP, TXD_CMD_TSE 与之前的 E1000_TXD_CMD_EOP, E1000_TXD_CMD_IFCS, E1000_TXD_CMD_IC 重复
// TXD_STAT_TC 与 TXD_STAT_LC 重复定义
pub(crate) const TXD_EXTCMD_TSTAMP: u32 = 0x00000010; /* IEEE1588 Timestamp packet */

pub(crate) const TCTL_RST: u32 = 0x00000001; // Software reset
pub(crate) const TCTL_EN: u32 = 0x00000002; // Enable TX
pub(crate) const TCTL_BCE: u32 = 0x00000004; // Busy check enable
pub(crate) const TCTL_PSP: u32 = 0x00000008; // Pad short packets
pub(crate) const TCTL_CT_SHIFT: u32 = 4; // Collision threshold shift
pub(crate) const TCTL_COLD_SHIFT: u32 = 12; // Collision distance shift
pub(crate) const TCTL_SWXOFF: u32 = 0x00400000; // SW Xoff transmission
pub(crate) const TCTL_PBE: u32 = 0x00800000; // Packet Burst Enable
pub(crate) const TCTL_RTLC: u32 = 0x01000000; // Re-transmit on late collision
pub(crate) const TCTL_NRTU: u32 = 0x02000000; // No re-transmit on underrun
pub(crate) const TCTL_MULR: u32 = 0x10000000; // Multiple request support

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
