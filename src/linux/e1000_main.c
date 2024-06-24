#include "e1000.h"
#include "linux/etherdevice.h"
#include "linux/gfp_types.h"
#include "linux/irqreturn.h"
#include "linux/printk.h"
#include "linux/skbuff.h"
#include "linux/stddef.h"
#include <linux/cpu.h>
#include <linux/delay.h>
#include <linux/dma-mapping.h>
#include <linux/ethtool.h>
#include <linux/if_vlan.h>
#include <linux/init.h>
#include <linux/interrupt.h>
#include <linux/ipv6.h>
#include <linux/module.h>
#include <linux/netdevice.h>
#include <linux/pagemap.h>
#include <linux/pci.h>
#include <linux/pm_qos.h>
#include <linux/pm_runtime.h>
#include <linux/prefetch.h>
#include <linux/slab.h>
#include <linux/smp.h>
#include <linux/suspend.h>
#include <linux/tcp.h>
#include <linux/types.h>
#include <linux/vmalloc.h>
#include <net/checksum.h>
#include <net/ip6_checksum.h>

char e1000_driver_name[] = "e1000_rs";

#define DEFAULT_MSG_ENABLE (NETIF_MSG_DRV | NETIF_MSG_PROBE | NETIF_MSG_LINK)
static int debug = -1;
module_param(debug, int, 0);
MODULE_PARM_DESC(debug, "Debug level (0=none,...,16=all)");

static char e1000_rs_driver_string[] =
    "Intel(R) PRO/1000 Network Driver (Rust)";
static const char e1000_rs_copyright[] = "Copyright (c) 2023-2024 QCL";

static int e1000_init_module(void);
static void e1000_exit_module(void);
static int e1000_probe(struct pci_dev *pdev, const struct pci_device_id *ent);
static void e1000_remove(struct pci_dev *pdev);
static int __maybe_unused e1000_suspend(struct device *dev);
static int __maybe_unused e1000_resume(struct device *dev);
static void e1000_shutdown(struct pci_dev *pdev);

#define COPYBREAK_DEFAULT 256
static unsigned int copybreak __read_mostly = COPYBREAK_DEFAULT;
module_param(copybreak, uint, 0644);
MODULE_PARM_DESC(
    copybreak,
    "Maximum size of packet that is copied to a new buffer on receive");

static pci_ers_result_t e1000_io_error_detected(struct pci_dev *pdev,
                                                pci_channel_state_t state);
static pci_ers_result_t e1000_io_slot_reset(struct pci_dev *pdev);
void e1000e_set_interrupt_capability(struct e1000_adapter *adapter);
void e1000e_reset_interrupt_capability(struct e1000_adapter *adapter);
static int e1000_request_msix(struct e1000_adapter *adapter);

static void cb_on_xmit_completed(const void *data, uint32_t pkts,
                                 uint32_t bytes) {

  struct e1000_adapter *adapter = (struct e1000_adapter *)data;
  netdev_completed_queue(adapter->netdev, pkts, bytes);
}
/**
 * e1000_watchdog - Timer Call-back
 * @t: pointer to timer_list containing private info adapter
 **/
static void e1000_watchdog(struct timer_list *t) {
  struct e1000_adapter *adapter = from_timer(adapter, t, watchdog_timer);

  /* Do the rest outside of interrupt context */
  schedule_work(&adapter->watchdog_task);

  /* TODO: make this use queue_delayed_work() */
}
static irqreturn_t e1000_intr_msix_rx(int __always_unused irq, void *data) {
  struct net_device *netdev = data;
  size_t e1000 = e1000e_get_e1000(netdev);
  struct e1000_adapter *adapter = netdev_priv(netdev);

  if (napi_schedule_prep(&adapter->napi)) {
    struct E1000Status *status = e1000r_status(e1000);
    status->total_rx_bytes = 0;
    status->total_rx_packets = 0;
    __napi_schedule(&adapter->napi);
  }
  return IRQ_HANDLED;
}

static void e1000_watchdog_task(struct work_struct *work) {

  struct e1000_adapter *adapter =
      container_of(work, struct e1000_adapter, watchdog_task);
  struct net_device *netdev = adapter->netdev;

  if ((netif_carrier_ok(netdev))) {
    /* Cancel scheduled suspend requests. */
    pm_runtime_resume(netdev->dev.parent);
    goto link_up;
  }

link_up:

  e1000r_task_watchdog(adapter->ptr);
  /* Reset the timer */
  if (!adapter->is_down)
    mod_timer(&adapter->watchdog_timer, round_jiffies(jiffies + 2 * HZ));
}

static irqreturn_t e1000_intr_msix_tx(int __always_unused irq, void *data) {
  struct net_device *netdev = data;
  size_t e1000 = e1000e_get_e1000(netdev);
  e1000r_irq_handle_msix_tx(e1000, irq);
  return IRQ_HANDLED;
}
static irqreturn_t e1000_msix_other(int __always_unused irq, void *data) {
  struct net_device *netdev = data;
  size_t e1000 = e1000e_get_e1000(netdev);
  if (e1000r_irq_handle(e1000, irq)) {
    return IRQ_NONE;
  }
  // struct e1000_adapter *adapter = netdev_priv(netdev);
  // struct e1000_hw *hw = &adapter->hw;
  // u32 icr = er32(ICR);

  // if (icr & adapter->eiac_mask)
  // 	ew32(ICS, (icr & adapter->eiac_mask));

  // if (icr & E1000_ICR_LSC) {
  // 	hw->mac.get_link_status = true;
  // 	/* guard against interrupt when we're going down */
  // 	if (!test_bit(__E1000_DOWN, &adapter->state))
  // 		mod_timer(&adapter->watchdog_timer, jiffies + 1);
  // }

  // if (!test_bit(__E1000_DOWN, &adapter->state))
  // 	ew32(IMS, E1000_IMS_OTHER | IMS_OTHER_MASK);

  return IRQ_HANDLED;
}
/**
 * e1000_intr_msi - Interrupt Handler
 * @irq: interrupt number
 * @data: pointer to a network interface device structure
 **/
