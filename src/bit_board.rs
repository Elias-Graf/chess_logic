use std::ops::{Index, IndexMut};

use crate::{square::BoardPos, Board, Color};

/// Custom default trait.
///
/// Same functionality as the [`Default`] trait, but defined in this create. That
/// way it can be used on type aliases.
pub trait CustomDefault {
    fn default() -> Self;
}

pub const SIZE: u64 = Board::SIZE as u64;
pub const HEIGHT: u64 = Board::HEIGHT as u64;
pub const WIDTH: u64 = Board::WIDTH as u64;

/// # Usage
/// ```ignore
/// board >> NORTH
/// ```
pub const NORTH: u64 = 8;
/// # Usage
/// ```ignore
/// board >> NO_EA
/// ```
pub const NO_EA: u64 = 7;
/// # Usage
/// ```ignore
/// board << EAST
/// ```
pub const EAST: u64 = 1;
/// # Usage
/// ```ignore
/// board << SO_EA
/// ```
pub const SO_EA: u64 = 9;
/// # Usage
/// ```ignore
/// board << SOUTH
/// ```
pub const SOUTH: u64 = 8;
/// # Usage
/// ```ignore
/// board << SO_WE
/// ```
pub const SO_WE: u64 = 7;
/// # Usage
/// ```ignore
/// board >> WEST
/// ```
pub const WEST: u64 = 1;
/// # Usage
/// ```ignore
/// board >> NO_WE
/// ```
pub const NO_WE: u64 = 9;

/// Created a new board with a `1` at the specified index.
pub fn with_bit_at(i: u64) -> u64 {
    let mut board = 0;
    set_bit(&mut board, i);
    board
}

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

/// Evaluates if the board has set bits - if it's truthy.
///
/// Would be the same as doing `if (board)` in languages that support general
/// truthy/falsy value evaluation (for example C).
pub fn has_set_bits(board: u64) -> bool {
    board > 0
}

/// Calculates the number of bits set to `1`.
pub fn count_set_bits(board: u64) -> u64 {
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
pub fn get_first_set_bit(board: u64) -> Option<u64> {
    if board == 0 {
        return None;
    }

    let board = board as i64;
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

        val += match get_bit(board, i as u64) {
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

pub type U64PerSquare = [u64; Board::SIZE];

impl CustomDefault for U64PerSquare {
    fn default() -> Self {
        [0; Board::SIZE]
    }
}

impl Index<&dyn BoardPos> for U64PerSquare {
    type Output = u64;

    fn index(&self, index: &dyn BoardPos) -> &Self::Output {
        &self[index.idx() as usize]
    }
}

impl IndexMut<&dyn BoardPos> for U64PerSquare {
    fn index_mut(&mut self, index: &dyn BoardPos) -> &mut Self::Output {
        &mut self[index.idx() as usize]
    }
}

pub type ColoredU64PerSquare = [U64PerSquare; 2];

impl CustomDefault for ColoredU64PerSquare {
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
