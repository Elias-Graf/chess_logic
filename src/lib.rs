pub mod board;
pub mod info_board;

mod piece;
mod display_board;
mod display_info_board;

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
