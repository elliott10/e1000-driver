#include "e1000.h"
#include "linux/cleanup.h"
#include "linux/etherdevice.h"
#include "linux/mroute.h"
#include "linux/printk.h"
#include "linux/refcount.h"
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

char e1000_driver_name[] = "e1000e";

#define DEFAULT_MSG_ENABLE (NETIF_MSG_DRV | NETIF_MSG_PROBE | NETIF_MSG_LINK)
static int debug = -1;
module_param(debug, int, 0);
MODULE_PARM_DESC(debug, "Debug level (0=none,...,16=all)");

static char e1000_rs_driver_string[] =
    "Intel(R) PRO/1000 Network Driver (Rust)";
static const char e1000_rs_copyright[] = "Copyright (c) 1999-2006 QCL";

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

static int __maybe_unused e1000_resume(struct device *dev) { return 0; }

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
int e1000e_open(struct net_device *netdev) { return -1; }
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
int e1000e_close(struct net_device *netdev) { return -1; }

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
                        struct rtnl_link_stats64 *stats) {}

/**
 * e1000e_set_rx_mode - secondary unicast, Multicast and Promiscuous mode set
 * @netdev: network interface device structure
 *
 * The ndo_set_rx_mode entry point is called whenever the unicast or multicast
 * address list or the network interface flags are updated.  This routine is
 * responsible for configuring the hardware for proper unicast, multicast,
 * promiscuous mode, and all-multi behavior.
 **/
static void e1000e_set_rx_mode(struct net_device *netdev) {}
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
/**
 * e1000_netpoll
 * @netdev: network interface device structure
 *
 * Polling 'interrupt' - used by things like netconsole to send skbs
 * without having to re-enable interrupts. It's not called while
 * the interrupt routine is executing.
 */
static void e1000_netpoll(struct net_device *netdev) {}
/**
 * e1000e_poll - NAPI Rx polling callback
 * @napi: struct associated with this polling callback
 * @budget: number of packets driver is allowed to process this poll
 **/
