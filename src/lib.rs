pub mod board;
pub mod info_board;

mod piece;

pub use board::Board;
pub use info_board::InfoBoard;
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
