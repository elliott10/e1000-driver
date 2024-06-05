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
    IMS_RXT0 |
    IMS_TXDW |
    IMS_RXDMT0 |
    IMS_RXSEQ |
    IMS_LSC;

pub(crate) const IMS_OTHER_MASK: u32 = 
    IMS_LSC |
    IMS_RXO |
    IMS_MDAC |
    IMS_SRPD |
    IMS_ACK |
    IMS_MNG;