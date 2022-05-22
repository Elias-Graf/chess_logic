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

impl Player {
    pub fn get_opposing(player: &Player) -> Player {
        match player {
            Player::You => Player::Opponent,
            Player::Opponent => Player::You,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}
