#include "e1000.h"
#include <linux/bitops.h>
#include <linux/if_vlan.h>
#include <linux/init.h>
#include <linux/io.h>
#include <linux/module.h>
#include <linux/prefetch.h>
#include <net/ip6_checksum.h>

char e1000_driver_name[] = "e1000_rs";
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
static void e1000_shutdown(struct pci_dev *pdev);

#define COPYBREAK_DEFAULT 256
static unsigned int copybreak __read_mostly = COPYBREAK_DEFAULT;
module_param(copybreak, uint, 0644);
MODULE_PARM_DESC(copybreak,
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
  			   "packets <= %u bytes\n", copybreak);
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

  return -1;
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
static void e1000_remove(struct pci_dev *pdev) {}
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
						pci_channel_state_t state)
{
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
static pci_ers_result_t e1000_io_slot_reset(struct pci_dev *pdev)
{
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
static void e1000_io_resume(struct pci_dev *pdev)
{
}
