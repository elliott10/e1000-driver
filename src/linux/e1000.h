#ifndef _E1000_H_
#define _E1000_H_

#include "e1000_bindings.h"
#include "hw.h"
#include <asm/byteorder.h>
#include <asm/io.h>
#include <asm/irq.h>
#include <linux/bitops.h>
#include <linux/capability.h>
#include <linux/delay.h>
#include <linux/dma-mapping.h>
#include <linux/errno.h>
#include <linux/etherdevice.h>
#include <linux/ethtool.h>
#include <linux/if_vlan.h>
#include <linux/in.h>
#include <linux/interrupt.h>
#include <linux/ioport.h>
#include <linux/ip.h>
#include <linux/ipv6.h>
#include <linux/kernel.h>
#include <linux/list.h>
#include <linux/mii.h>
#include <linux/mm.h>
#include <linux/module.h>
#include <linux/netdevice.h>
#include <linux/pagemap.h>
#include <linux/pci.h>
#include <linux/reboot.h>
#include <linux/skbuff.h>
#include <linux/slab.h>
#include <linux/stddef.h>
#include <linux/string.h>
#include <linux/tcp.h>
#include <linux/timer.h>
#include <linux/types.h>
#include <linux/udp.h>
#include <linux/vmalloc.h>
#include <net/checksum.h>
#include <net/pkt_sched.h>

#define BAR_0 0
#define BAR_1 1

#define INTEL_E1000_ETHERNET_DEVICE(device_id)     \
    {                                              \
        PCI_DEVICE(PCI_VENDOR_ID_INTEL, device_id) \
    }

extern char e1000_driver_name[];
// #include "e1000_hw.h"

/* board specific private data structure */

struct e1000_adapter {
    size_t ptr;
    int bars;
    struct napi_struct napi;
    u32 msg_enable;
    u32 bd_number;
    struct pci_dev* pdev;

};

#define E1000_MAX_INTR 10

/*
 * Count for polling __E1000_RESET condition every 10-20msec.
 */
#define E1000_CHECK_RESET_COUNT 50

/* TX/RX descriptor defines */
#define E1000_DEFAULT_TXD 256
#define E1000_MAX_TXD 256
#define E1000_MIN_TXD 48
#define E1000_MAX_82544_TXD 4096

#define E1000_DEFAULT_RXD 256
#define E1000_MAX_RXD 256
#define E1000_MIN_RXD 48
#define E1000_MAX_82544_RXD 4096

#define E1000_MIN_ITR_USECS 10 /* 100000 irq/sec */
#define E1000_MAX_ITR_USECS 10000 /* 100    irq/sec */

/* this is the size past which hardware will drop packets when setting LPE=0 */
#define MAXIMUM_ETHERNET_VLAN_SIZE 1522

/* Supported Rx Buffer Sizes */
#define E1000_RXBUFFER_128 128 /* Used for packet split */
#define E1000_RXBUFFER_256 256 /* Used for packet split */
#define E1000_RXBUFFER_512 512
#define E1000_RXBUFFER_1024 1024
#define E1000_RXBUFFER_2048 2048
#define E1000_RXBUFFER_4096 4096
#define E1000_RXBUFFER_8192 8192
#define E1000_RXBUFFER_16384 16384

/* SmartSpeed delimiters */
#define E1000_SMARTSPEED_DOWNSHIFT 3
#define E1000_SMARTSPEED_MAX 15

/* Packet Buffer allocations */
#define E1000_PBA_BYTES_SHIFT 0xA
#define E1000_TX_HEAD_ADDR_SHIFT 7
#define E1000_PBA_TX_MASK 0xFFFF0000

/* Flow Control Watermarks */
#define E1000_FC_HIGH_DIFF 0x1638 /* High: 5688 bytes below Rx FIFO size */
#define E1000_FC_LOW_DIFF 0x1640 /* Low:  5696 bytes below Rx FIFO size */

#define E1000_FC_PAUSE_TIME 0xFFFF /* pause for the max or until send xon */

/* How many Tx Descriptors do we need to call netif_wake_queue ? */
#define E1000_TX_QUEUE_WAKE 16
/* How many Rx Buffers do we bundle into one write to the hardware ? */
#define E1000_RX_BUFFER_WRITE 16 /* Must be power of 2 */

#define AUTO_ALL_MODES 0
#define E1000_EEPROM_82544_APM 0x0004
#define E1000_EEPROM_APME 0x0400

#ifndef E1000_MASTER_SLAVE
/* Switch to override PHY master/slave setting */
#define E1000_MASTER_SLAVE e1000_ms_hw_default
#endif

#define E1000_MNG_VLAN_NONE (-1)

void e1000e_set_ethtool_ops(struct net_device* netdev);
size_t e1000e_get_e1000(struct net_device* netdev);
#endif /* _E1000_H_ */