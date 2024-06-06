// SPDX-License-Identifier: GPL-2.0
/* Copyright(c) 1999 - 2018 Intel Corporation. */

/* ethtool support for e1000 */

#include "e1000.h"
#include <linux/delay.h>
#include <linux/ethtool.h>
#include <linux/interrupt.h>
#include <linux/netdevice.h>
#include <linux/pci.h>
#include <linux/pm_runtime.h>
#include <linux/slab.h>
#include <linux/vmalloc.h>

enum { NETDEV_STATS, E1000_STATS };

struct e1000_stats {
  char stat_string[ETH_GSTRING_LEN];
  int type;
  int sizeof_stat;
  int stat_offset;
};

static const char e1000e_priv_flags_strings[][ETH_GSTRING_LEN] = {
#define E1000E_PRIV_FLAGS_S0IX_ENABLED BIT(0)
    "s0ix-enabled",
};
#define E1000E_PRIV_FLAGS_STR_LEN ARRAY_SIZE(e1000e_priv_flags_strings)

#define E1000_STAT(str, m)                                                     \
  {                                                                            \
    .stat_string = str, .type = E1000_STATS,                                   \
    .sizeof_stat = sizeof(((struct E1000Status *)0)->m),                       \
    .stat_offset = offsetof(struct E1000Status, m)                             \
  }
#define E1000_NETDEV_STAT(str, m)                                              \
  {                                                                            \
    .stat_string = str, .type = NETDEV_STATS,                                  \
    .sizeof_stat = sizeof(((struct rtnl_link_stats64 *)0)->m),                 \
    .stat_offset = offsetof(struct rtnl_link_stats64, m)                       \
  }

static const struct e1000_stats e1000_gstrings_stats[] = {
    E1000_STAT("rx_packets", stats.gprc), E1000_STAT("tx_packets", stats.gptc),
    E1000_STAT("rx_bytes", stats.gorc), E1000_STAT("tx_bytes", stats.gotc),
    E1000_STAT("rx_broadcast", stats.bprc),
    E1000_STAT("tx_broadcast", stats.bptc),
    E1000_STAT("rx_multicast", stats.mprc),
    E1000_STAT("tx_multicast", stats.mptc),
    E1000_NETDEV_STAT("rx_errors", rx_errors),
    E1000_NETDEV_STAT("tx_errors", tx_errors),
    E1000_NETDEV_STAT("tx_dropped", tx_dropped),
    E1000_STAT("multicast", stats.mprc), E1000_STAT("collisions", stats.colc),
    E1000_NETDEV_STAT("rx_length_errors", rx_length_errors),
    E1000_NETDEV_STAT("rx_over_errors", rx_over_errors),
    E1000_STAT("rx_crc_errors", stats.crcerrs),
    E1000_NETDEV_STAT("rx_frame_errors", rx_frame_errors),
    E1000_STAT("rx_no_buffer_count", stats.rnbc),
    E1000_STAT("rx_missed_errors", stats.mpc),
    E1000_STAT("tx_aborted_errors", stats.ecol),
    E1000_STAT("tx_carrier_errors", stats.tncrs),
    E1000_NETDEV_STAT("tx_fifo_errors", tx_fifo_errors),
    E1000_NETDEV_STAT("tx_heartbeat_errors", tx_heartbeat_errors),
    E1000_STAT("tx_window_errors", stats.latecol),
    E1000_STAT("tx_abort_late_coll", stats.latecol),
    E1000_STAT("tx_deferred_ok", stats.dc),
    E1000_STAT("tx_single_coll_ok", stats.scc),
    E1000_STAT("tx_multi_coll_ok", stats.mcc),
    // E1000_STAT("tx_timeout_count", tx_timeout_count),
    // E1000_STAT("tx_restart_queue", restart_queue),
    E1000_STAT("rx_long_length_errors", stats.roc),
    E1000_STAT("rx_short_length_errors", stats.ruc),
    E1000_STAT("rx_align_errors", stats.algnerrc),
    E1000_STAT("tx_tcp_seg_good", stats.tsctc),
    E1000_STAT("tx_tcp_seg_failed", stats.tsctfc),
    E1000_STAT("rx_flow_control_xon", stats.xonrxc),
    E1000_STAT("rx_flow_control_xoff", stats.xoffrxc),
    E1000_STAT("tx_flow_control_xon", stats.xontxc),
    E1000_STAT("tx_flow_control_xoff", stats.xofftxc),
    // E1000_STAT("rx_csum_offload_good", hw_csum_good),
    // E1000_STAT("rx_csum_offload_errors", hw_csum_err),
    // E1000_STAT("rx_header_split", rx_hdr_split),
    // E1000_STAT("alloc_rx_buff_failed", alloc_rx_buff_failed),
    E1000_STAT("tx_smbus", stats.mgptc), E1000_STAT("rx_smbus", stats.mgprc),
    E1000_STAT("dropped_smbus", stats.mgpdc),
    // E1000_STAT("rx_dma_failed", rx_dma_failed),
    // E1000_STAT("tx_dma_failed", tx_dma_failed),
    // E1000_STAT("rx_hwtstamp_cleared", rx_hwtstamp_cleared),
    // E1000_STAT("uncorr_ecc_errors", uncorr_errors),
    // E1000_STAT("corr_ecc_errors", corr_errors),
    // E1000_STAT("tx_hwtstamp_timeouts", tx_hwtstamp_timeouts),
    // E1000_STAT("tx_hwtstamp_skipped", tx_hwtstamp_skipped),
};

