#include "e1000.h"
#include "linux/device.h"
#include "linux/types.h"
#include <linux/bitops.h>
#include <linux/if_vlan.h>
#include <linux/init.h>
#include <linux/io.h>
#include <linux/module.h>
#include <linux/prefetch.h>
#include <net/ip6_checksum.h>

char e1000_rs_driver_name[] = "e1000_rs";
static char e1000_rs_driver_string[] =
    "Intel(R) PRO/1000 Network Driver (Rust)";
static const char e1000_rs_copyright[] = "Copyright (c) 1999-2006 QCL";

/* e1000_pci_tbl - PCI Device ID Table
 *
 * Last entry must be all 0s
 *
 * Macro expands to...
 *   {PCI_DEVICE(PCI_VENDOR_ID_INTEL, device_id)}
 */
static const struct pci_device_id e1000_pci_tbl[] = {
    INTEL_E1000_ETHERNET_DEVICE(0x1000),
    INTEL_E1000_ETHERNET_DEVICE(0x1001),
    INTEL_E1000_ETHERNET_DEVICE(0x1004),
    INTEL_E1000_ETHERNET_DEVICE(0x1008),
    INTEL_E1000_ETHERNET_DEVICE(0x1009),
    INTEL_E1000_ETHERNET_DEVICE(0x100C),
    INTEL_E1000_ETHERNET_DEVICE(0x100D),
    INTEL_E1000_ETHERNET_DEVICE(0x100E),
    INTEL_E1000_ETHERNET_DEVICE(0x100F),
    INTEL_E1000_ETHERNET_DEVICE(0x1010),
    INTEL_E1000_ETHERNET_DEVICE(0x1011),
    INTEL_E1000_ETHERNET_DEVICE(0x1012),
    INTEL_E1000_ETHERNET_DEVICE(0x1013),
    INTEL_E1000_ETHERNET_DEVICE(0x1014),
    INTEL_E1000_ETHERNET_DEVICE(0x1015),
    INTEL_E1000_ETHERNET_DEVICE(0x1016),
    INTEL_E1000_ETHERNET_DEVICE(0x1017),
    INTEL_E1000_ETHERNET_DEVICE(0x1018),
    INTEL_E1000_ETHERNET_DEVICE(0x1019),
    INTEL_E1000_ETHERNET_DEVICE(0x101A),
    INTEL_E1000_ETHERNET_DEVICE(0x101D),
    INTEL_E1000_ETHERNET_DEVICE(0x101E),
    INTEL_E1000_ETHERNET_DEVICE(0x1026),
    INTEL_E1000_ETHERNET_DEVICE(0x1027),
    INTEL_E1000_ETHERNET_DEVICE(0x1028),
    INTEL_E1000_ETHERNET_DEVICE(0x1075),
    INTEL_E1000_ETHERNET_DEVICE(0x1076),
    INTEL_E1000_ETHERNET_DEVICE(0x1077),
    INTEL_E1000_ETHERNET_DEVICE(0x1078),
    INTEL_E1000_ETHERNET_DEVICE(0x1079),
    INTEL_E1000_ETHERNET_DEVICE(0x107A),
    INTEL_E1000_ETHERNET_DEVICE(0x107B),
    INTEL_E1000_ETHERNET_DEVICE(0x107C),
    INTEL_E1000_ETHERNET_DEVICE(0x108A),
    INTEL_E1000_ETHERNET_DEVICE(0x1099),
    INTEL_E1000_ETHERNET_DEVICE(0x10B5),
    INTEL_E1000_ETHERNET_DEVICE(0x2E6E),
    /* required last entry */
    {
        0,
    }};
MODULE_DEVICE_TABLE(pci, e1000_pci_tbl);

static int e1000_init_module(void);
static void e1000_exit_module(void);
static int e1000_probe(struct pci_dev *pdev, const struct pci_device_id *ent);
static void e1000_remove(struct pci_dev *pdev);
static int __maybe_unused e1000_suspend(struct device *dev);
static int __maybe_unused e1000_resume(struct device *dev);
static int e1000_is_need_ioport(struct pci_dev *pdev);
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

static const struct pci_error_handlers e1000_err_handler = {
    .error_detected = e1000_io_error_detected,
    .slot_reset = e1000_io_slot_reset,
    .resume = e1000_io_resume,
};

static SIMPLE_DEV_PM_OPS(e1000_pm_ops, e1000_suspend, e1000_resume);

static struct pci_driver e1000_driver = {.name = e1000_rs_driver_name,
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
  // static int global_quad_port_a; /* global ksp3 port a indication */
  // int i, pci_using_dac;
  // u16 eeprom_data = 0;
  // u16 tmp = 0;
  // u16 eeprom_apme_mask = E1000_EEPROM_APME;
  int bars, need_ioport;
  bool disable_dev = false;