static irqreturn_t e1000_intr_msi(int __always_unused irq, void *data) {
  struct net_device *netdev = data;
  struct e1000_adapter *adapter = netdev_priv(netdev);
  size_t e1000 = e1000e_get_e1000(netdev);
  struct E1000Status *status = e1000r_status(e1000);
  if (e1000r_irq_handle(e1000, irq)) {
    return IRQ_NONE;
  }
  // mod_timer(&adapter->watchdog_timer, jiffies + 2 );

  if (napi_schedule_prep(&adapter->napi)) {
    status->total_tx_bytes = 0;
    status->total_tx_packets = 0;
    status->total_rx_bytes = 0;
    status->total_rx_packets = 0;
    __napi_schedule(&adapter->napi);
  }
  return IRQ_HANDLED;
}

/**
 * e1000_request_irq - initialize interrupts
 * @adapter: board private structure
 *
 * Attempts to configure interrupts using the best available
 * capabilities of the hardware and kernel.
 **/
static int e1000_request_irq(struct e1000_adapter *adapter) {
  struct net_device *netdev = adapter->netdev;
  size_t e1000 = e1000e_get_e1000(adapter->netdev);
  struct E1000Status *status = e1000r_status(e1000);
  struct e1000r_info_all info = e1000r_get_info_all(e1000);
  struct Flags flags = info.flags;
  int err;

  if (adapter->msix_entries) {
    err = e1000_request_msix(adapter);
    if (!err)
      return err;
    /* fall back to MSI */
    e1000e_reset_interrupt_capability(adapter);
    status->int_mode = E1000E_INT_MODE_MSI;
    e1000e_set_interrupt_capability(adapter);
  }
  if (flags.msi_enabled) {
    err = request_irq(adapter->pdev->irq, e1000_intr_msi, 0, netdev->name,
                      netdev);
    if (!err)
      return err;

    /* fall back to legacy interrupt */
    e1000e_reset_interrupt_capability(adapter);
    status->int_mode = E1000E_INT_MODE_LEGACY;
  }

  err = request_irq(adapter->pdev->irq, e1000_intr_msi, IRQF_SHARED,
                    netdev->name, netdev);
  if (err)
    e_err("Unable to allocate interrupt, Error: %d\n", err);

  return err;
}

/**
 * e1000_request_msix - Initialize MSI-X interrupts
 * @adapter: board private structure
 *
 * e1000_request_msix allocates MSI-X vectors and requests interrupts from the
 * kernel.
 **/
static int e1000_request_msix(struct e1000_adapter *adapter) {
  struct net_device *netdev = adapter->netdev;
  size_t e1000 = e1000e_get_e1000(adapter->netdev);

  int err = 0, vector = 0;

  err = request_irq(adapter->msix_entries[vector].vector, e1000_intr_msix_rx, 0,
                    "E1000R_RX", netdev);
  if (err)
    return err;

  vector++;
  err = request_irq(adapter->msix_entries[vector].vector, e1000_intr_msix_tx, 0,
                    "E1000R_TX", netdev);
  if (err)
    return err;

  vector++;

  err = request_irq(adapter->msix_entries[vector].vector, e1000_msix_other, 0,
                    netdev->name, netdev);
  if (err)
    return err;

  e1000r_configure_msix(e1000);
  return 0;
}
static void e1000_io_resume(struct pci_dev *pdev);

/**
 * e1000_remove - Device Removal Routine
 * @pdev: PCI device information struct
 *
 * e1000_remove is called by the PCI subsystem to alert the driver
 * that it should release a PCI device. That could be caused by a
 * Hot-Plug event, or because the driver is going to be removed from
 * memory.
 **/
static void e1000_remove(struct pci_dev *pdev) {
  struct net_device *netdev = pci_get_drvdata(pdev);
  struct e1000_adapter *adapter = netdev_priv(netdev);

  e1000r_drop(adapter->ptr);

  unregister_netdev(netdev);
  pci_release_selected_regions(pdev, adapter->bars);
}
static int __maybe_unused e1000_suspend(struct device *dev) { return 0; }

static int __maybe_unused e1000_resume(struct device *dev) {
  pr_info("e1000_resume");
  return 0;
}

static void e1000_shutdown(struct pci_dev *pdev) {}

/**
 * e1000e_open - Called when a network interface is made active
 * @netdev: network interface device structure
 *
 * Returns 0 on success, negative value on failure
 *
 * The open entry point is called when a network interface is made
 * active by the system (IFF_UP).  At this point all resources needed
 * for transmit and receive operations are allocated, the interrupt
 * handler is registered with the OS, the watchdog timer is started,
 * and the stack is notified that the interface is ready.
 **/
int e1000e_open(struct net_device *netdev) {

  struct e1000_adapter *adapter = netdev_priv(netdev);
  size_t e1000 = adapter->ptr;
  struct pci_dev *pdev = adapter->pdev;

  pm_runtime_get_sync(&pdev->dev);

  netif_carrier_off(netdev);
  netif_stop_queue(netdev);

  pr_info("e1000e open");
  size_t err = e1000r_open(e1000);
  if (err != 0) {
    pr_err("e1000e open fail: %d", err);
    goto err_req_irq;
  }

  err = e1000_request_irq(adapter);
  if (err)
    goto err_req_irq;

  napi_enable(&adapter->napi);

  e1000r_irq_enable(e1000);

  pm_runtime_put(&pdev->dev);

  e1000r_trigger_lsc(e1000);

  // netif_start_queue(netdev);

  // netif_carrier_on(netdev);

  pr_info("e1000e open ok");
  mod_timer(&adapter->watchdog_timer, round_jiffies(jiffies + 1));
  return 0;
err_req_irq:

  pm_runtime_put_sync(&pdev->dev);

  return err;
}
/**
 * e1000e_close - Disables a network interface
 * @netdev: network interface device structure
 *
 * Returns 0, this is not allowed to fail
 *
 * The close entry point is called when an interface is de-activated
 * by the OS.  The hardware is still under the drivers control, but
 * needs to be disabled.  A global MAC reset is issued to stop the
 * hardware, and all transmit and receive resources are freed.
 **/
