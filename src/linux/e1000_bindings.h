#include "linux/types.h"


typedef enum InterruptMode {
  Legacy,
  MSI,
  MSIX,
} InterruptMode;

typedef struct E1000RcvPkt {
  const uint8_t *data;
  size_t length;
} E1000RcvPkt;

typedef void (*E1000CbOnRcv)(const void *pdev, struct E1000RcvPkt pkt);

typedef struct Settings {
  bool enable_msi;
} Settings;

/**
 * 通过bar获取到映射地址
 * return: 映射地址
 */
typedef size_t (*E1000CbBarNremap)(const void *pdev, size_t bar);

/**
 * 释放映射地址
 */
typedef void (*E1000CbIounmap)(size_t addr);

typedef void (*E1000CbUdelay)(uint32_t usecs);

typedef struct E1000rPcieInfo {
  uint16_t bus_link_width;
} E1000rPcieInfo;

typedef struct E1000rPcieInfo (*E1000CbPcieInfo)(const void *pdev);

typedef bool (*E1000CbMtuBgEth)(const void *pdev);

typedef size_t (*E1000CbDmaAllocCoherent)(const void *pdev, size_t pages, uint64_t *dma);

typedef void (*E1000CbDmaFreeCoherent)(const void *pdev, size_t vaddr, uint64_t dma, size_t pages);

typedef uint16_t (*E1000CbPciReadConfigWord)(const void *pdev, int32_t where_);

typedef void (*E1000CbOnXmitCompleted)(const void *pdev, uint32_t pkts, uint32_t bytes);

typedef void (*E1000CbEnableNet)(const void *pdev);

typedef int32_t (*E1000CbPcieCapOffset)(const void *pdev);

typedef struct E1000Config {
  uint32_t device_id;
  struct Settings settings;
  E1000CbBarNremap cb_bar_n_remap;
  E1000CbIounmap cb_iounmap;
  E1000CbUdelay cb_udelay;
  E1000CbPcieInfo cb_pcie_info;
  E1000CbMtuBgEth cb_mtu_bg_eth;
  E1000CbDmaAllocCoherent cb_dma_alloc_coherent;
  E1000CbDmaFreeCoherent cb_dma_free_coherent;
  E1000CbPciReadConfigWord cb_pci_read_config_word;
  E1000CbOnXmitCompleted cb_on_xmit_completed;
  E1000CbEnableNet cb_enable_net;
  E1000CbPcieCapOffset cb_pcie_cap_offset;
  const void *adapter;
} E1000Config;

typedef struct E1000HWStats {
  uint64_t crcerrs;
  uint64_t algnerrc;
  uint64_t symerrs;
  uint64_t rxerrc;
  uint64_t mpc;
  uint64_t scc;
  uint64_t ecol;
  uint64_t mcc;
  uint64_t latecol;
  uint64_t colc;
  uint64_t dc;
  uint64_t tncrs;
  uint64_t sec;
  uint64_t cexterr;
  uint64_t rlec;
  uint64_t xonrxc;
  uint64_t xontxc;
  uint64_t xoffrxc;
  uint64_t xofftxc;
  uint64_t fcruc;
  uint64_t prc64;
  uint64_t prc127;
  uint64_t prc255;
  uint64_t prc511;
  uint64_t prc1023;
  uint64_t prc1522;
  uint64_t gprc;
  uint64_t bprc;
  uint64_t mprc;
  uint64_t gptc;
  uint64_t gorc;
  uint64_t gotc;
  uint64_t rnbc;
  uint64_t ruc;
  uint64_t rfc;
  uint64_t roc;
  uint64_t rjc;
  uint64_t mgprc;
  uint64_t mgpdc;
  uint64_t mgptc;
  uint64_t tor;
  uint64_t tot;
  uint64_t tpr;
  uint64_t tpt;
  uint64_t ptc64;
  uint64_t ptc127;
  uint64_t ptc255;
  uint64_t ptc511;
  uint64_t ptc1023;
  uint64_t ptc1522;
  uint64_t mptc;
  uint64_t bptc;
  uint64_t tsctc;
  uint64_t tsctfc;
  uint64_t iac;
  uint64_t icrxptc;
  uint64_t icrxatc;
  uint64_t ictxptc;
  uint64_t ictxatc;
  uint64_t ictxqec;
  uint64_t ictxqmtc;
  uint64_t icrxdmtc;
  uint64_t icrxoc;
} E1000HWStats;

typedef struct E1000XmitConfig {
  uint64_t timestamp;
  uint32_t segs;
  bool ipv4;
  bool no_fcs;
  bool vlan_tag_present;
} E1000XmitConfig;

void e1000_print_device_info(size_t ptr);

size_t e1000r_clear_rx(size_t ptr, uint32_t max, E1000CbOnRcv on_rcv, const void *meta);

void e1000r_configure_msix(size_t ptr);

int32_t e1000r_down(size_t ptr);

void e1000r_drop(size_t ptr);

bool e1000r_irq_clean_tx(size_t ptr);

void e1000r_irq_disable(size_t ptr);

void e1000r_irq_enable(size_t ptr);

enum InterruptMode e1000r_irq_get_mode(size_t ptr);

int32_t e1000r_irq_handle(size_t ptr, int32_t irq);

int32_t e1000r_irq_handle_msix_tx(size_t ptr, int32_t irq);

void e1000r_irq_set_mode(size_t ptr, enum InterruptMode mode);

size_t e1000r_new(struct E1000Config config);

int32_t e1000r_open(size_t ptr);

void e1000r_read_mac_addr(size_t ptr, uint8_t (*mac)[6]);

struct E1000HWStats e1000r_stats64(size_t ptr);

bool e1000r_task_watchdog(size_t ptr);

int32_t e1000r_transmit(size_t ptr, struct E1000XmitConfig cfg, const uint8_t *pkt, uint32_t len);

void e1000r_trigger_lsc(size_t ptr);
