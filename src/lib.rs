pub mod bit_board;
pub mod board;
pub mod fen;
pub mod magic_bit_board;
pub mod move_generator;
pub mod piece;
pub mod square;
pub mod type_alias_default;

#[cfg(test)]
mod testing_utils;

pub use board::Board;
pub use piece::Piece;
pub use square::Square;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const fn opposing(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}