int e1000e_close(struct net_device *netdev) {
  pr_info("e1000e_close");
  return -1;
}

/**
 * e1000_io_error_detected - called when PCI error is detected
 * @pdev: Pointer to PCI device
 * @state: The current pci connection state
 *
 * This function is called after a PCI bus error affecting
 * this device has been detected.
 */
static pci_ers_result_t e1000_io_error_detected(struct pci_dev *pdev,
                                                pci_channel_state_t state) {
  /* Request a slot reset. */
  return PCI_ERS_RESULT_NEED_RESET;
}

/**
 * e1000_io_slot_reset - called after the pci bus has been reset.
 * @pdev: Pointer to PCI device
 *
 * Restart the card from scratch, as if from a cold-boot. Implementation
 * resembles the first-half of the e1000_resume routine.
 */
static pci_ers_result_t e1000_io_slot_reset(struct pci_dev *pdev) {
  return PCI_ERS_RESULT_RECOVERED;
}

/**
 * e1000_io_resume - called when traffic can start flowing again.
 * @pdev: Pointer to PCI device
 *
 * This callback is called when the error recovery driver tells us that
 * its OK to resume normal operation. Implementation resembles the
 * second-half of the e1000_resume routine.
 */
static void e1000_io_resume(struct pci_dev *pdev) {}
static netdev_tx_t e1000_xmit_frame(struct sk_buff *skb,
                                    struct net_device *netdev) {
  struct e1000_adapter *adapter = netdev_priv(netdev);

  // unsigned int first;
  size_t e1000 = adapter->ptr;
  unsigned int len = skb_headlen(skb);
  unsigned int nr_frags;
  unsigned int mss;
  int count = 0;
  // int tso;
  // unsigned int tx_flags = 0;
  unsigned int f;
  __be16 protocol = vlan_get_protocol(skb);
  struct E1000Status *status = e1000r_status(e1000);

  if (skb->len <= 0) {
    dev_kfree_skb_any(skb);
    return NETDEV_TX_OK;
  }

  /* The minimum packet size with TCTL.PSP set is 17 bytes so
   * pad skb in order to meet this minimum size requirement
   */
  if (skb_put_padto(skb, 17))
    return NETDEV_TX_OK;

  mss = skb_shinfo(skb)->gso_size;
  if (mss) {
    u8 hdr_len;

    /* TSO Workaround for 82571/2/3 Controllers -- if skb->data
     * points to just header, pull a few bytes of payload from
     * frags into skb->data
     */
    hdr_len = skb_tcp_all_headers(skb);
    /* we do this workaround for ES2LAN, but it is un-necessary,
     * avoiding it could save a lot of cycles
     */
    if (skb->data_len && (hdr_len == len)) {
      unsigned int pull_size;

      pull_size = min_t(unsigned int, 4, skb->data_len);
      if (!__pskb_pull_tail(skb, pull_size)) {
        e_err("__pskb_pull_tail failed.\n");
        dev_kfree_skb_any(skb);
        return NETDEV_TX_OK;
      }
      len = skb_headlen(skb);
    }
  }

  /* reserve a descriptor for the offload context */
  if ((mss) || (skb->ip_summed == CHECKSUM_PARTIAL))
    count++;
  count++;

  count += DIV_ROUND_UP(len, status->tx_fifo_limit);

  nr_frags = skb_shinfo(skb)->nr_frags;

  for (f = 0; f < nr_frags; f++)
    count += DIV_ROUND_UP(skb_frag_size(&skb_shinfo(skb)->frags[f]),
                          status->tx_fifo_limit);

  struct E1000XmitConfig config = {
      .timestamp = jiffies,
      .segs = 1,
      .ipv4 = protocol == htons(ETH_P_IP),
      .no_fcs = unlikely(skb->no_fcs),
      .vlan_tag_present = skb_vlan_tag_present(skb),
  };

  // if (adapter->hw.mac.tx_pkt_filtering)
  // e1000_transfer_dhcp_info(adapter, skb);

  skb_tx_timestamp(skb);
  netdev_sent_queue(netdev, skb->len);

  int32_t n = e1000r_transmit(e1000, config, skb->data, len);
  if (n < 0) {
    return NETDEV_TX_BUSY;
  }
  napi_consume_skb(skb, 64);
  return NETDEV_TX_OK;
}
/**
 * e1000e_get_stats64 - Get System Network Statistics
 * @netdev: network interface device structure
 * @stats: rtnl_link_stats64 pointer
 *
 * Returns the address of the device statistics structure.
 **/
