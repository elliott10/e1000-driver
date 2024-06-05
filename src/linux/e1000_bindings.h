#include "linux/types.h"


enum E1000MacType {
  e1000_82571,
  e1000_82572,
  e1000_82573,
  e1000_82574,
  e1000_82583,
  e1000_80003es2lan,
  e1000_ich8lan,
  e1000_ich9lan,
  e1000_ich10lan,
  e1000_pchlan,
  e1000_pch2lan,
  e1000_pch_lpt,
  e1000_pch_spt,
  e1000_pch_cnp,
  e1000_pch_tgp,
  e1000_pch_adp,
  e1000_pch_mtp,
  e1000_pch_lnp,
  e1000_pch_ptp,
  e1000_pch_nvp,
};
typedef uint32_t E1000MacType;

typedef struct Flags {
  bool has_amt;
  bool has_flash;
  bool has_hw_vlan_filter;
  bool has_wol;
  bool has_ctrlext_on_load;
  bool has_swsm_on_load;
  bool has_jumbo_frames;
  bool read_only_nvm;
  bool is_ich;
  bool has_msix;
  bool has_smart_power_down;
  bool is_quad_port_a;
  bool is_quad_port;
  bool has_hw_timestamp;
  bool apme_in_wuc;
  bool apme_in_ctrl3;
  bool apme_check_port_b;
  bool disable_fc_pause_time;
  bool no_wake_ucast;
  bool mng_pt_enabled;
  bool reset_overwrites_laa;
  bool tarc_speed_mode_bit;
  bool tarc_set_bit_zero;
  bool rx_needs_restart;
  bool lsc_gig_speed_drop;
  bool smart_power_down;
  bool msi_enabled;
  bool tso_force;
  bool restart_now;
  bool msi_test_failed;
} Flags;

typedef struct Flags2 {
  bool crc_stripping;
  bool has_phy_wakeup;
  bool is_discarding;
  bool disable_aspm_l1;
  bool has_phy_stats;
  bool has_eee;
  bool dma_burst;
  bool disable_aspm_l0s;
  bool disable_aim;
  bool check_phy_hang;
  bool no_disable_rx;
  bool pcim2pci_arbiter_wa;
  bool dflt_crc_stripping;
  bool check_rx_hwtstamp;
  bool check_systim_overflow;
  bool enable_s0ix_flows;
} Flags2;

typedef struct BoardInfo {
  E1000MacType mac;
  struct Flags flags;
  struct Flags2 flags2;
  uint32_t pba;
  uint32_t max_hw_frame_size;
} BoardInfo;

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

typedef struct E1000Config {
  uint32_t device_id;
  E1000CbBarNremap cb_bar_n_remap;
  E1000CbIounmap cb_iounmap;
  E1000CbUdelay cb_udelay;
  const void *pdev;
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

typedef struct E1000Status {
  struct E1000HWStats stats;
  uint32_t itr;
  uint32_t itr_setting;
  uint16_t tx_it;
  uint16_t rx_itr;
  uint32_t tx_int_delay;
  uint32_t tx_abs_int_delay;
  uint32_t rx_int_delay;
  uint32_t rx_abs_int_delay;
  uint32_t total_tx_bytes;
  uint32_t total_tx_packets;
  uint32_t total_rx_bytes;
  uint32_t total_rx_packets;
} E1000Status;

struct BoardInfo e1000r_board_info(size_t ptr);

void e1000r_drop(size_t ptr);

int32_t e1000r_init(size_t ptr);

void e1000r_irq_disable(size_t ptr, bool misx_enabled);

int32_t e1000r_irq_handle(size_t ptr, int32_t irq);

size_t e1000r_new(struct E1000Config config);

void e1000r_read_mac_addr(size_t ptr, uint8_t (*mac)[6]);

void e1000r_set_flags(size_t ptr, struct Flags flags);

struct E1000Status *e1000r_status(size_t ptr);
