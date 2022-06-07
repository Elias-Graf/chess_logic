use crate::{board::PieceInstance, square::Square, Board, Color, Piece};

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

#[rustfmt::skip]
pub const FEN_SQUARE_SYMBOL_LOOKUP: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
];

impl Fen for Square {
    fn get_fen(&self) -> String {
        FEN_SQUARE_SYMBOL_LOOKUP[*self as usize].to_owned()
    }

    fn from_fen(fen: &str) -> Result<Self, String> {
        let idx = FEN_SQUARE_SYMBOL_LOOKUP
            .iter()
            .position(|sym| sym == &fen)
            .ok_or_else(|| format!("could not identify square with symbol '{}'", fen))?;

        idx.try_into()
    }
}

impl Fen for Board {
    fn get_fen(&self) -> String {
        return format!(
            "{} {} {} {} {} {}",
            pieces(self),
            side_to_move(self),
            castling_abilities(self),
            en_passant_target(self),
            halve_move_clock(),
            full_move_counter()
        );

        fn pieces(board: &Board) -> String {
            let mut val = String::new();

            let mut empty_count = 0;
            for idx in 0..Board::SIZE {
                match board.get(idx) {
                    Some(ins) => {
                        if empty_count > 0 {
                            val.push_str(&empty_count.to_string());
                            empty_count = 0;
                        }

                        val += &ins.get_fen();
                    }
                    None => empty_count += 1,
                }

                let is_end_of_rank = idx % Board::HEIGHT == Board::WIDTH - 1;
                let last_separator = idx == Board::SIZE - 1;

                if is_end_of_rank {
                    if empty_count > 0 {
                        val += &empty_count.to_string();
                        empty_count = 0;
                    }

                    if !last_separator {
                        val.push('/');
                    }
                }
            }

            val
        }

        fn side_to_move(board: &Board) -> String {
            if board.is_whites_turn { "w" } else { "b" }.to_owned()
        }

        fn castling_abilities(board: &Board) -> String {
            let mut val = String::new();

            if board.can_white_castle_king_side {
                val.push('K');
            }
            if board.can_white_castle_queen_side {
                val.push('Q');
            }
            if board.can_black_castle_king_side {
                val.push('k');
            }
            if board.can_black_castle_queen_side {
                val.push('q');
            }

            if val.len() == 0 {
                val = "-".to_owned();
            }

            val
        }

        fn en_passant_target(board: &Board) -> String {
            if let Some(idx) = board.en_passant_target_idx {
                return Square::try_from(idx)
                    .unwrap_or_else(|err| {
                        panic!("couldn't convert en passant index to square: {}", err)
                    })
                    .get_fen();
            }

            "-".to_owned()
        }

        fn halve_move_clock() -> String {
            // (At least right now) this feature is not relevant and we just return
            // "0" to get a valid FEN.
            "0".to_owned()
        }

        fn full_move_counter() -> String {
            // (At least right now) this feature is not relevant and we just return
            // "0" to get a valid FEN.
            "0".to_owned()
        }
    }

