#include "linux/types.h"


void e1000r_drop(size_t ptr);

size_t e1000r_new(size_t mmio);

void e1000r_read_mac_addr(size_t ptr, uint8_t (*mac)[6]);
