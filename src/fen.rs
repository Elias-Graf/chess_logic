use crate::{board::PieceInstance, Board, Color, Piece};

/// An interface to convert a playing board to and from a fen string.
///
/// For more information, visit: https://www.chess.com/terms/fen-chess
pub trait Fen: Sized {
    fn get_fen(&self) -> String;
    fn from_fen(fen: &str) -> Result<Self, String>;
}

impl Fen for PieceInstance {
    fn get_fen(&self) -> String {
        match (&self.color, &self.piece) {
            // TODO: It is currently assumed that you are white.
            // This should be changed as soon as color information is stored with
            // the piece.
            (Color::White, Piece::Bishop) => "B",
            (Color::White, Piece::King) => "K",
            (Color::White, Piece::Knight) => "N",
            (Color::White, Piece::Pawn) => "P",
            (Color::White, Piece::Queen) => "Q",
            (Color::White, Piece::Rook) => "R",
            (Color::Black, Piece::Bishop) => "b",
            (Color::Black, Piece::King) => "k",
            (Color::Black, Piece::Knight) => "n",
            (Color::Black, Piece::Pawn) => "p",
            (Color::Black, Piece::Queen) => "q",
            (Color::Black, Piece::Rook) => "r",
        }
        .to_owned()
    }

    fn from_fen(fen: &str) -> Result<PieceInstance, String> {
        Ok(match fen {
            // TODO: It is currently assumed that you are white.
            // This should be changed as soon as color information is stored with
            // the piece.
            "B" => PieceInstance::new(Color::White, Piece::Bishop),
            "K" => PieceInstance::new(Color::White, Piece::King),
            "N" => PieceInstance::new(Color::White, Piece::Knight),
            "P" => PieceInstance::new(Color::White, Piece::Pawn),
            "Q" => PieceInstance::new(Color::White, Piece::Queen),
            "R" => PieceInstance::new(Color::White, Piece::Rook),
            "b" => PieceInstance::new(Color::Black, Piece::Bishop),
            "k" => PieceInstance::new(Color::Black, Piece::King),
            "n" => PieceInstance::new(Color::Black, Piece::Knight),
            "p" => PieceInstance::new(Color::Black, Piece::Pawn),
            "q" => PieceInstance::new(Color::Black, Piece::Queen),
            "r" => PieceInstance::new(Color::Black, Piece::Rook),
            unknown => return Err(format!("cannot convert from '{}' to piece", unknown)),
        })
    }
}

impl Fen for Board {
    fn get_fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;

        for idx in 0..Board::SIZE {
            match self.get(idx) {
                None => empty_count += 1,
                Some(ins) => {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    fen += &Fen::get_fen(ins);
                }
            }

            let is_end_of_rank = idx % Self::HEIGHT == Self::WIDTH - 1;
            let last_separator = idx == Board::SIZE - 1;

            if is_end_of_rank && !last_separator {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen.push('/');
            }
        }

        fen
    }

    fn from_fen(fen: &str) -> Result<Board, String> {
        // Currently only the actual position of the pieces is supported.
        let fen = fen.split(' ').collect::<Vec<_>>()[0];
        let mut board = Board::new(Color::White, Color::Black);
        let mut idx: usize = 0;

        for c in fen.chars() {
            if c == '/' {
                continue;
            }

            if let Some(empty_squares) = c.to_digit(10) {
                idx += empty_squares as usize;

                continue;
            }

            let mut ins: PieceInstance = Fen::from_fen(&c.to_string())?;
            if ins.piece == Piece::Pawn {
                if ins.color == Color::White {
                    ins.was_moved = !(idx > 47 && idx < 56);
                } else {
                    ins.was_moved = !(idx > 7 && idx < 16);
                }
            }

            board.set(idx, Some(ins));
            idx += 1;
        }

        Ok(board)
    }
}

// pub fn fen(&self) -> String {
//     let mut fen = String::new();
//     let mut empty_count = 0;

//     for idx in 0..Board::SIZE {
//         match self.get(idx) {
//             Some(ins) => {
//                 if empty_count > 0 {
//                     fen.push_str(&empty_count.to_string());
//                     empty_count = 0;
//                 }

//                 let mut piece_symbol = match ins.piece {
//                     Piece::Bishop => "b",
//                     Piece::King => "k",
//                     Piece::Knight => "n",
//                     Piece::Pawn => "p",
//                     Piece::Queen => "q",
//                     Piece::Rook => "r",
//                 }
//                 .to_owned();

//                 if self.get_color_of_player(&ins.player) == &Color::White {
//                     piece_symbol = piece_symbol.to_uppercase();
//                 }

//                 fen.push_str(&piece_symbol);
//             }
//             None => {
//                 empty_count += 1;
//             }
//         };

//         let is_end_of_rank = idx % Self::HEIGHT == Self::WIDTH - 1;
//         let last_separator = idx == Board::SIZE - 1;

//         if is_end_of_rank && !last_separator {
//             if empty_count > 0 {
//                 fen.push_str(&empty_count.to_string());
//                 empty_count = 0;
//             }
//             fen.push('/');
//         }
//     }

//     fen
// }

// pub fn from_fen(fen: &str) -> Result<Board, String> {
//     let fen = fen.split(' ').collect::<Vec<_>>()[0];
//     let mut board = Board::new(Color::White, Color::Black);
//     let mut idx: usize = 0;

//     for c in fen.chars() {
//         if c == '/' {
//             continue;
//         }

//         if let Some(empty_squares) = c.to_digit(10) {
//             idx += empty_squares as usize;

//             continue;
//         }

//          let mut ins = match c {
//             'b' => PieceInstance::new(Player::Opponent, Piece::Bishop),
//             'k' => PieceInstance::new(Player::Opponent, Piece::King),
//             'n' => PieceInstance::new(Player::Opponent, Piece::Knight),
//             'p' => PieceInstance::new(Player::Opponent, Piece::Pawn),
//             'q' => PieceInstance::new(Player::Opponent, Piece::Queen),
//             'r' => PieceInstance::new(Player::Opponent, Piece::Rook),
//             'B' => PieceInstance::new(Player::You, Piece::Bishop),
//             'K' => PieceInstance::new(Player::You, Piece::King),
//             'N' => PieceInstance::new(Player::You, Piece::Knight),
//             'P' => PieceInstance::new(Player::You, Piece::Pawn),
//             'Q' => PieceInstance::new(Player::You, Piece::Queen),
//             'R' => PieceInstance::new(Player::You, Piece::Rook),
//             c => return Err(format!("unknown fen symbol '{}'", c)),
//         };

//         if ins.piece == Piece::Pawn {
//             if board.get_color_of_player(&ins.player) == &Color::White {
//                 ins.was_moved = !(idx > 47 && idx < 56);
//             } else {
//                 ins.was_moved = !(idx > 7 && idx < 16);
//             }
//         }

//         board.set(idx, Some(ins));
//         idx += 1;
//     }

//     Ok(board)
// }