void e1000e_get_stats64(struct net_device *netdev,
                        struct rtnl_link_stats64 *stats) {
  struct e1000_adapter *adapter = netdev_priv(netdev);

  spin_lock(&adapter->stats64_lock);

  struct E1000HWStats st = e1000r_stats64(adapter->ptr);

  /* Fill out the OS statistics structure */
  stats->rx_bytes = st.gorc;
  stats->rx_packets = st.gprc;
  stats->tx_bytes = st.gotc;
  stats->tx_packets = st.gptc;
  stats->multicast = st.mprc;
  stats->collisions = st.colc;

  /* Rx Errors */

  /* RLEC on some newer hardware can be incorrect so build
   * our own version based on RUC and ROC
   */
  stats->rx_errors =
      st.rxerrc + st.crcerrs + st.algnerrc + st.ruc + st.roc + st.cexterr;
  stats->rx_length_errors = st.ruc + st.roc;
  stats->rx_crc_errors = st.crcerrs;
  stats->rx_frame_errors = st.algnerrc;
  stats->rx_missed_errors = st.mpc;

  /* Tx Errors */
  stats->tx_errors = st.ecol + st.latecol;
  stats->tx_aborted_errors = st.ecol;
  stats->tx_window_errors = st.latecol;
  stats->tx_carrier_errors = st.tncrs;

  /* Tx Dropped needs to be maintained elsewhere */

  spin_unlock(&adapter->stats64_lock);
}

/**
 * e1000e_set_rx_mode - secondary unicast, Multicast and Promiscuous mode set
 * @netdev: network interface device structure
 *
 * The ndo_set_rx_mode entry point is called whenever the unicast or multicast
 * address list or the network interface flags are updated.  This routine is
 * responsible for configuring the hardware for proper unicast, multicast,
 * promiscuous mode, and all-multi behavior.
 **/
static void e1000e_set_rx_mode(struct net_device *netdev) {
  // pr_info("e1000e_set_rx_mode");
}
/**
 * e1000_set_mac - Change the Ethernet Address of the NIC
 * @netdev: network interface device structure
 * @p: pointer to an address structure
 *
 * Returns 0 on success, negative on failure
 **/
static int e1000_set_mac(struct net_device *netdev, void *p) { return -1; }
/**
 * e1000_change_mtu - Change the Maximum Transfer Unit
 * @netdev: network interface device structure
 * @new_mtu: new value for maximum frame size
 *
 * Returns 0 on success, negative on failure
 **/
static int e1000_change_mtu(struct net_device *netdev, int new_mtu) {

  pr_info("e1000_change_mtu");
  return -1;
}
static int e1000_ioctl(struct net_device *netdev, struct ifreq *ifr, int cmd) {
  return -1;
}
/**
 * e1000_tx_timeout - Respond to a Tx Hang
 * @netdev: network interface device structure
 * @txqueue: index of the hung queue (unused)
 **/
static void e1000_tx_timeout(struct net_device *netdev,
                             unsigned int __always_unused txqueue) {}
static int e1000_vlan_rx_add_vid(struct net_device *netdev,
                                 __always_unused __be16 proto, u16 vid) {
  return -1;
}
static int e1000_vlan_rx_kill_vid(struct net_device *netdev,
                                  __always_unused __be16 proto, u16 vid) {
  return -1;
}

static irqreturn_t e1000_intr_msix(int __always_unused irq, void *data) {
  struct net_device *netdev = data;
  struct e1000_adapter *adapter = netdev_priv(netdev);

  if (adapter->msix_entries) {
    int vector, msix_irq;

    vector = 0;
    msix_irq = adapter->msix_entries[vector].vector;
    if (disable_hardirq(msix_irq))
      e1000_intr_msix_rx(msix_irq, netdev);
    enable_irq(msix_irq);

    vector++;
    msix_irq = adapter->msix_entries[vector].vector;
    if (disable_hardirq(msix_irq))
      e1000_intr_msix_tx(msix_irq, netdev);
    enable_irq(msix_irq);

    vector++;
    msix_irq = adapter->msix_entries[vector].vector;
    if (disable_hardirq(msix_irq))
      e1000_msix_other(msix_irq, netdev);
    enable_irq(msix_irq);
  }

  return IRQ_HANDLED;
}

/**
 * e1000_netpoll
 * @netdev: network interface device structure
 *
 * Polling 'interrupt' - used by things like netconsole to send skbs
 * without having to re-enable interrupts. It's not called while
 * the interrupt routine is executing.
 */
static void e1000_netpoll(struct net_device *netdev) {

  struct e1000_adapter *adapter = netdev_priv(netdev);
  e1000_intr_msix(adapter->pdev->irq, netdev);
}

void cb_on_rcv_pkt(const void *pdev, struct E1000RcvPkt pkt) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)pdev;
  struct sk_buff *skb = napi_alloc_skb(&adapter->napi, pkt.length);
  skb_copy_to_linear_data_offset(skb, -NET_IP_ALIGN, (pkt.data - NET_IP_ALIGN),
                                 (pkt.length + NET_IP_ALIGN));
  /* end copybreak code */
  skb_put(skb, pkt.length);
  skb->protocol = eth_type_trans(skb, adapter->netdev);
  // skb_checksum_none_assert(skb);
  napi_gro_receive(&adapter->napi, skb);
}
void cb_enbale_net(const void *pdev) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)pdev;
  netif_start_queue(adapter->netdev);

  netif_carrier_on(adapter->netdev);
}

/**
 * e1000e_poll - NAPI Rx polling callback
 * @napi: struct associated with this polling callback
 * @budget: number of packets driver is allowed to process this poll
 **/
