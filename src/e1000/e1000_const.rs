// E1000 hardware definitions: registers and DMA ring format.
// from the Intel 82540EP/EM &c manual.

/* Registers */
pub(crate) const E1000_CTL: usize = 0x00000 / 4; /* Device Control Register - RW */
pub(crate) const E1000_STAT: usize = 0x00008 / 4; /* Device Status Register - R */
pub(crate) const E1000_ICR: usize = 0x000C0 / 4; /* Interrupt Cause Read - R */
pub(crate) const E1000_ITR: usize = 0x000C4 / 4; /* Interrupt Throttling Rate - RW */
pub(crate) const E1000_ICS: usize = 0x000C8 / 4; /* Interrupt Cause Set - WO */
pub(crate) const E1000_IMS: usize = 0x000D0 / 4; /* Interrupt Mask Set - RW */
pub(crate) const E1000_IMC: usize = 0x000D8 / 4; /* Interrupt Mask Clear - WO */
pub(crate) const E1000_RCTL: usize = 0x00100 / 4; /* RX Control - RW */
pub(crate) const E1000_TCTL: usize = 0x00400 / 4; /* TX Control - RW */
pub(crate) const E1000_TIPG: usize = 0x00410 / 4; /* TX Inter-packet gap -RW */
pub(crate) const E1000_RDBAL: usize = 0x02800 / 4; /* RX Descriptor Base Address Low - RW */
pub(crate) const E1000_RDBAH: usize = 0x02804 / 4; /* RX Descriptor Base Address High - RW */
pub(crate) const E1000_RDTR: usize = 0x02820 / 4; /* RX Delay Timer */
pub(crate) const E1000_RADV: usize = 0x0282C / 4; /* RX Interrupt Absolute Delay Timer */
pub(crate) const E1000_RDH: usize = 0x02810 / 4; /* RX Descriptor Head - RW */
pub(crate) const E1000_RDT: usize = 0x02818 / 4; /* RX Descriptor Tail - RW */
pub(crate) const E1000_RDLEN: usize = 0x02808 / 4; /* RX Descriptor Length - RW */
pub(crate) const E1000_RSRPD: usize = 0x02C00 / 4; /* RX Small Packet Detect Interrupt */
pub(crate) const E1000_TDBAL: usize = 0x03800 / 4; /* TX Descriptor Base Address Low - RW */
pub(crate) const E1000_TDBAH: usize = 0x03804 / 4; /* TX Descriptor Base Address High - RW */
pub(crate) const E1000_TDLEN: usize = 0x03808 / 4; /* TX Descriptor Length - RW */
pub(crate) const E1000_TDH: usize = 0x03810 / 4; /* TX Descriptor Head - RW */
pub(crate) const E1000_TDT: usize = 0x03818 / 4; /* TX Descripotr Tail - RW */
pub(crate) const E1000_TIDV: usize = 0x03820 / 4; /* TX Interrupt Delay Value - RW */
pub(crate) const E1000_TADV: usize = 0x0382C / 4; /* TX Interrupt Absolute Delay Val - RW */
pub(crate) const E1000_MTA: usize = 0x05200 / 4; /* Multicast Table Array - RW Array */
pub(crate) const E1000_RA: usize = 0x05400 / 4; /* Receive Address - RW Array */

/* This defines the bits that are set in the Interrupt Mask
 * Set/Read Register.  Each bit is documented below:
 *   o RXT0   = Receiver Timer Interrupt (ring 0)
 *   o TXDW   = Transmit Descriptor Written Back
 *   o RXDMT0 = Receive Descriptor Minimum Threshold hit (ring 0)
 *   o RXSEQ  = Receive Sequence Error
 *   o LSC    = Link Status Change
 */
pub(crate) const IMS_ENABLE_MASK: u32 = E1000_IMS_RXT0 | E1000_IMS_RXDMT0 | E1000_IMS_RXSEQ
    | E1000_IMS_LSC /* | E1000_IMS_TXQE | E1000_IMS_TXDW */;

