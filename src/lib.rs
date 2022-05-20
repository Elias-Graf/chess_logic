pub mod board;

mod display_board;
mod piece;

pub use board::Board;
pub use piece::Piece;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Player {
    You,
    Opponent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}