static int e1000e_poll(struct napi_struct *napi, int budget) {
  struct e1000_adapter *adapter =
      container_of(napi, struct e1000_adapter, napi);

  bool tx_cleand = false;

  if (!adapter->msix_entries) {
    tx_cleand = e1000r_irq_clean_tx(adapter->ptr);
  }

  int work_done = e1000r_clear_rx(adapter->ptr, budget, cb_on_rcv_pkt, adapter);

  if (!tx_cleand || work_done == budget)
    return budget;

  /* Exit the polling mode, but don't re-enable interrupts if stack might
   * poll us due to busy-polling
   */
  if (likely(napi_complete_done(napi, work_done))) {
    e1000r_irq_enable(adapter->ptr);
    //   if (adapter->itr_setting & 3)
    //     e1000_set_itr(adapter);
    //   if (!test_bit(__E1000_DOWN, &adapter->state)) {
    //     if (adapter->msix_entries)
    //       ew32(IMS, adapter->rx_ring->ims_val);
    //     else
    //       e1000_irq_enable(adapter);
    //   }
  }

  return work_done;
}
static int e1000_set_features(struct net_device *netdev,
                              netdev_features_t features) {
  pr_info("e1000_set_features");
  return -1;
}
static netdev_features_t e1000_fix_features(struct net_device *netdev,
                                            netdev_features_t features) {

  pr_info("e1000_fix_features");

  /* Jumbo frame workaround on 82579 and newer requires CRC be stripped */
  if (netdev->mtu > ETH_DATA_LEN)
    features &= ~NETIF_F_RXFCS;

  /* Since there is no support for separate Rx/Tx vlan accel
   * enable/disable make sure Tx flag is always in same state as Rx.
   */
  if (features & NETIF_F_HW_VLAN_CTAG_RX)
    features |= NETIF_F_HW_VLAN_CTAG_TX;
  else
    features &= ~NETIF_F_HW_VLAN_CTAG_TX;

  return features;
}
void e1000e_reset_interrupt_capability(struct e1000_adapter *adapter) {
  size_t e1000 = e1000e_get_e1000(adapter->netdev);
  struct e1000r_info_all info = e1000r_get_info_all(e1000);
  struct E1000Status *status = e1000r_status(e1000);
  struct Flags flags = info.flags;
  if (adapter->msix_entries) {
    pci_disable_msix(adapter->pdev);
    status->msix_vec_num = 0;
    kfree(adapter->msix_entries);
    adapter->msix_entries = NULL;
  } else if (flags.msi_enabled) {
    pci_disable_msi(adapter->pdev);
    flags.msi_enabled = false;
    e1000r_set_flags(e1000, flags);
  }
}
/**
 * e1000e_set_interrupt_capability - set MSI or MSI-X if supported
 * @adapter: board private structure
 *
 * Attempt to configure interrupts using the best available
 * capabilities of the hardware and kernel.
 **/
void e1000e_set_interrupt_capability(struct e1000_adapter *adapter) {
  int err;
  int i;
  size_t e1000 = e1000e_get_e1000(adapter->netdev);
  struct e1000r_info_all info = e1000r_get_info_all(e1000);
  struct Flags flags = info.flags;
  struct E1000Status *status = e1000r_status(e1000);

  switch (status->int_mode) {
  case E1000E_INT_MODE_MSIX:
    if (flags.has_msix) {
      status->num_vectors = 3; /* RxQ0, TxQ0 and other */
      status->msix_vec_num = status->num_vectors;
      adapter->msix_entries =
          kcalloc(status->num_vectors, sizeof(struct msix_entry), GFP_KERNEL);
      if (adapter->msix_entries) {
        struct e1000_adapter *a = adapter;

        for (i = 0; i < status->num_vectors; i++)
          adapter->msix_entries[i].entry = i;

        err = pci_enable_msix_range(a->pdev, a->msix_entries,
                                    status->num_vectors, status->num_vectors);
        if (err > 0)
          return;
      }
      /* MSI-X failed, so fall through and try MSI */
      e_err("Failed to initialize MSI-X interrupts.  Falling back to MSI "
            "interrupts.\n");
      e1000e_reset_interrupt_capability(adapter);
    }
    status->int_mode = E1000E_INT_MODE_MSI;
    fallthrough;
  case E1000E_INT_MODE_MSI:
    if (!pci_enable_msi(adapter->pdev)) {
      flags.msi_enabled = true;
      e1000r_set_flags(e1000, flags);
    } else {
      status->int_mode = E1000E_INT_MODE_LEGACY;
      e_err("Failed to initialize MSI interrupts.  Falling back to legacy "
            "interrupts.\n");
    }
    fallthrough;
  case E1000E_INT_MODE_LEGACY:
    /* Don't do anything; this is the system default */
    break;
  }

  /* store the number of vectors being used */
  status->num_vectors = 1;
}
/**
 * e1000_irq_disable - Mask off interrupt generation on the NIC
 * @adapter: board private structure
 **/
static void e1000_irq_disable(struct e1000_adapter *adapter) {
  size_t e1000 = e1000e_get_e1000(adapter->netdev);
  e1000r_irq_disable(e1000);
  struct E1000Status *status = e1000r_status(e1000);

  if (adapter->msix_entries) {
    int i;

    for (i = 0; i < status->num_vectors; i++)
      synchronize_irq(adapter->msix_entries[i].vector);
  } else {
    synchronize_irq(adapter->pdev->irq);
  }
}
static const struct net_device_ops e1000e_netdev_ops = {
    .ndo_open = e1000e_open,
    .ndo_stop = e1000e_close,
    .ndo_start_xmit = e1000_xmit_frame,
    .ndo_get_stats64 = e1000e_get_stats64,
    .ndo_set_rx_mode = e1000e_set_rx_mode,
    .ndo_set_mac_address = e1000_set_mac,
    .ndo_change_mtu = e1000_change_mtu,
    .ndo_eth_ioctl = e1000_ioctl,
    .ndo_tx_timeout = e1000_tx_timeout,
    .ndo_validate_addr = eth_validate_addr,

    .ndo_vlan_rx_add_vid = e1000_vlan_rx_add_vid,
    .ndo_vlan_rx_kill_vid = e1000_vlan_rx_kill_vid,
#ifdef CONFIG_NET_POLL_CONTROLLER
    .ndo_poll_controller = e1000_netpoll,
#endif
    .ndo_set_features = e1000_set_features,
    .ndo_fix_features = e1000_fix_features,
    .ndo_features_check = passthru_features_check,
};

