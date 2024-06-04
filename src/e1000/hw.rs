use super::regs;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hw {
    mmio: usize,
    board: Board,
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(usize)]
pub enum Board {
    board_82571,
    board_82572,
    board_82573,
    board_82574,
    board_82583,
    board_80003es2lan,
    board_ich8lan,
    board_ich9lan,
    board_ich10lan,
    board_pchlan,
    board_pch2lan,
    board_pch_lpt,
    board_pch_spt,
    board_pch_cnp,
    board_pch_tgp,
    board_pch_adp,
    board_pch_mtp,
}
impl From<usize> for Board {
    fn from(value: usize) -> Self {
        unsafe { *(&value as *const usize as *const Board) }
    }
}

impl Hw {
    pub(crate) fn new(mmio: usize, board: Board) -> Self {
        Hw { mmio: mmio, board }
    }

    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.mmio + offset) as *const T;
            ptr.read_volatile()
        }
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
