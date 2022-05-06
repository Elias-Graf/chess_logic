use std::fmt;

use crate::{board::PieceInstance, Board, Piece};

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
#[derive(Debug)]
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

    fn get_display_square_bg_color(&self, x: i8, y: i8) -> &str {
        const BG_BLACK: &str = "\u{001b}[48;5;126m";
        const BG_WHITE: &str = "\u{001b}[48;5;145m";

        let is_even_row = y % 2 == 0;
        let is_even_column = x % 2 == 0;

        if is_even_row && is_even_column || !is_even_row && !is_even_column {
            return BG_WHITE;
        }

        BG_BLACK
    }

    fn get_display_symbol_for_piece_instance(&self, info: &PosInfo) -> String {
        match info {
            PosInfo::Move => "*".to_owned(),
            PosInfo::None => "".to_owned(),
            PosInfo::Piece(i) => Piece::get_symbol(&i.piece),
            PosInfo::PieceHit(i) => format!("*{}", Piece::get_symbol(&i.piece)),
        }
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

impl From<&Board> for InfoBoard {
    fn from(board: &Board) -> Self {
        let mut info_board = InfoBoard::new();

        for y in 0..board.height() {
            for x in 0..board.width() {
                if let Some(ins) = board.get(x, y) {
                    info_board.set(x, y, PosInfo::Piece(ins.clone()));
                }
            }
        }

        info_board
    }
}

impl fmt::Display for InfoBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\u{001b}[0m";
        const FG_BLACK: &str = "\u{001b}[38;5;0m";

        let mut val = "\n".to_owned();

        for y in 0..self.height {
            for x in 0..self.height {
                let piece_ins = self.get(x, y);
                let bg_color = self.get_display_square_bg_color(x, y);
                let piece_symbol = self.get_display_symbol_for_piece_instance(piece_ins);

                val.push_str(&format!(
                    "{}{}{: ^4}{}",
                    FG_BLACK, bg_color, piece_symbol, RESET
                ));
            }

            val.push('\n');
        }

        write!(f, "{}", val)
    }
}
