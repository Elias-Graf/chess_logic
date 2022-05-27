use std::ops::{Index, IndexMut};

use crate::{square::BoardIdx, Board, Color};

/// # Usage
/// ```
/// board >> NORTH
/// ```
pub const NORTH: u64 = 8;
/// # Usage
/// ```
/// board >> NO_EA
/// ```
pub const NO_EA: u64 = 7;
/// # Usage
/// ```
/// board << EAST
/// ```
pub const EAST: u64 = 1;
/// # Usage
/// ```
/// board << SO_EA
/// ```
pub const SO_EA: u64 = 9;
/// # Usage
/// ```
/// board << SOUTH
/// ```
pub const SOUTH: u64 = 8;
/// # Usage
/// ```
/// board << SO_WE
/// ```
pub const SO_WE: u64 = 7;
/// # Usage
/// ```
/// board >> WEST
/// ```
pub const WEST: u64 = 1;
/// # Usage
/// ```
/// board >> NO_WE
/// ```
pub const NO_WE: u64 = 9;

pub fn is_set(board: u64, i: u64) -> bool {
    get_bit(board, i) > 0
}

pub fn get_bit(board: u64, i: u64) -> u64 {
    board & (1 << i)
}

pub fn set_bit(board: &mut u64, i: u64) {
    *board |= 1 << i
}

pub fn clear_bit(board: &mut u64, i: u64) {
    *board &= !(1 << i)
}

pub fn display(board: u64) -> String {
    let mut val = String::new();

    for i in 0..Board::SIZE {
        let file = i % Board::HEIGHT;
        let rank = i / Board::HEIGHT;

        if file == 0 {
            val += &format!("{}  ", Board::HEIGHT - rank);
        }

        val += match get_bit(board, i as u64) {
            0 => " 0",
            _ => " 1",
        };

        if file == 7 {
            val += "\n";
        }
    }

    val += "\n    a b c d e f g h";

    val
}

pub struct MoveMask {
    squares: [u64; 64],
}

impl MoveMask {
    pub fn new() -> Self {
        MoveMask { squares: [0; 64] }
    }
}

impl Index<&dyn BoardIdx> for MoveMask {
    type Output = u64;

    fn index(&self, index: &dyn BoardIdx) -> &Self::Output {
        &self.squares[index.idx() as usize]
    }
}

impl IndexMut<&dyn BoardIdx> for MoveMask {
    fn index_mut(&mut self, index: &dyn BoardIdx) -> &mut Self::Output {
        &mut self.squares[index.idx() as usize]
    }
}

pub struct ColoredMovedMask {
    masks: [MoveMask; 2],
}

impl ColoredMovedMask {
    pub fn new() -> Self {
        ColoredMovedMask {
            masks: [MoveMask::new(), MoveMask::new()],
        }
    }
}

impl Index<Color> for ColoredMovedMask {
    type Output = MoveMask;

    fn index(&self, index: Color) -> &Self::Output {
        &self.masks[index as usize]
    }
}

impl IndexMut<Color> for ColoredMovedMask {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self.masks[index as usize]
    }
}
