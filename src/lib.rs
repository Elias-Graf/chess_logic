pub mod board;
pub mod display_board;
pub mod fen;

mod piece;

pub use board::Board;
pub use piece::Piece;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub fn get_opposing(&self) -> Color {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }
}
