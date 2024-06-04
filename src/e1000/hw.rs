use super::regs;
use crate::*;
use core::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

#[derive(Debug, Clone, Copy)]
pub(crate) struct Hw {
    mmio: usize,
    board: Board,
    flash: Option<usize>,
}

impl Hw {
    pub(crate) fn new(mmio: usize, board: Board) -> Self {
        Hw {
            mmio: mmio,
            board,
            flash: None,
        }
    }

    fn read<T>(&self, offset: usize) -> T {
        unsafe {
            let ptr = (self.mmio + offset) as *const T;
            ptr.read_volatile()
        }
    }
    pub(crate) fn board_info(&self) -> BoardInfo {
        self.board.get_info()
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