pub(crate) const E1000_IMS_TXDW: u32 = 0x00000001;
pub(crate) const E1000_IMS_TXQE: u32 = 0x00000002;
pub(crate) const E1000_IMS_LSC: u32 = 0x00000004;
pub(crate) const E1000_IMS_RXSEQ: u32 = 0x00000008;
pub(crate) const E1000_IMS_RXDMT0: u32 = 0x00000010;
pub(crate) const E1000_IMS_RXT0: u32 = 0x00000080;

pub(crate) const E1000_ICR_LSC: u32 = 0x00000004; /* Link Status Change */

/* Device Control */
pub(crate) const E1000_CTL_SLU: u32 = 0x00000040; /* set link up */
pub(crate) const E1000_CTL_FRCSPD: u32 = 0x00000800; /* force speed */
pub(crate) const E1000_CTL_FRCDPLX: u32 = 0x00001000; /* force duplex */
pub(crate) const E1000_CTL_RST: u32 = 0x00400000; /* full reset */

/* Transmit Control */
pub(crate) const E1000_TCTL_RST: u32 = 0x00000001; /* software reset */
pub(crate) const E1000_TCTL_EN: u32 = 0x00000002; /* enable tx */
pub(crate) const E1000_TCTL_BCE: u32 = 0x00000004; /* busy check enable */
pub(crate) const E1000_TCTL_PSP: u32 = 0x00000008; /* pad short packets */
pub(crate) const E1000_TCTL_CT: u32 = 0x00000ff0; /* collision threshold */
pub(crate) const E1000_TCTL_CT_SHIFT: u32 = 4;
pub(crate) const E1000_TCTL_COLD: u32 = 0x003ff000; /* collision distance */
pub(crate) const E1000_TCTL_COLD_SHIFT: u32 = 12;
pub(crate) const E1000_TCTL_SWXOFF: u32 = 0x00400000; /* SW Xoff transmission */
pub(crate) const E1000_TCTL_PBE: u32 = 0x00800000; /* Packet Burst Enable */
pub(crate) const E1000_TCTL_RTLC: u32 = 0x01000000; /* Re-transmit on late collision */
pub(crate) const E1000_TCTL_NRTU: u32 = 0x02000000; /* No Re-transmit on underrun */
pub(crate) const E1000_TCTL_MULR: u32 = 0x10000000; /* Multiple request support */