#define E1000_GLOBAL_STATS_LEN ARRAY_SIZE(e1000_gstrings_stats)
#define E1000_STATS_LEN (E1000_GLOBAL_STATS_LEN)
static const char e1000_gstrings_test[][ETH_GSTRING_LEN] = {
    "Register test  (offline)", "Eeprom test    (offline)",
    "Interrupt test (offline)", "Loopback test  (offline)",
    "Link test   (on/offline)"};

#define E1000_TEST_LEN ARRAY_SIZE(e1000_gstrings_test)
static int e1000_get_link_ksettings(struct net_device *netdev,
                                    struct ethtool_link_ksettings *cmd) {

  return 0;
}

static int e1000_set_link_ksettings(struct net_device *netdev,
                                    const struct ethtool_link_ksettings *cmd) {
  return 0;
}

static void e1000_get_pauseparam(struct net_device *netdev,
                                 struct ethtool_pauseparam *pause) {}

static int e1000_set_pauseparam(struct net_device *netdev,
                                struct ethtool_pauseparam *pause) {
  return 0;
}

static u32 e1000_get_msglevel(struct net_device *netdev) {
  struct e1000_adapter *adapter = netdev_priv(netdev);
  return adapter->msg_enable;
}

static void e1000_set_msglevel(struct net_device *netdev, u32 data) {
  struct e1000_adapter *adapter = netdev_priv(netdev);
  adapter->msg_enable = data;
}

static int e1000_get_regs_len(struct net_device __always_unused *netdev) {
#define E1000_REGS_LEN 32 /* overestimate */
  return E1000_REGS_LEN * sizeof(u32);
}

static void e1000_get_regs(struct net_device *netdev, struct ethtool_regs *regs,
                           void *p) {}

static int e1000_get_eeprom_len(struct net_device *netdev) { return 0; }

static int e1000_get_eeprom(struct net_device *netdev,
                            struct ethtool_eeprom *eeprom, u8 *bytes) {
  return -1;
}

static int e1000_set_eeprom(struct net_device *netdev,
                            struct ethtool_eeprom *eeprom, u8 *bytes) {
  return 0;
}

