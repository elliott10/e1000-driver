use super::regs;
use crate::*;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

#[derive(Debug)]
pub(crate) struct Hw {
    mmio: usize,
    pub(crate) board: Board,
    pub(crate) flash: Option<usize>,
    pub(crate) stats: E1000HWStats,
    pub(crate) board_info: BoardInfo,
}

impl Hw {
    pub(crate) fn new(mmio: usize, board: Board) -> Self {
        Hw {
            mmio: mmio,
            board,
            flash: None,
            stats: Default::default(),
            board_info: board.new_default_info(),
        }
    }

    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.mmio + offset) as *const T;
            ptr.read_volatile()
        }
    }

    pub(crate) fn set_flash(&mut self, flash: usize) {
        self.flash = Some(flash);
    }

    fn write<T>(&self, offset: usize, value: T) {
        unsafe {
            let ptr = (self.mmio + offset) as *mut T;
            ptr.write_volatile(value);
        }
    }

    fn ral(&self, index: usize) -> usize {
        regs::RA + index * 8
    }

    fn rah(&self, index: usize) -> usize {
        regs::RA + index * 8 + 4
    }

    pub(crate) fn get_ra(&self, index: usize) -> [u8; 6] {
        let ral = self.read::<u32>(self.ral(index));
        let rah = self.read::<u32>(self.rah(index));
        [
            ral as u8,
            (ral >> 8) as u8,
            (ral >> 16) as u8,
            (ral >> 24) as u8,
            rah as u8,
            (rah >> 8) as u8,
        ]
    }

    pub(crate) fn set_ra(&self, index: usize, mac: [u8; 6]) {
        unsafe {
            let mut ral = 0;
            let mut rah = 0;

            ral = (mac[0] as u32)
                | ((mac[1] as u32) << 8)
                | ((mac[2] as u32) << 16)
                | ((mac[3] as u32) << 24);
            rah = (mac[4] as u32) | ((mac[5] as u32) << 8);

            self.write::<u32>(self.ral(index), ral);
            self.write::<u32>(self.rah(index), rah);
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct E1000HWStats {
    pub crcerrs: u64,
    pub algnerrc: u64,
    pub symerrs: u64,
    pub rxerrc: u64,
    pub mpc: u64,
    pub scc: u64,
    pub ecol: u64,
    pub mcc: u64,
    pub latecol: u64,
    pub colc: u64,
    pub dc: u64,
    pub tncrs: u64,
    pub sec: u64,
    pub cexterr: u64,
    pub rlec: u64,
    pub xonrxc: u64,
    pub xontxc: u64,
    pub xoffrxc: u64,
    pub xofftxc: u64,
    pub fcruc: u64,
    pub prc64: u64,
    pub prc127: u64,
    pub prc255: u64,
    pub prc511: u64,
    pub prc1023: u64,
    pub prc1522: u64,
    pub gprc: u64,
    pub bprc: u64,
    pub mprc: u64,
    pub gptc: u64,
    pub gorc: u64,
    pub gotc: u64,
    pub rnbc: u64,
    pub ruc: u64,
    pub rfc: u64,
    pub roc: u64,
    pub rjc: u64,
    pub mgprc: u64,
    pub mgpdc: u64,
    pub mgptc: u64,
    pub tor: u64,
    pub tot: u64,
    pub tpr: u64,
    pub tpt: u64,
    pub ptc64: u64,
    pub ptc127: u64,
    pub ptc255: u64,
    pub ptc511: u64,
    pub ptc1023: u64,
    pub ptc1522: u64,
    pub mptc: u64,
    pub bptc: u64,
    pub tsctc: u64,
    pub tsctfc: u64,
    pub iac: u64,
    pub icrxptc: u64,
    pub icrxatc: u64,
    pub ictxptc: u64,
    pub ictxatc: u64,
    pub ictxqec: u64,
    pub ictxqmtc: u64,
    pub icrxdmtc: u64,
    pub icrxoc: u64,
}
