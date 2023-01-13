// e1000 Driver for Intel 82540EP/EM

// Slice切片，内存連續的動態大小的序列；

// array, 数组
// Vec, 内存連續的可增長數組類型

use volatile::Volatile;
// let mut value: u32 = 0;
// let val: Volatile<u32> = Volatile::new(&mut value);
// val.write(1);
// assert_eq!(val.read(), 1);



pub const TX_RING_SIZE: u32 = 16;
//static struct tx_desc tx_ring[TX_RING_SIZE] __attribute__((aligned(16)));
//static struct mbuf *tx_mbufs[TX_RING_SIZE];

pub const RX_RING_SIZE: u32 = 16;
//static struct rx_desc rx_ring[RX_RING_SIZE] __attribute__((aligned(16)));
//static struct mbuf *rx_mbufs[RX_RING_SIZE];


/// xregs is the memory address at which the e1000's registers are mapped.
pub fn e1000_init(uint32 *xregs)
{


// 由一个指针和一个长度len形成一个slice切片。len是元素的个数，而非字节数。


  let len = 0x1FFFF/mem::size_of::<u32>();
  let regs = unsafe{ from_raw_parts_mut(mapped_regs as *mut Volatile<u32>, len) };

  // Reset the device
  regs[E1000_IMS].write(0); // disable interrupts
  regs[E1000_CTL].write(regs[E1000_CTL].read() | E1000_CTL_RST);
  regs[E1000_IMS].write(0); // redisable interrupts
  
  // 内存壁垒 fence
  //__sync_synchronize();

  // [E1000 14.5] Transmit initialization
  memset(tx_ring, 0, sizeof(tx_ring));

  for i in 0..TX_RING_SIZE {
    tx_ring[i].status = E1000_TXD_STAT_DD;
    tx_mbufs[i] = 0;
  }
  regs[E1000_TDBAL].write(tx_ring);
  if sizeof(tx_ring) % 128 != 0 {
    panic("e1000");
  }

  regs[E1000_TDLEN].write(sizeof(tx_ring));
  regs[E1000_TDT].write(0);
  regs[E1000_TDH].write(0);
  
  // [E1000 14.4] Receive initialization
  memset(rx_ring, 0, sizeof(rx_ring));
  for i in 0..RX_RING_SIZE {
    rx_mbufs[i] = mbufalloc(0);
    if (!rx_mbufs[i])
      panic("e1000");
    rx_ring[i].addr = (uint64) rx_mbufs[i]->head;
  }

  regs[E1000_RDBAL].write(rx_ring);
  if(sizeof(rx_ring) % 128 != 0)
    panic("e1000");

  regs[E1000_RDH].write(0);
  regs[E1000_RDT].write(RX_RING_SIZE - 1);
  regs[E1000_RDLEN].write(sizeof(rx_ring));

  // filter by qemu's MAC address, 52:54:00:12:34:56
  regs[E1000_RA].write(0x12005452);
  regs[E1000_RA+1].write(0x5634 | (1<<31));
  // multicast table
  for i in 0..4096/32 {
    regs[E1000_MTA + i].write(0);
  }
  // transmitter control bits.
  regs[E1000_TCTL].write( E1000_TCTL_EN |  // enable
    E1000_TCTL_PSP |                  // pad short packets
    (0x10 << E1000_TCTL_CT_SHIFT) |   // collision stuff
    (0x40 << E1000_TCTL_COLD_SHIFT) );
  regs[E1000_TIPG].write( 10 | (8<<10) | (6<<20) ); // inter-pkt gap

  // receiver control bits.
  regs[E1000_RCTL].write( E1000_RCTL_EN | // enable receiver
    E1000_RCTL_BAM |                 // enable broadcast
    E1000_RCTL_SZ_2048 |             // 2048-byte rx buffers
    E1000_RCTL_SECRC );                // strip CRC
  
  // ask e1000 for receive interrupts.
  regs[E1000_RDTR].write(0); // interrupt after every received packet (no timer)
  regs[E1000_RADV].write(0); // interrupt after every packet (no timer)
  regs[E1000_IMS].write(1 << 7); // RXDW -- Receiver Descriptor Write Back
}