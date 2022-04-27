use crate::board::PieceInstance;

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
pub struct InfoBoard {
    board: Vec<Vec<PosInfo>>,
    height: i8,
    width: i8,
}

impl InfoBoard {
    pub fn get(&self, x: i8, y: i8) -> &PosInfo {
        assert!(
            self.is_in_bounds(x, y),
            "cannot get a position outside the board ({}/{})",
            x,
            y,
        );

        &self.board[y as usize][x as usize]
    }

    pub fn height(&self) -> i8 {
        self.height
    }

    /// Checks if a position is within the bounds of the board.
    ///
    /// The variable might safely be cased to [`usize`] after `true` was returned
    /// from this function.
    pub fn is_in_bounds(&self, x: i8, y: i8) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn new() -> Self {
        let height = 8;
        let width = 8;

        Self {
            board: vec![vec![PosInfo::None; width]; height],
            height: height as i8,
            width: width as i8,
        }
    }

    pub fn set(&mut self, x: i8, y: i8, info: PosInfo) {
        assert!(
            self.is_in_bounds(x, y),
            "cannot set a position outside the board ({}/{})",
            x,
            y,
        );

        self.board[y as usize][x as usize] = info;
    }

    pub fn width(&self) -> i8 {
        self.width
    }
}