static void e1000_get_drvinfo(struct net_device *netdev,
                              struct ethtool_drvinfo *drvinfo) {
  struct e1000_adapter *adapter = netdev_priv(netdev);

  strscpy(drvinfo->driver, e1000_driver_name, sizeof(drvinfo->driver));

  strscpy(drvinfo->bus_info, pci_name(adapter->pdev),
          sizeof(drvinfo->bus_info));
}

static void e1000_get_ringparam(struct net_device *netdev,
                                struct ethtool_ringparam *ring,
                                struct kernel_ethtool_ringparam *kernel_ring,
                                struct netlink_ext_ack *extack) {

  ring->rx_max_pending = E1000_MAX_RXD;
  ring->tx_max_pending = E1000_MAX_TXD;
  // ring->rx_pending = adapter->rx_ring_count;
  // ring->tx_pending = adapter->tx_ring_count;
}

static int e1000_set_ringparam(struct net_device *netdev,
                               struct ethtool_ringparam *ring,
                               struct kernel_ethtool_ringparam *kernel_ring,
                               struct netlink_ext_ack *extack) {

  return 0;
}

#define REG_PATTERN_TEST_ARRAY(reg, offset, mask, write)                       \
  do {                                                                         \
    if (reg_pattern_test(adapter, data, reg, offset, mask, write))             \
      return 1;                                                                \
  } while (0)
#define REG_PATTERN_TEST(reg, mask, write)                                     \
  REG_PATTERN_TEST_ARRAY(reg, 0, mask, write)

#define REG_SET_AND_CHECK(reg, mask, write)                                    \
  do {                                                                         \
    if (reg_set_and_check(adapter, data, reg, mask, write))                    \
      return 1;                                                                \
  } while (0)

static int e1000e_get_sset_count(struct net_device __always_unused *netdev,
                                 int sset) {
  switch (sset) {
  // case ETH_SS_TEST:
  //     return E1000_TEST_LEN;
  // case ETH_SS_STATS:
  //     return E1000_STATS_LEN;
  // case ETH_SS_PRIV_FLAGS:
  //     return E1000E_PRIV_FLAGS_STR_LEN;
  default:
    return -EOPNOTSUPP;
  }
}

static void e1000_diag_test(struct net_device *netdev,
                            struct ethtool_test *eth_test, u64 *data) {}

static void e1000_get_wol(struct net_device *netdev,
                          struct ethtool_wolinfo *wol) {

  wol->supported = 0;
  wol->wolopts = 0;
}

static int e1000_set_wol(struct net_device *netdev,
                         struct ethtool_wolinfo *wol) {

  return 0;
}

static int e1000_set_phys_id(struct net_device *netdev,
                             enum ethtool_phys_id_state state) {

  return 0;
}

static int e1000_get_coalesce(struct net_device *netdev,
                              struct ethtool_coalesce *ec,
                              struct kernel_ethtool_coalesce *kernel_coal,
                              struct netlink_ext_ack *extack) {
  struct E1000Status *status = e1000_status(netdev);

  if (status->itr_setting <= 4)
    ec->rx_coalesce_usecs = status->itr_setting;
  else
    ec->rx_coalesce_usecs = 1000000 / status->itr_setting;
  return 0;
}

static int e1000_set_coalesce(struct net_device *netdev,
                              struct ethtool_coalesce *ec,
                              struct kernel_ethtool_coalesce *kernel_coal,
                              struct netlink_ext_ack *extack) {
  struct E1000Status *adapter = e1000_status(netdev);
  if ((ec->rx_coalesce_usecs > E1000_MAX_ITR_USECS) ||
      ((ec->rx_coalesce_usecs > 4) &&
       (ec->rx_coalesce_usecs < E1000_MIN_ITR_USECS)) ||
      (ec->rx_coalesce_usecs == 2))
    return -EINVAL;

  if (ec->rx_coalesce_usecs == 4) {
    adapter->itr_setting = 4;
    adapter->itr = adapter->itr_setting;
  } else if (ec->rx_coalesce_usecs <= 3) {
    adapter->itr = 20000;
    adapter->itr_setting = ec->rx_coalesce_usecs;
  } else {
    adapter->itr = (1000000 / ec->rx_coalesce_usecs);
    adapter->itr_setting = adapter->itr & ~3;
  }

  pm_runtime_get_sync(netdev->dev.parent);

  // if (adapter->itr_setting != 0)
  // e1000e_write_itr(adapter, adapter->itr);
  // else
  // e1000e_write_itr(adapter, 0);

  pm_runtime_put_sync(netdev->dev.parent);
  return 0;
}

