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

impl Fen for Board {
    fn get_fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;

        for idx in 0..Board::SIZE {
            match self.get(&idx) {
                None => empty_count += 1,
                Some(ins) => {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }

                    fen += &Fen::get_fen(&ins);
                }
            }

            let is_end_of_rank = idx % Self::HEIGHT == Self::WIDTH - 1;
            let last_separator = idx == Board::SIZE - 1;

            if is_end_of_rank {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }

                if !last_separator {
                    fen.push('/');
                }
            }
        }

        fen
    }

    fn from_fen(fen: &str) -> Result<Board, String> {
        // Currently only the actual position of the pieces is supported.
        let fen = fen.split(' ').collect::<Vec<_>>()[0];
        let mut board = Board::new_empty();
        let mut idx: usize = 0;

        for c in fen.chars() {
            if c == '/' {
                continue;
            }

            if let Some(empty_squares) = c.to_digit(10) {
                idx += empty_squares as usize;

                continue;
            }

            let ins: PieceInstance = Fen::from_fen(&c.to_string())?;

            board.set(&idx, ins.color, ins.piece);
            idx += 1;
        }

        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use crate::{bit_board, square::Square};

    use super::*;

    #[test]
    fn starting_formation() {
        let board = Board::new_with_standard_formation();

        assert_eq!(
            board.get_fen(),
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR"
        );
    }

    #[test]
    fn not_starting_formation() {
        let mut board = Board::new_empty();
        board.set(&Square::D8, Color::White, Piece::Bishop);
        board.set(&Square::A7, Color::Black, Piece::King);
        board.set(&Square::H7, Color::White, Piece::Pawn);
        board.set(&Square::E6, Color::White, Piece::Knight);
        board.set(&Square::G6, Color::Black, Piece::Pawn);
        board.set(&Square::A5, Color::White, Piece::King);
        board.set(&Square::B4, Color::White, Piece::Pawn);
        board.set(&Square::F4, Color::White, Piece::Pawn);
        board.set(&Square::G4, Color::White, Piece::Bishop);
        board.set(&Square::H4, Color::Black, Piece::Pawn);
        board.set(&Square::F3, Color::White, Piece::Pawn);
        board.set(&Square::H3, Color::Black, Piece::Rook);
        board.set(&Square::A2, Color::White, Piece::Rook);
        board.set(&Square::E2, Color::White, Piece::Pawn);
        board.set(&Square::G2, Color::White, Piece::Pawn);
        board.set(&Square::H2, Color::Black, Piece::Pawn);

        assert_eq!(board.get_fen(), "3B4/k6P/4N1p1/K7/1P3PBp/5P1r/R3P1Pp/8");
    }
}
