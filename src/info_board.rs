use std::fmt;

use crate::{board::PieceInstance, Board, Color, Piece, Player};

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
    pub board: [PosInfo; Board::SIZE as usize],
    pub moves: Vec<(i8, i8)>,
    pub opponent_color: Color,
    pub you_color: Color,
}

impl InfoBoard {
    #[deprecated(note = "use `get_by_idx` instead")]
    pub fn get(&self, x: i8, y: i8) -> &PosInfo {
        assert!(
            self.is_in_bounds(x, y),
            "cannot get a position outside the board ({}/{})",
            x,
            y,
        );

        self.get_by_idx(Self::x_y_to_idx(x, y))
    }

    pub fn get_by_idx(&self, idx: usize) -> &PosInfo {
        &self.board[idx]
    }

    fn get_color_of(&self, player: &Player) -> &Color {
        match player {
            Player::You => &self.you_color,
            Player::Opponent => &self.opponent_color,
        }
    }

    fn get_display_fg_color_for(&self, pos: &PosInfo) -> &str {
        const FG_BLACK: &str = "\u{001b}[38;5;0m";
        const FG_WHITE: &str = "\u{001b}[38;5;15m";

        match pos {
            PosInfo::Piece(ins) | PosInfo::PieceHit(ins) => match self.get_color_of(&ins.player) {
                Color::Black => FG_BLACK,
                Color::White => FG_WHITE,
            },
            _ => "",
        }
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

    fn get_display_symbol_for(&self, pos: &PosInfo) -> String {
        match pos {
            PosInfo::Move => "*".to_owned(),
            PosInfo::None => "".to_owned(),
            PosInfo::Piece(i) => Piece::get_symbol(&i.piece),
            PosInfo::PieceHit(i) => format!("*{}", Piece::get_symbol(&i.piece)),
        }
    }

    /// Checks if a position is within the bounds of the board.
    ///
    /// The variable might safely be cased to [`usize`] after `true` was returned
    /// from this function.
    pub fn is_in_bounds(&self, x: i8, y: i8) -> bool {
        x >= 0 && (x as u8) < Board::WIDTH && y >= 0 && (y as u8) < Board::HEIGHT
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        const INIT: PosInfo = PosInfo::None;
        Self {
            board: [INIT; Board::SIZE as usize],
            moves: Vec::new(),
            opponent_color,
            you_color,
        }
    }

    #[deprecated(note = "use `set_by_idx` instead")]
    pub fn set(&mut self, x: i8, y: i8, info: PosInfo) {
        assert!(
            self.is_in_bounds(x, y),
            "cannot set a position outside the board ({}/{})",
            x,
            y,
        );

        self.set_by_idx(Self::x_y_to_idx(x, y), info);
    }

    pub fn set_by_idx(&mut self, idx: usize, info: PosInfo) {
        if matches!(info, PosInfo::Move | PosInfo::PieceHit(_)) {
            let x = idx % Board::HEIGHT as usize;
            let y = idx / Board::HEIGHT as usize;

            self.moves.push((x as i8, y as i8));
        }

        self.board[idx] = info;
    }

    pub fn take(&mut self, idx: usize) -> PosInfo {
        std::mem::replace(&mut self.board[idx], PosInfo::None)
    }

    fn x_y_to_idx(x: i8, y: i8) -> usize {
        y as usize * Board::HEIGHT as usize + x as usize
    }
}

impl From<&Board> for InfoBoard {
    fn from(board: &Board) -> Self {
        let you_color = board.get_color_of_player(&Player::You).clone();
        let opponent_color = board.get_color_of_player(&Player::Opponent).clone();
        let mut info_board = InfoBoard::new(you_color, opponent_color);

        for (idx, pos) in board.board.as_ref().iter().enumerate() {
            if let Some(ins) = pos {
                info_board.set_by_idx(idx, PosInfo::Piece(ins.clone()));
            }
        }

        info_board
    }
}

impl fmt::Display for InfoBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\u{001b}[0m";

        let mut val = "\n".to_owned();

        for y in 0..Board::HEIGHT as i8 {
            for x in 0..Board::WIDTH as i8 {
                let idx = (y * Board::HEIGHT as i8 + x) as usize;
                let pos = self.get_by_idx(idx);
                let bg_color = self.get_display_square_bg_color(x, y);
                let fg_color = self.get_display_fg_color_for(pos);
                let piece_symbol = self.get_display_symbol_for(pos);

                val.push_str(&format!(
                    "{}{}{: ^4}{}",
                    fg_color, bg_color, piece_symbol, RESET
                ));
            }

            val.push('\n');
        }

        write!(f, "{}", val)
    }
}
