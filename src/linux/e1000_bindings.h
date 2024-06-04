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

struct BoardInfo e1000r_board_info(size_t ptr);

void e1000r_drop(size_t ptr);

int32_t e1000r_init(size_t ptr);

size_t e1000r_new(size_t mmio, size_t board);

void e1000r_read_mac_addr(size_t ptr, uint8_t (*mac)[6]);

void e1000r_set_flash(size_t ptr, size_t flash);