static int e1000e_poll(struct napi_struct *napi, int budget) { return -1; }
static int e1000_set_features(struct net_device *netdev,
                              netdev_features_t features) {
  return -1;
}
static netdev_features_t e1000_fix_features(struct net_device *netdev,
                                            netdev_features_t features) {
  return features;
}
void e1000e_reset_interrupt_capability(struct e1000_adapter *adapter) {
  size_t e1000 = e1000e_get_e1000(adapter->netdev);
  struct BoardInfo board = e1000r_board_info(e1000);
  struct Flags flags = board.flags;
  if (adapter->msix_entries) {
    pci_disable_msix(adapter->pdev);
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
  struct BoardInfo board = e1000r_board_info(e1000);
  struct Flags flags = board.flags;

  switch (adapter->int_mode) {
  case E1000E_INT_MODE_MSIX:
    if (flags.has_msix) {
      adapter->num_vectors = 3; /* RxQ0, TxQ0 and other */
      adapter->msix_entries =
          kcalloc(adapter->num_vectors, sizeof(struct msix_entry), GFP_KERNEL);
      if (adapter->msix_entries) {
        struct e1000_adapter *a = adapter;

        for (i = 0; i < adapter->num_vectors; i++)
          adapter->msix_entries[i].entry = i;

        err = pci_enable_msix_range(a->pdev, a->msix_entries, a->num_vectors,
                                    a->num_vectors);
        if (err > 0)
          return;
      }
      /* MSI-X failed, so fall through and try MSI */
      e_err("Failed to initialize MSI-X interrupts.  Falling back to MSI "
            "interrupts.\n");
      e1000e_reset_interrupt_capability(adapter);
    }
    adapter->int_mode = E1000E_INT_MODE_MSI;
    fallthrough;
  case E1000E_INT_MODE_MSI:
    if (!pci_enable_msi(adapter->pdev)) {
      flags.msi_enabled = true;
      e1000r_set_flags(e1000, flags);
    } else {
      adapter->int_mode = E1000E_INT_MODE_LEGACY;
      e_err("Failed to initialize MSI interrupts.  Falling back to legacy "
            "interrupts.\n");
    }
    fallthrough;
  case E1000E_INT_MODE_LEGACY:
    /* Don't do anything; this is the system default */
    break;
  }

  /* store the number of vectors being used */
  adapter->num_vectors = 1;
}
/**
 * e1000_irq_disable - Mask off interrupt generation on the NIC
 * @adapter: board private structure
 **/
static void e1000_irq_disable(struct e1000_adapter *adapter)
{
	e1000r_irq_disable(e1000e_get_e1000(adapter->netdev), adapter->msix_entries);

	if (adapter->msix_entries) {
		int i;

		for (i = 0; i < adapter->num_vectors; i++)
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

size_t cb_bar_n_remap(const void *pdev_v, size_t bar) {
  struct pci_dev *pdev = (struct pci_dev *)pdev_v;
  resource_size_t mmio_start = pci_resource_start(pdev, bar);
  resource_size_t mmio_len = pci_resource_len(pdev, bar);
  return (size_t)ioremap(mmio_start, mmio_len);
}
void cb_io_unmap(size_t addr) { iounmap((void *)addr); }

size_t e1000e_get_e1000(struct net_device *netdev){
  struct e1000_adapter *adapter = netdev_priv(netdev);
  return adapter->ptr;
}


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

  E1000Config config;
  config.pdev = pdev;
  config.device_id = ent->device;
  config.cb_iounmap = cb_io_unmap;
  config.cb_bar_n_remap = cb_bar_n_remap;

  size_t e1000 = e1000r_new(config);
  if (!e1000)
    goto err_e1000r;

  adapter = netdev_priv(netdev);
  adapter->ptr = e1000;
  adapter->bars = bars;
  adapter->pdev = pdev;
  adapter->netdev = netdev;
  adapter->msg_enable = netif_msg_init(debug, DEFAULT_MSG_ENABLE);

  struct BoardInfo board = e1000r_board_info(e1000);

  /* Set default EEE advertisement */
  if (board.flags2.has_eee)
    adapter->eee_advert = MDIO_EEE_100TX | MDIO_EEE_1000T;

  netdev->netdev_ops = &e1000e_netdev_ops;
  e1000e_set_ethtool_ops(netdev);

  netdev->watchdog_timeo = 5 * HZ;
  netif_napi_add(netdev, &adapter->napi, e1000e_poll);
  strscpy(netdev->name, pci_name(pdev), sizeof(netdev->name));

  netdev->mem_start = mmio_start;
  netdev->mem_end = mmio_start + mmio_len;
  adapter->bd_number = cards_found++;

  /* Set initial default active device features */
  netdev->features = (NETIF_F_SG | NETIF_F_HW_VLAN_CTAG_RX |
                      NETIF_F_HW_VLAN_CTAG_TX | NETIF_F_TSO | NETIF_F_TSO6 |
                      NETIF_F_RXHASH | NETIF_F_RXCSUM | NETIF_F_HW_CSUM);

  /* Set user-changeable features (subset of all device features) */
  netdev->hw_features = netdev->features;
  netdev->hw_features |= NETIF_F_RXFCS;
  netdev->priv_flags |= IFF_SUPP_NOFCS;
  netdev->hw_features |= NETIF_F_RXALL;

  u8 mac_addr[6];

  err = e1000r_init(e1000);
  if (err != 0)
    goto err_eeprom;

  err = e1000_sw_init(adapter);
  if (err)
    goto err_sw_init;

  e1000r_read_mac_addr(e1000, &mac_addr);

  eth_hw_addr_set(netdev, mac_addr);
  if (!is_valid_ether_addr(netdev->dev_addr)) {
    dev_err(&pdev->dev, "Invalid MAC Address: %pM\n", netdev->dev_addr);
    err = -EIO;
    goto err_eeprom;
  }

  strscpy(netdev->name, "eth%d", sizeof(netdev->name));
  err = register_netdev(netdev);
  if (err)
    goto err_register;

  pr_info("Intel(R) PRO/1000 Network Connection\n");

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