size_t e1000_get_e1000r(struct net_device *netdev) {
  struct e1000_adapter *adapter = netdev_priv(netdev);

  return adapter->ptr;
}

struct E1000Status *e1000_status(struct net_device *netdev) {
  return e1000r_status(e1000_get_e1000r(netdev));
}

size_t cb_bar_n_remap(const void *data, size_t bar) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)data;
  struct pci_dev *pdev = adapter->pdev;
  resource_size_t mmio_start = pci_resource_start(pdev, bar);
  resource_size_t mmio_len = pci_resource_len(pdev, bar);
  return (size_t)ioremap(mmio_start, mmio_len);
}

struct E1000rPcieInfo cb_pcie_info(const void *data) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)data;

  u16 bus_width = 0;

  u16 pcie_link_status, cap_offset;
  cap_offset = adapter->pdev->pcie_cap;

  if (!cap_offset) {
    bus_width = 0;
  } else {
    pci_read_config_word(adapter->pdev, cap_offset + PCIE_LINK_STATUS,
                         &pcie_link_status);
    bus_width =
        ((pcie_link_status & PCIE_LINK_WIDTH_MASK) >> PCIE_LINK_WIDTH_SHIFT);
  }

  struct E1000rPcieInfo info = {
      .bus_link_width = bus_width,
  };

  return info;
}

void cb_io_unmap(size_t addr) { iounmap((void *)addr); }

bool cb_mtu_bg_eth(const void *data) {

  struct e1000_adapter *adapter = (struct e1000_adapter *)data;

  return adapter->netdev->mtu > ETH_DATA_LEN;
}

size_t cb_dma_alloc(const void *data, size_t pages, u64 *dma) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)data;
  struct pci_dev *pdev = adapter->pdev;

  return (size_t)dma_alloc_coherent(&pdev->dev, pages * 4096, dma, GFP_KERNEL);
}

void cb_dma_free(const void *data, size_t cpu_addr, u64 dma, size_t pages) {
  struct e1000_adapter *adapter = (struct e1000_adapter *)data;
  struct pci_dev *pdev = adapter->pdev;

  dma_free_coherent(&pdev->dev, pages * 4096, (void *)cpu_addr, dma);
  return;
}

size_t e1000e_get_e1000(struct net_device *netdev) {
  struct e1000_adapter *adapter = netdev_priv(netdev);
  return adapter->ptr;
}

u16 cb_pci_read_config_word(const void *data, int32_t where) {

  struct e1000_adapter *adapter = (struct e1000_adapter *)data;
  u16 val;
  pci_read_config_word(adapter->pdev, where, &val);

  return val;
}

void cb_udelay(uint32_t usecs) { udelay(usecs); }

/**
 * e1000_sw_init - Initialize general software structures (struct e1000_adapter)
 * @adapter: board private structure to initialize
 *
 * e1000_sw_init initializes the Adapter private data structure.
 * Fields are initialized based on PCI device information and
 * OS network device settings (MTU size).
 **/
static int e1000_sw_init(struct e1000_adapter *adapter) {
  spin_lock_init(&adapter->stats64_lock);
  spin_lock_init(&adapter->systim_lock);

  e1000e_set_interrupt_capability(adapter);
  /* Explicitly disable IRQ since the NIC can be in any state. */
  e1000_irq_disable(adapter);
  return 0;
}

/**
 * e1000_probe - Device Initialization Routine
 * @pdev: PCI device information struct
 * @ent: entry in e1000_pci_tbl
 *
 * Returns 0 on success, negative on failure
 *
 * e1000_probe initializes an adapter identified by a pci_dev structure.
 * The OS initialization, configuring of the adapter private structure,
 * and a hardware reset occur.
 **/
