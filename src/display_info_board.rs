use std::fmt::{self, Display};

use crate::{info_board::PosInfo, Board, Color, InfoBoard, Piece};

impl Display for InfoBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\u{001b}[0m";

        let mut val = "\n".to_owned();

        for (idx, _) in (&self.poses).iter().enumerate() {
            if idx % Board::WIDTH == 0 {
                val.push('\n');
            }

            let bg_color = get_bg_color_of(idx);
            let fg_color = get_fg_color_of(idx, self);
            let piece_symbol = get_piece_symbol_at(idx, self);

            val.push_str(&format!(
                "{}{}{: ^4}{}",
                fg_color, bg_color, piece_symbol, RESET,
            ));
        }

        write!(f, "{}", val)
    }
}

fn get_bg_color_of(idx: usize) -> &'static str {
    const BG_BLACK: &str = "\u{001b}[48;5;126m";
    const BG_WHITE: &str = "\u{001b}[48;5;145m";

    if (idx + (idx / Board::WIDTH)) % 2 == 0 {
        return BG_WHITE;
    }

    BG_BLACK
}

fn get_fg_color_of(idx: usize, board: &InfoBoard) -> &str {
    const FG_BLACK: &str = "\u{001b}[38;5;0m";
    const FG_WHITE: &str = "\u{001b}[38;5;15m";

    match board.get(idx) {
        PosInfo::Piece(ins) | PosInfo::PieceHit(ins) => match board.get_color_of(&ins.player) {
            Color::Black => FG_BLACK,
            Color::White => FG_WHITE,
        },
        _ => "",
    }
}

fn get_piece_symbol_at(idx: usize, board: &InfoBoard) -> String {
    match board.get(idx) {
        PosInfo::Move => "*".to_owned(),
        PosInfo::None => "".to_owned(),
        PosInfo::Piece(i) => Piece::get_symbol(&i.piece).to_owned(),
        PosInfo::PieceHit(i) => format!("*{}", Piece::get_symbol(&i.piece)),
    }
}