/* Receive Control */
pub(crate) const E1000_RCTL_RST: u32 = 0x00000001; /* Software reset */
pub(crate) const E1000_RCTL_EN: u32 = 0x00000002; /* enable */
pub(crate) const E1000_RCTL_SBP: u32 = 0x00000004; /* store bad packet */
pub(crate) const E1000_RCTL_UPE: u32 = 0x00000008; /* unicast promiscuous enable */
pub(crate) const E1000_RCTL_MPE: u32 = 0x00000010; /* multicast promiscuous enab */
pub(crate) const E1000_RCTL_LPE: u32 = 0x00000020; /* long packet enable */
pub(crate) const E1000_RCTL_LBM_NO: u32 = 0x00000000; /* no loopback mode */
pub(crate) const E1000_RCTL_LBM_MAC: u32 = 0x00000040; /* MAC loopback mode */
pub(crate) const E1000_RCTL_LBM_SLP: u32 = 0x00000080; /* serial link loopback mode */
pub(crate) const E1000_RCTL_LBM_TCVR: u32 = 0x000000C0; /* tcvr loopback mode */
pub(crate) const E1000_RCTL_DTYP_MASK: u32 = 0x00000C00; /* Descriptor type mask */
pub(crate) const E1000_RCTL_DTYP_PS: u32 = 0x00000400; /* Packet Split descriptor */
pub(crate) const E1000_RCTL_RDMTS_HALF: u32 = 0x00000000; /* rx desc min threshold size */
pub(crate) const E1000_RCTL_RDMTS_QUAT: u32 = 0x00000100; /* rx desc min threshold size */
pub(crate) const E1000_RCTL_RDMTS_EIGTH: u32 = 0x00000200; /* rx desc min threshold size */
pub(crate) const E1000_RCTL_MO_SHIFT: u32 = 12; /* multicast offset shift */
pub(crate) const E1000_RCTL_MO_0: u32 = 0x00000000; /* multicast offset 11:0 */
pub(crate) const E1000_RCTL_MO_1: u32 = 0x00001000; /* multicast offset 12:1 */
pub(crate) const E1000_RCTL_MO_2: u32 = 0x00002000; /* multicast offset 13:2 */
pub(crate) const E1000_RCTL_MO_3: u32 = 0x00003000; /* multicast offset 15:4 */
pub(crate) const E1000_RCTL_MDR: u32 = 0x00004000; /* multicast desc ring 0 */
pub(crate) const E1000_RCTL_BAM: u32 = 0x00008000; /* broadcast enable */
/* these buffer sizes are valid if E1000_RCTL_BSEX is 0 */
pub(crate) const E1000_RCTL_SZ_2048: u32 = 0x00000000; /* rx buffer size 2048 */
pub(crate) const E1000_RCTL_SZ_1024: u32 = 0x00010000; /* rx buffer size 1024 */
pub(crate) const E1000_RCTL_SZ_512: u32 = 0x00020000; /* rx buffer size 512 */
pub(crate) const E1000_RCTL_SZ_256: u32 = 0x00030000; /* rx buffer size 256 */
/* these buffer sizes are valid if E1000_RCTL_BSEX is 1 */
pub(crate) const E1000_RCTL_SZ_16384: u32 = 0x00010000; /* rx buffer size 16384 */
pub(crate) const E1000_RCTL_SZ_8192: u32 = 0x00020000; /* rx buffer size 8192 */
pub(crate) const E1000_RCTL_SZ_4096: u32 = 0x00030000; /* rx buffer size 4096 */
pub(crate) const E1000_RCTL_VFE: u32 = 0x00040000; /* vlan filter enable */
pub(crate) const E1000_RCTL_CFIEN: u32 = 0x00080000; /* canonical form enable */
pub(crate) const E1000_RCTL_CFI: u32 = 0x00100000; /* canonical form indicator */
pub(crate) const E1000_RCTL_DPF: u32 = 0x00400000; /* discard pause frames */
pub(crate) const E1000_RCTL_PMCF: u32 = 0x00800000; /* pass MAC control frames */
pub(crate) const E1000_RCTL_BSEX: u32 = 0x02000000; /* Buffer size extension */
pub(crate) const E1000_RCTL_SECRC: u32 = 0x04000000; /* Strip Ethernet CRC */
pub(crate) const E1000_RCTL_FLXBUF_MASK: u32 = 0x78000000; /* Flexible buffer size */
pub(crate) const E1000_RCTL_FLXBUF_SHIFT: u32 = 27; /* Flexible buffer shift */

pub(crate) const DATA_MAX: u32 = 1518;

/* Transmit Descriptor command definitions [E1000 3.3.3.1] */
pub(crate) const E1000_TXD_CMD_EOP: u32 = 0x01; /* End of Packet */
pub(crate) const E1000_TXD_CMD_RS: u32 = 0x08; /* Report Status */

/* Transmit Descriptor status definitions [E1000 3.3.3.2] */
pub(crate) const E1000_TXD_STAT_DD: u32 = 0x00000001; /* Descriptor Done */

/* Receive Descriptor bit definitions [E1000 3.2.3.1] */
pub(crate) const E1000_RXD_STAT_DD: u32 = 0x01; /* Descriptor Done */
pub(crate) const E1000_RXD_STAT_EOP: u32 = 0x02; /* End of Packet */
