use std::fmt::{self, Display};

use crate::Board;

pub const RESET_ANSI: &str = "\u{001b}[0m";

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut val = "\n".to_owned();

        for i in 0..Board::SIZE {
            if i % Board::WIDTH == 0 {
                val.push('\n');
            }

            let bg_color = get_bg_color_of(i);
            let fg_color = get_fg_color_of(i, self);
            let piece_symbol = get_piece_symbol_at(i, self);

            val.push_str(&format!(
                "{}{}{: ^4}{}",
                fg_color, bg_color, piece_symbol, RESET_ANSI
            ));
        }

        write!(f, "{}", val)
    }
}

pub const fn get_bg_color_of(idx: usize) -> &'static str {
    const BG_BLACK: &str = "\u{001b}[48;5;126m";
    const BG_WHITE: &str = "\u{001b}[48;5;145m";

    if (idx + (idx / Board::WIDTH)) % 2 == 0 {
        return BG_WHITE;
    }

    BG_BLACK
}

pub fn get_fg_color_of(idx: usize, board: &Board) -> &str {
    const FG_BLACK: &str = "\u{001b}[38;5;0m";
    const FG_WHITE: &str = "\u{001b}[38;5;15m";

    match board.get(&idx) {
        Some(ins) => match ins.color {
            crate::Color::Black => FG_BLACK,
            crate::Color::White => FG_WHITE,
        },
        None => "",
    }
}

pub fn get_piece_symbol_at(idx: usize, board: &Board) -> &str {
    match board.get(&idx) {
        Some(ins) => ins.piece.get_symbol(),
        None => "",
    }
}
