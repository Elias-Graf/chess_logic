use crate::{bit_board, Piece};

pub fn assert_bit_boards_eq(left: u64, right: u64) {
    assert_eq!(
        left,
        right,
        "expected:\n{}\nto equal:\n{}",
        bit_board::display(left),
        bit_board::display(right)
    );
}

impl Piece {
    pub fn all_variants() -> [Piece; 6] {
        [
            Piece::Bishop,
            Piece::King,
            Piece::Knight,
            Piece::Pawn,
            Piece::Queen,
            Piece::Rook,
        ]
    }
}