static int e1000_probe(struct pci_dev *pdev, const struct pci_device_id *ent) {
  int err;
  struct net_device *netdev;
  struct e1000_adapter *adapter = NULL;
  static int cards_found;
  //   void *hw_addr = NULL;
  //   void *flash_addr = NULL;
  resource_size_t mmio_start, mmio_len;
  //   resource_size_t flash_start, flash_len;
  int bars;

  err = pci_enable_device_mem(pdev);
  if (err)
    return err;
  err = dma_set_mask_and_coherent(&pdev->dev, DMA_BIT_MASK(64));
  if (err) {
    dev_err(&pdev->dev, "No usable DMA configuration, aborting\n");
    goto err_dma;
  }

  bars = pci_select_bars(pdev, IORESOURCE_MEM);

  err = pci_request_selected_regions_exclusive(pdev, bars, e1000_driver_name);
  if (err)
    goto err_pci_reg;

  pci_set_master(pdev);
  err = pci_save_state(pdev);
  if (err)
    goto err_alloc_etherdev;

  err = -ENOMEM;

  netdev = alloc_etherdev(sizeof(struct e1000_adapter));
  if (!netdev)
    goto err_alloc_etherdev;

  SET_NETDEV_DEV(netdev, &pdev->dev);
  netdev->irq = pdev->irq;

  pci_set_drvdata(pdev, netdev);

  mmio_start = pci_resource_start(pdev, 0);
  mmio_len = pci_resource_len(pdev, 0);

  adapter = netdev_priv(netdev);
  adapter->is_down = false;
  adapter->bars = bars;
  adapter->pdev = pdev;
  adapter->netdev = netdev;

  E1000Config config;
  config.adapter = adapter;
  config.device_id = ent->device;
  config.cb_iounmap = cb_io_unmap;
  config.cb_bar_n_remap = cb_bar_n_remap;
  config.cb_pcie_info = cb_pcie_info;
  config.cb_mtu_bg_eth = cb_mtu_bg_eth;
  config.cb_dma_alloc_coherent = cb_dma_alloc;
  config.cb_dma_free_coherent = cb_dma_free;
  config.cb_pci_read_config_word = cb_pci_read_config_word;
  config.cb_udelay = cb_udelay;
  config.cb_on_xmit_completed = cb_on_xmit_completed;
  config.cb_enable_net = cb_enbale_net;

  size_t e1000 = e1000r_new(config);
  if (!e1000)
    goto err_e1000r;

  adapter->ptr = e1000;
  adapter->msg_enable = netif_msg_init(debug, DEFAULT_MSG_ENABLE);

  struct e1000r_info_all info = e1000r_get_info_all(e1000);
  struct Flags flags = info.flags;
  struct Flags2 flags2 = info.flags2;

  /* Set default EEE advertisement */
  if (flags2.has_eee)
    adapter->eee_advert = MDIO_EEE_100TX | MDIO_EEE_1000T;

  netdev->netdev_ops = &e1000e_netdev_ops;
  // e1000e_set_ethtool_ops(netdev);

  netdev->watchdog_timeo = 5 * HZ;
  netif_napi_add(netdev, &adapter->napi, e1000e_poll);
  strscpy(netdev->name, pci_name(pdev), sizeof(netdev->name));

  netdev->mem_start = mmio_start;
  netdev->mem_end = mmio_start + mmio_len;
  adapter->bd_number = cards_found++;
  u8 mac_addr[6];

  err = e1000r_init(e1000);
  if (err != 0)
    goto err_eeprom;

  err = e1000_sw_init(adapter);
  if (err)
    goto err_sw_init;

  struct E1000Status *status = e1000r_status(e1000);

  /* Set initial default active device features */
  netdev->features = (NETIF_F_SG | NETIF_F_HW_CSUM);

  // netdev->features = (NETIF_F_SG | NETIF_F_HW_VLAN_CTAG_RX |
  //                     NETIF_F_HW_VLAN_CTAG_TX | NETIF_F_TSO | NETIF_F_TSO6 |
  //                     NETIF_F_RXHASH | NETIF_F_RXCSUM | NETIF_F_HW_CSUM);

  /* disable TSO for pcie and 10/100 speeds to avoid
   * some hardware issues and for i219 to fix transfer
   * speed being capped at 60%
   */
  // if (!flags.tso_force) {
  //   switch (status->link_speed) {
  //   case SPEED_10:
  //   case SPEED_100:
  //     e_info("10/100 speed: disabling TSO\n");
  //     netdev->features &= ~NETIF_F_TSO;
  //     netdev->features &= ~NETIF_F_TSO6;
  //     break;
  //   case SPEED_1000:
  //     netdev->features |= NETIF_F_TSO;
  //     netdev->features |= NETIF_F_TSO6;
  //     break;
  //   default:
  //     /* oops */
  //     break;
  //   }
  //   if (info.mac.kind == e1000_pch_spt) {
  //     netdev->features &= ~NETIF_F_TSO;
  //     netdev->features &= ~NETIF_F_TSO6;
  //   }
  // }

  netdev->features &= ~NETIF_F_TSO;
  netdev->features &= ~NETIF_F_TSO6;

  /* Set user-changeable features (subset of all device features) */
  netdev->hw_features = netdev->features;
  netdev->hw_features |= NETIF_F_RXFCS;
  netdev->priv_flags |= IFF_SUPP_NOFCS;
  netdev->hw_features |= NETIF_F_RXALL;

  if (flags.has_hw_vlan_filter)
    netdev->features |= NETIF_F_HW_VLAN_CTAG_FILTER;

  netdev->vlan_features |=
      (NETIF_F_SG | NETIF_F_TSO | NETIF_F_TSO6 | NETIF_F_HW_CSUM);

  netdev->priv_flags |= IFF_UNICAST_FLT;

  netdev->features |= NETIF_F_HIGHDMA;
  netdev->vlan_features |= NETIF_F_HIGHDMA;

  /* MTU range: 68 - max_hw_frame_size */
  netdev->min_mtu = ETH_MIN_MTU;
  netdev->max_mtu = status->max_hw_frame_size - (VLAN_ETH_HLEN + ETH_FCS_LEN);

  e1000r_read_mac_addr(e1000, &mac_addr);

  // netdev->flags |= IFF_NOARP;
  // netdev->features |= NETIF_F_IP_CSUM;

  eth_hw_addr_set(netdev, mac_addr);
  if (!is_valid_ether_addr(netdev->dev_addr)) {
    dev_err(&pdev->dev, "Invalid MAC Address: %pM\n", netdev->dev_addr);
    err = -EIO;
    goto err_eeprom;
  }

  timer_setup(&adapter->watchdog_timer, e1000_watchdog, 0);
  INIT_WORK(&adapter->watchdog_task, e1000_watchdog_task);

  strscpy(netdev->name, "eth%d", sizeof(netdev->name));
  err = register_netdev(netdev);
  if (err)
    goto err_register;

  /* carrier off reporting is important to ethtool even BEFORE open */
  netif_carrier_off(netdev);

  e1000_print_device_info(e1000);

  dev_pm_set_driver_flags(&pdev->dev, DPM_FLAG_SMART_PREPARE);

  if (pci_dev_run_wake(pdev))
    pm_runtime_put_noidle(&pdev->dev);

  return 0;

err_register:
err_eeprom:
//   e1000_phy_hw_reset(hw);

// if (hw->flash_address)
// iounmap(hw->flash_address);
// kfree(adapter->tx_ring);
// kfree(adapter->rx_ring);
// err_dma:
err_sw_init:
  //   err_mdio_ioremap:
  //   iounmap(hw->ce4100_gbe_mdio_base_virt);
  //   if ((flash_addr) && (board_info.mac < e1000_pch_spt))
  //     iounmap(flash_addr);
  // err_flashmap:
  //   iounmap(hw_addr);

  e1000r_drop(e1000);
err_e1000r:
  free_netdev(netdev);
err_alloc_etherdev:
  pci_release_selected_regions(pdev, bars);
err_pci_reg:
err_dma:
  pci_disable_device(pdev);
  return err;
}

