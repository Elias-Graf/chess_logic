pub mod board;
pub use board::Board;

mod piece;
pub use piece::Piece;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Player {
    You,
    Opponent,
}

#[derive(Clone, Debug)]
pub enum Color {
    Black,
    White,
}
