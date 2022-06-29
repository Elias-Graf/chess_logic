use std::ops::Index;

use crate::{bit_board::SetBitsIter, Board, Color, Piece};

use Color::*;
use Piece::*;

/// Contains the material values of all pieces.
pub const MAT_VAL: MatValTbl = MatValTbl([
    3,       /* Bishop */
    i8::MAX, /* King */
    3,       /* Knight */
    1,       /* Pawn */
    9,       /* Queen */
    5,       /* Rook */
]);

/// Scores the board so it can later be used in a min-max algorithm.
///
/// [`Black`] received pieces decrease the overall score, while [`White`] increases
/// it.
pub fn evaluate(board: &Board) -> i32 {
    let mut val = 0;

    for color in [Black, White] {
        for (mat_val, bit_board) in [
            (MAT_VAL[Bishop], board.bishops[color]),
            (MAT_VAL[King], board.king[color]),
            (MAT_VAL[Pawn], board.pawns[color]),
            (MAT_VAL[Queen], board.queens[color]),
            (MAT_VAL[Rook], board.rooks[color]),
        ] {
            for _ in SetBitsIter(bit_board) {
                if color == White {
                    val += mat_val as i32;
                } else {
                    val -= mat_val as i32;
                }
            }
        }
    }

    val
}

pub struct MatValTbl([i8; 6]);

impl Index<Piece> for MatValTbl {
    type Output = i8;

    fn index(&self, index: Piece) -> &Self::Output {
        &self.0[index as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::fen::Fen;

    use super::*;

    #[test]
    fn bishop() {
        let board = Board::from_fen("8/8/8/8/8/8/8/2B2B2 w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), 6);
    }

    #[test]
    fn king() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), i8::MAX as i32);
    }

    #[test]
    fn pawn() {
        let board = Board::from_fen("8/8/8/8/8/8/PPPPPPPP/8 w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), 8);
    }

    #[test]
    fn queen() {
        let board = Board::from_fen("8/8/8/8/8/8/8/3Q4 w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), 9);
    }

    #[test]
    fn rook() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R6R w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), 10);
    }

    #[test]
    fn initial_position() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 0").unwrap();

        assert_eq!(evaluate(&board), 0);
    }
}