  /* do not allocate ioport bars when not needed */
  need_ioport = e1000_is_need_ioport(pdev);
  if (need_ioport) {
    bars = pci_select_bars(pdev, IORESOURCE_MEM | IORESOURCE_IO);
    err = pci_enable_device(pdev);
  } else {
    bars = pci_select_bars(pdev, IORESOURCE_MEM);
    err = pci_enable_device_mem(pdev);
  }
  if (err)
    return err;

  err = pci_request_selected_regions(pdev, bars, e1000_rs_driver_name);
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

  pci_set_drvdata(pdev, netdev);

  void *hw_addr = pci_ioremap_bar(pdev, BAR_0);
  if (!hw_addr)
    goto err_ioremap;

  size_t e1000 = e1000_new((size_t)hw_addr);
  if (!e1000)
    goto err_alloc_etherdev;

  adapter = netdev_priv(netdev);
  adapter->ptr = e1000;
  // adapter->netdev = netdev;
  // adapter->pdev = pdev;
  // adapter->msg_enable = netif_msg_init(debug, DEFAULT_MSG_ENABLE);
  adapter->bars = bars;
  //   adapter->need_ioport = need_ioport;

  // hw = &adapter->hw;
  // hw->back = adapter;

  strcpy(netdev->name, "eth%d");
  err = register_netdev(netdev);
  if (err)
    goto err_register;

  //   /* print bus type/speed/width info */
  //   e_info(probe, "(PCI%s:%dMHz:%d-bit) %pM\n",
  //          ((hw->bus_type == e1000_bus_type_pcix) ? "-X" : ""),
  //          ((hw->bus_speed == e1000_bus_speed_133)   ? 133
  //           : (hw->bus_speed == e1000_bus_speed_120) ? 120
  //           : (hw->bus_speed == e1000_bus_speed_100) ? 100
  //           : (hw->bus_speed == e1000_bus_speed_66)  ? 66
  //                                                    : 33),
  //          ((hw->bus_width == e1000_bus_width_64) ? 64 : 32),
  //          netdev->dev_addr);

  /* carrier off reporting is important to ethtool even BEFORE open */
  netif_carrier_off(netdev);

  //   e_info(probe, "Intel(R) PRO/1000 Network Connection\n");
  pr_info("Intel(R) PRO/1000 Network Connection\n");

  cards_found++;
  return 0;

err_register:
  //   err_eeprom:
  //   e1000_phy_hw_reset(hw);

  // if (hw->flash_address)
  // iounmap(hw->flash_address);
  // kfree(adapter->tx_ring);
  // kfree(adapter->rx_ring);
  // err_dma:
  // err_sw_init:
  //   err_mdio_ioremap:
  //   iounmap(hw->ce4100_gbe_mdio_base_virt);
  iounmap(hw_addr);
err_ioremap:
  // disable_dev = !test_and_set_bit(__E1000_DISABLED, &adapter->flags);
  free_netdev(netdev);
err_alloc_etherdev:
  pci_release_selected_regions(pdev, bars);
err_pci_reg:
  if (!adapter || disable_dev)
    pci_disable_device(pdev);
  return err;
}

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

  e1000_drop(adapter->ptr);

  unregister_netdev(netdev);
  pci_release_selected_regions(pdev, adapter->bars);
}
static int __maybe_unused e1000_suspend(struct device *dev) { return 0; }

static int __maybe_unused e1000_resume(struct device *dev) { return 0; }

static void e1000_shutdown(struct pci_dev *pdev) {}

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
/**
 * e1000_is_need_ioport - determine if an adapter needs ioport resources or not
 * @pdev: PCI device information struct
 *
 * Return true if an adapter needs ioport resources
 **/
static int e1000_is_need_ioport(struct pci_dev *pdev) {
  return false;
  // switch (pdev->device) {
  // case E1000_DEV_ID_82540EM:
  // case E1000_DEV_ID_82540EM_LOM:
  // case E1000_DEV_ID_82540EP:
  // case E1000_DEV_ID_82540EP_LOM:
  // case E1000_DEV_ID_82540EP_LP:
  // case E1000_DEV_ID_82541EI:
  // case E1000_DEV_ID_82541EI_MOBILE:
  // case E1000_DEV_ID_82541ER:
  // case E1000_DEV_ID_82541ER_LOM:
  // case E1000_DEV_ID_82541GI:
  // case E1000_DEV_ID_82541GI_LF:
  // case E1000_DEV_ID_82541GI_MOBILE:
  // case E1000_DEV_ID_82544EI_COPPER:
  // case E1000_DEV_ID_82544EI_FIBER:
  // case E1000_DEV_ID_82544GC_COPPER:
  // case E1000_DEV_ID_82544GC_LOM:
  // case E1000_DEV_ID_82545EM_COPPER:
  // case E1000_DEV_ID_82545EM_FIBER:
  // case E1000_DEV_ID_82546EB_COPPER:
  // case E1000_DEV_ID_82546EB_FIBER:
  // case E1000_DEV_ID_82546EB_QUAD_COPPER:
  // 	return true;
  // default:
  // 	return false;
  // }
}