    fn from_fen(fen: &str) -> Result<Board, String> {
        let fen: Vec<_> = fen.split(' ').collect();

        let mut board = Board::new_empty();

        pieces(fen[0], &mut board)?;
        side_to_move(fen[1], &mut board)?;
        castling_rights(fen[2], &mut board);
        en_passant_pos(fen[3], &mut board)?;

        return Ok(board);

        fn pieces(pieces: &str, board: &mut Board) -> Result<(), String> {
            let mut idx: usize = 0;

            for c in pieces.chars() {
                if c == '/' {
                    continue;
                }

                if let Some(empty_squares) = c.to_digit(10) {
                    idx += empty_squares as usize;

                    continue;
                }

                let ins: PieceInstance = Fen::from_fen(&c.to_string())?;

                board.set(idx, ins.color, ins.piece);
                idx += 1;
            }

            Ok(())
        }

        fn side_to_move(side_to_move: &str, board: &mut Board) -> Result<(), String> {
            board.is_whites_turn = match side_to_move {
                "b" => false,
                "w" => true,
                _ => {
                    return Err(format!(
                        "failed to parse whose turn it is, expected 'b' or 'w' but received {}",
                        side_to_move
                    ))
                }
            };

            Ok(())
        }

        fn castling_rights(castling_rights: &str, board: &mut Board) {
            if castling_rights.contains('K') {
                board.can_white_castle_king_side = true;
            }
            if castling_rights.contains('Q') {
                board.can_white_castle_queen_side = true;
            }
            if castling_rights.contains('k') {
                board.can_black_castle_king_side = true;
            }
            if castling_rights.contains('q') {
                board.can_black_castle_queen_side = true;
            }
        }

        fn en_passant_pos(en_passant_pos: &str, board: &mut Board) -> Result<(), String> {
            if en_passant_pos != "-" {
                board.en_passant_target_idx = Some(Square::from_fen(en_passant_pos)?.into());
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    use crate::square::Square;

    #[test]
    fn starting_formation() {
        let truth = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 0";

        let board = Board::new_with_standard_formation();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn not_starting_formation() {
        use Square::*;

        let truth = "3B4/k6P/4N1p1/K7/1P3PBp/5P1r/R3P1Pp/8 w - - 0 0";

        let mut board = Board::new_empty();
        board.set(D8, Color::White, Piece::Bishop);
        board.set(A7, Color::Black, Piece::King);
        board.set(H7, Color::White, Piece::Pawn);
        board.set(E6, Color::White, Piece::Knight);
        board.set(G6, Color::Black, Piece::Pawn);
        board.set(A5, Color::White, Piece::King);
        board.set(B4, Color::White, Piece::Pawn);
        board.set(F4, Color::White, Piece::Pawn);
        board.set(G4, Color::White, Piece::Bishop);
        board.set(H4, Color::Black, Piece::Pawn);
        board.set(F3, Color::White, Piece::Pawn);
        board.set(H3, Color::Black, Piece::Rook);
        board.set(A2, Color::White, Piece::Rook);
        board.set(E2, Color::White, Piece::Pawn);
        board.set(G2, Color::White, Piece::Pawn);
        board.set(H2, Color::Black, Piece::Pawn);

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn white_to_move() {
        let truth = "8/8/8/8/8/8/8/8 w - - 0 0";

        let board = Board::new_empty();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn black_to_move() {
        let truth = "8/8/8/8/8/8/8/8 b - - 0 0";

        let mut board = Board::new_empty();
        board.is_whites_turn = false;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn castle_none() {
        let truth = "8/8/8/8/8/8/8/8 w - - 0 0";

        let board = Board::new_empty();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/8 w - - 0 0").unwrap());
    }

    #[test]
    fn castle_white_queen_side() {
        let truth = "8/8/8/8/8/8/8/8 w Q - 0 0";

        let mut board = Board::new_empty();
        board.can_white_castle_queen_side = true;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen("8/8/8/8/8/8/8/8 w Q - 0 0").unwrap());
    }

    #[test]
    fn castle_white_king_side() {
        let truth = "8/8/8/8/8/8/8/8 w K - 0 0";

        let mut board = Board::new_empty();
        board.can_white_castle_king_side = true;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn castle_black_queen_side() {
        let truth = "8/8/8/8/8/8/8/8 w q - 0 0";

        let mut board = Board::new_empty();
        board.can_black_castle_queen_side = true;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn castle_black_king_side() {
        let truth = "8/8/8/8/8/8/8/8 w k - 0 0";

        let mut board = Board::new_empty();
        board.can_black_castle_king_side = true;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn castle_all_sides() {
        let truth = "8/8/8/8/8/8/8/8 w KQkq - 0 0";

        let mut board = Board::new_empty();
        board.can_white_castle_king_side = true;
        board.can_white_castle_queen_side = true;
        board.can_black_castle_king_side = true;
        board.can_black_castle_queen_side = true;

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn en_passant_none() {
        let truth = "8/8/8/8/8/8/8/8 w - - 0 0";

        let board = Board::new_empty();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn en_passant_e4() {
        let truth = "8/8/8/8/8/8/8/8 w - e4 0 0";

        let mut board = Board::new_empty();
        board.en_passant_target_idx = Some(Square::E4.into());

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn en_passant_c5() {
        let truth = "8/8/8/8/8/8/8/8 w - c5 0 0";

        let mut board = Board::new_empty();
        board.en_passant_target_idx = Some(Square::C5.into());

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn half_move_clock() {
        let truth = "8/8/8/8/8/8/8/8 w - - 0 0";

        // Currently the half move clock is not relevant in this engine, and thus
        // always emitted as 0.
        let board = Board::new_empty();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }

    #[test]
    fn full_move_counter() {
        let truth = "8/8/8/8/8/8/8/8 w - - 0 0";

        // Currently the full move container is not relevant in this engine, and
        // thus always emitted as 0.
        let board = Board::new_empty();

        assert_eq!(board.get_fen(), truth);
        assert_eq!(board, Board::from_fen(truth).unwrap());
    }
}
