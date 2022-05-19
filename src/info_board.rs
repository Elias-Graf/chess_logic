use crate::{
    board::{Move, PieceInstance},
    Board, Color, Player,
};

/// Information on individual positions on the board.
#[derive(Clone, Debug)]
pub enum PosInfo {
    /// Move of a piece. Much like [`PosInfo::PieceHit`], but on an "empty"
    /// position.
    Move,
    /// Nothing is on this position.
    None,
    /// A piece which could not be hit (taken) ([`PosInfo::PieceHit`]) by
    /// another.
    Piece(PieceInstance),
    /// A piece that could be hit (taken) by another.
    PieceHit(PieceInstance),
}

/// Contains information (**no** logic) about each position on the board.
///
/// Can be useful for displaying it, figuring out what moves are valid, etc.
#[derive(Clone, Debug)]
pub struct InfoBoard {
    pub poses: [PosInfo; Board::SIZE as usize],
    pub moves: Vec<Move>,
    pub opponent_color: Color,
    pub you_color: Color,
}

impl InfoBoard {
    pub fn get(&self, idx: usize) -> &PosInfo {
        &self.poses[idx]
    }

    pub fn get_color_of(&self, player: &Player) -> &Color {
        match player {
            Player::You => &self.you_color,
            Player::Opponent => &self.opponent_color,
        }
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        const INIT: PosInfo = PosInfo::None;
        Self {
            poses: [INIT; Board::SIZE as usize],
            moves: Vec::new(),
            opponent_color,
            you_color,
        }
    }

    pub fn set(&mut self, idx: usize, info: PosInfo) {
        if matches!(info, PosInfo::Move | PosInfo::PieceHit(_)) {
            self.moves.push((usize::MAX, idx));
        }

        self.poses[idx] = info;
    }

    pub fn take(&mut self, idx: usize) -> PosInfo {
        std::mem::replace(&mut self.poses[idx], PosInfo::None)
    }
}

impl From<&Board> for InfoBoard {
    fn from(board: &Board) -> Self {
        let you_color = board.get_color_of_player(&Player::You).clone();
        let opponent_color = board.get_color_of_player(&Player::Opponent).clone();
        let mut info_board = InfoBoard::new(you_color, opponent_color);

        for (idx, pos) in board.poses.as_ref().iter().enumerate() {
            if let Some(ins) = pos {
                info_board.set(idx, PosInfo::Piece(ins.clone()));
            }
        }

        info_board
    }
}