static int e1000_nway_reset(struct net_device *netdev) { return 0; }

static void e1000_get_ethtool_stats(struct net_device *netdev,
                                    struct ethtool_stats __always_unused *stats,
                                    u64 *data) {}

static void e1000_get_strings(struct net_device __always_unused *netdev,
                              u32 stringset, u8 *data) {}

static int e1000_get_rxnfc(struct net_device *netdev,
                           struct ethtool_rxnfc *info,
                           u32 __always_unused *rule_locs) {
  info->data = 0;

  return 0;
}

static int e1000e_get_eee(struct net_device *netdev,
                          struct ethtool_eee *edata) {
  return 0;
}

static int e1000e_set_eee(struct net_device *netdev,
                          struct ethtool_eee *edata) {

  return 0;
}

static int e1000e_get_ts_info(struct net_device *netdev,
                              struct ethtool_ts_info *info) {
  return 0;
}

static u32 e1000e_get_priv_flags(struct net_device *netdev) {
  struct e1000_adapter *adapter = netdev_priv(netdev);
  u32 priv_flags = 0;
  Flags2 flags2 = e1000r_get_info_all(adapter->ptr).flags2;

  if (flags2.enable_s0ix_flows)
    priv_flags |= E1000E_PRIV_FLAGS_S0IX_ENABLED;

  return priv_flags;
}

static int e1000e_set_priv_flags(struct net_device *netdev, u32 priv_flags) {
  return 0;
}

static const struct ethtool_ops e1000_ethtool_ops = {
    .supported_coalesce_params = ETHTOOL_COALESCE_RX_USECS,
    .get_drvinfo = e1000_get_drvinfo,
    .get_regs_len = e1000_get_regs_len,
    .get_regs = e1000_get_regs,
    .get_wol = e1000_get_wol,
    .set_wol = e1000_set_wol,
    .get_msglevel = e1000_get_msglevel,
    .set_msglevel = e1000_set_msglevel,
    .nway_reset = e1000_nway_reset,
    .get_link = ethtool_op_get_link,
    .get_eeprom_len = e1000_get_eeprom_len,
    .get_eeprom = e1000_get_eeprom,
    .set_eeprom = e1000_set_eeprom,
    .get_ringparam = e1000_get_ringparam,
    .set_ringparam = e1000_set_ringparam,
    .get_pauseparam = e1000_get_pauseparam,
    .set_pauseparam = e1000_set_pauseparam,
    .self_test = e1000_diag_test,
    .get_strings = e1000_get_strings,
    .set_phys_id = e1000_set_phys_id,
    .get_ethtool_stats = e1000_get_ethtool_stats,
    .get_sset_count = e1000e_get_sset_count,
    .get_coalesce = e1000_get_coalesce,
    .set_coalesce = e1000_set_coalesce,
    .get_rxnfc = e1000_get_rxnfc,
    .get_ts_info = e1000e_get_ts_info,
    .get_eee = e1000e_get_eee,
    .set_eee = e1000e_set_eee,
    .get_link_ksettings = e1000_get_link_ksettings,
    .set_link_ksettings = e1000_set_link_ksettings,
    .get_priv_flags = e1000e_get_priv_flags,
    .set_priv_flags = e1000e_set_priv_flags,
};

void e1000e_set_ethtool_ops(struct net_device *netdev) {
  netdev->ethtool_ops = &e1000_ethtool_ops;
}
