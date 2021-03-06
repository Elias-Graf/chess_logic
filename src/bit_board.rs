use std::ops::{Index, IndexMut};

use crate::{type_alias_default::TypeAliasDefault, Board, Color};

pub const SIZE: u64 = Board::SIZE as u64;
pub const HEIGHT: u64 = Board::HEIGHT as u64;
pub const WIDTH: u64 = Board::WIDTH as u64;

/// # Usage
/// `board >> NORTH` or `idx - NORTH`
pub const NORTH: usize = 8;
/// # Usage
/// `board >> NO_EA` or `idx - NO_EA`
pub const NO_EA: usize = 7;
/// # Usage
/// `board << EAST` or `idx + EAST`
pub const EAST: usize = 1;
/// # Usage
/// `board << SO_EA` or `idx + SO_EA`
pub const SO_EA: usize = 9;
/// # Usage
/// `board << SOUTH` or `idx + SOUTH`
pub const SOUTH: usize = 8;
/// # Usage
/// `board << SO_WE` or `idx + SO_WE`
pub const SO_WE: usize = 7;
/// # Usage
/// `board >> WEST` or `idx - WEST`
pub const WEST: usize = 1;
/// # Usage
/// `board >> NO_WE` or `idx - NO_WE`
pub const NO_WE: usize = 9;

/// Created a new board with a `1` at the specified index.
pub fn with_bit_at(i: usize) -> u64 {
    let mut board = 0;
    set_bit(&mut board, i);
    board
}

pub fn is_bit_set(board: u64, i: usize) -> bool {
    get_bit(board, i) > 0
}

pub fn get_bit(board: u64, i: usize) -> u64 {
    board & (1 << i)
}

pub fn set_bit(board: &mut u64, i: usize) {
    *board |= 1 << i
}

pub fn clear_bit(board: &mut u64, i: usize) {
    *board &= !(1 << i)
}

/// Evaluates if the board has set bits - if it's truthy.
///
/// Would be the same as doing `if (board)` in languages that support general
/// truthy/falsy value evaluation (for example C).
pub fn has_set_bits(board: u64) -> bool {
    board > 0
}

/// Calculates the number of bits set to `1`.
pub fn count_set_bits(board: u64) -> usize {
    let mut board = board;
    let mut count = 0;

    while board > 0 {
        count += 1;

        board &= board - 1;
    }

    count
}

/// Returns the index of the first bit set to `1`.
///
/// This is also known as the least significant set bit. If no bits are set,
/// the function will return `None`.
pub fn get_first_set_bit(board: u64) -> Option<usize> {
    if board == 0 {
        return None;
    }

    let board = board as i128;
    // Set all the bits to 1 up to the first bit.
    let filled_up_to_first = ((board & -board) - 1) as u64;

    // If the 1 bits are now counted, we can retrieve the index of it.
    Some(count_set_bits(filled_up_to_first))
}

/// Displays a board in a human readable way.
///
/// For example:
/// ```text
/// 8   . . . . . . . .
/// 7   . . . . . . . .
/// 6   . . . . . . . .
/// 5   . . 1 . 1 . . .
/// 4   . . . . . . . .
/// 3   . . . . . . . .
/// 2   . . . . . . . .
/// 1   . . . . . . . .
///
///     a b c d e f g h
/// ```
pub fn display(board: u64) -> String {
    let mut val = String::new();

    for i in 0..Board::SIZE {
        let file = i % Board::HEIGHT;
        let rank = i / Board::HEIGHT;

        if file == 0 {
            val += &format!("{}  ", Board::HEIGHT - rank);
        }

        val += match get_bit(board, i) {
            0 => " .",
            _ => " 1",
        };

        if file == 7 {
            val += "\n";
        }
    }

    val += "\n    a b c d e f g h";
    val += &format!("\n\n    Decimal: {}", board);

    val
}

pub struct SetBitsIter(pub u64);

impl Iterator for SetBitsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = get_first_set_bit(self.0) {
            clear_bit(&mut self.0, i);

            return Some(i as usize);
        }

        None
    }
}

pub type U64PerSquare = [u64; Board::SIZE];

impl TypeAliasDefault for U64PerSquare {
    fn default() -> Self {
        [0; Board::SIZE]
    }
}

pub type ColoredU64PerSquare = [U64PerSquare; 2];

impl TypeAliasDefault for ColoredU64PerSquare {
    fn default() -> Self {
        [U64PerSquare::default(); 2]
    }
}

impl Index<Color> for ColoredU64PerSquare {
    type Output = U64PerSquare;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Color> for ColoredU64PerSquare {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