static const struct pci_device_id e1000_pci_tbl[] = {
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_COPPER), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_FIBER), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_QUAD_COPPER), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_QUAD_COPPER_LP), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_QUAD_FIBER), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_SERDES), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_SERDES_DUAL), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571EB_SERDES_QUAD), board_82571},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82571PT_QUAD_COPPER), board_82571},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82572EI), board_82572},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82572EI_COPPER), board_82572},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82572EI_FIBER), board_82572},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82572EI_SERDES), board_82572},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82573E), board_82573},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82573E_IAMT), board_82573},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82573L), board_82573},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82574L), board_82574},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82574LA), board_82574},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_82583V), board_82583},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_80003ES2LAN_COPPER_DPT),
     board_80003es2lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_80003ES2LAN_COPPER_SPT),
     board_80003es2lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_80003ES2LAN_SERDES_DPT),
     board_80003es2lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_80003ES2LAN_SERDES_SPT),
     board_80003es2lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IFE), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IFE_G), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IFE_GT), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IGP_AMT), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IGP_C), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IGP_M), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_IGP_M_AMT), board_ich8lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH8_82567V_3), board_ich8lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IFE), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IFE_G), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IFE_GT), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IGP_AMT), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IGP_C), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_BM), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IGP_M), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IGP_M_AMT), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH9_IGP_M_V), board_ich9lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_R_BM_LM), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_R_BM_LF), board_ich9lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_R_BM_V), board_ich9lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_D_BM_LM), board_ich10lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_D_BM_LF), board_ich10lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_ICH10_D_BM_V), board_ich10lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_M_HV_LM), board_pchlan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_M_HV_LC), board_pchlan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_D_HV_DM), board_pchlan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_D_HV_DC), board_pchlan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH2_LV_LM), board_pch2lan},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH2_LV_V), board_pch2lan},

    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LPT_I217_LM), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LPT_I217_V), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LPTLP_I218_LM), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LPTLP_I218_V), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_I218_LM2), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_I218_V2), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_I218_LM3), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_I218_V3), board_pch_lpt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_LM), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_V), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_LM2), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_V2), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LBG_I219_LM3), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_LM4), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_V4), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_LM5), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_SPT_I219_V5), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CNP_I219_LM6), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CNP_I219_V6), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CNP_I219_LM7), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CNP_I219_V7), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ICP_I219_LM8), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ICP_I219_V8), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ICP_I219_LM9), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ICP_I219_V9), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_LM10), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_V10), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_LM11), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_V11), board_pch_cnp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_LM12), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_CMP_I219_V12), board_pch_spt},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_LM13), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_V13), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_LM14), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_V14), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_LM15), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_TGP_I219_V15), board_pch_tgp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_RPL_I219_LM23), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_RPL_I219_V23), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ADP_I219_LM16), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ADP_I219_V16), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ADP_I219_LM17), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ADP_I219_V17), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_RPL_I219_LM22), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_RPL_I219_V22), board_pch_adp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_MTP_I219_LM18), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_MTP_I219_V18), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_MTP_I219_LM19), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_MTP_I219_V19), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LNP_I219_LM20), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LNP_I219_V20), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LNP_I219_LM21), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_LNP_I219_V21), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ARL_I219_LM24), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_ARL_I219_V24), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_LM25), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_V25), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_LM26), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_V26), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_LM27), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_PTP_I219_V27), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_NVL_I219_LM29), board_pch_mtp},
    {PCI_VDEVICE(INTEL, E1000_DEV_ID_PCH_NVL_I219_V29), board_pch_mtp},

    {0, 0, 0, 0, 0, 0, 0} /* terminate list */
};
MODULE_DEVICE_TABLE(pci, e1000_pci_tbl);

static const struct pci_error_handlers e1000_err_handler = {
    .error_detected = e1000_io_error_detected,
    .slot_reset = e1000_io_slot_reset,
    .resume = e1000_io_resume,
};

static SIMPLE_DEV_PM_OPS(e1000_pm_ops, e1000_suspend, e1000_resume);

static struct pci_driver e1000_driver = {.name = e1000_driver_name,
                                         .id_table = e1000_pci_tbl,
                                         .probe = e1000_probe,
                                         .remove = e1000_remove,
                                         .driver =
                                             {
                                                 .pm = &e1000_pm_ops,
                                             },
                                         .shutdown = e1000_shutdown,
                                         .err_handler = &e1000_err_handler};

module_init(e1000_init_module);
module_exit(e1000_exit_module);

int e1000_init_module(void) {
  int ret;
  pr_info("%s\n", e1000_rs_driver_string);

  pr_info("%s\n", e1000_rs_copyright);

  ret = pci_register_driver(&e1000_driver);
  if (copybreak != COPYBREAK_DEFAULT) {
    if (copybreak == 0)
      pr_info("copybreak disabled\n");
    else
      pr_info("copybreak enabled for "
              "packets <= %u bytes\n",
              copybreak);
  }
  return ret;
}

void e1000_exit_module(void) {
  pci_unregister_driver(&e1000_driver);
  return;
}
MODULE_AUTHOR("zhourui <zhourui818@126.com>");
MODULE_DESCRIPTION("e1000 extensions support");
MODULE_LICENSE("GPL");