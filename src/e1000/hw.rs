use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use super::regs;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hw {
    mmio: usize,
    board: Board,
}
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum Board {
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

impl Hw {
    pub(crate) fn new(mmio: usize) -> Self {
        Hw {
            mmio: mmio,
            board: Board::board_pch_mtp,
        }
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
        unsafe {
            let l = &*slice_from_raw_parts(&ral as *const u32 as *const u8, 4);
            let h = &*slice_from_raw_parts(&rah as *const u32 as *const u8, 4);

            [l[0], l[1], l[2], l[3], h[0], h[1]]
        }
    }

    pub(crate) fn set_ra(&self, index: usize, mac: [u8; 6]) {
        unsafe {
            let mut ral = 0;
            let mut rah = 0;
            let l = &mut *slice_from_raw_parts_mut(&mut ral as *mut u32 as *mut u8, 4);
            let h = &mut *slice_from_raw_parts_mut(&mut rah as *mut u32 as *mut u8, 4);

            l[0] = mac[0];
            l[1] = mac[1];
            l[2] = mac[2];
            l[3] = mac[3];
            h[0] = mac[4];
            h[1] = mac[5];

            self.write::<u32>(self.ral(index), ral);
            self.write::<u32>(self.rah(index), rah);
        }
    }
}
