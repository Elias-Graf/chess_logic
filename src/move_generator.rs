use std::fmt::{Debug, Display};

use crate::{bit_board, piece::get_pawn_attacks_for, Board, Color, Piece, Square};

pub fn all_moves(board: &Board) -> Vec<Move> {
    let all_pieces = board.all_pieces();

    let mut moves = Vec::new();

    for src_idx in SetBitsIter(board.pawns[Color::White]) {
        let dst_idx = src_idx - bit_board::NORTH as usize;

        if dst_idx <= 7 {
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Bishop));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Knight));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Queen));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Rook));
        } else {
            if !bit_board::is_bit_set(all_pieces, dst_idx) {
                moves.push(Move::new(src_idx, dst_idx, Piece::Pawn));
            }
        }

        if src_idx >= 48 && src_idx <= 55 {
            if !bit_board::is_bit_set(all_pieces, dst_idx) {
                let dst_idx = src_idx - bit_board::NORTH as usize * 2;

                if !bit_board::is_bit_set(all_pieces, dst_idx) {
                    moves.push(Move::new(src_idx, dst_idx, Piece::Pawn));
                }
            }
        }

        let attacks = get_pawn_attacks_for(src_idx, &Color::White) & board.pawns[Color::Black];

        for attack in SetBitsIter(attacks) {
            moves.push(Move::new(src_idx, attack, Piece::Pawn));
        }
    }

    for src_idx in SetBitsIter(board.pawns[Color::Black]) {
        let dst_idx = src_idx + bit_board::SOUTH as usize;

        if dst_idx >= 56 && dst_idx <= 63 {
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Bishop));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Knight));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Queen));
            moves.push(Move::new_prom(src_idx, dst_idx, Piece::Rook));
        } else {
            if !bit_board::is_bit_set(all_pieces, dst_idx) {
                moves.push(Move::new(src_idx, dst_idx, Piece::Pawn));
            }
        }

        if src_idx >= 7 && src_idx <= 15 {
            if !bit_board::is_bit_set(all_pieces, dst_idx) {
                let dst_idx = src_idx + bit_board::SOUTH as usize * 2;

                if !bit_board::is_bit_set(all_pieces, dst_idx) {
                    moves.push(Move::new(src_idx, dst_idx, Piece::Pawn));
                }
            }
        }

        let attacks = get_pawn_attacks_for(src_idx, &Color::Black) & board.pawns[Color::White];

        for attack in SetBitsIter(attacks) {
            moves.push(Move::new(src_idx, attack, Piece::Pawn));
        }
    }

    moves
}

struct SetBitsIter(u64);

impl Iterator for SetBitsIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = bit_board::get_first_set_bit(self.0) {
            bit_board::clear_bit(&mut self.0, i);

            return Some(i as usize);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;

    use crate::fen::Fen;

    use super::*;

    use Square::*;

    #[test]
    fn white_pawn_push() {
        for (src, dst) in [(A3, A4), (B3, B4)] {
            let mut board = Board::new_empty();
            board.set(src, Color::White, Piece::Pawn);

            assert_moves_eq(&all_moves(&board), &vec![Move::new(src, dst, Piece::Pawn)]);
        }
    }

    #[test]
    fn black_pawns_push() {
        for (src, dst) in [(A6, A5), (B6, B5)] {
            let mut board = Board::new_empty();
            board.set(src, Color::Black, Piece::Pawn);

            assert_moves_eq(&all_moves(&board), &vec![Move::new(src, dst, Piece::Pawn)]);
        }
    }

    #[test]
    fn white_pawn_double_push() {
        let mut board = Board::new_empty();
        board.set(A2, Color::White, Piece::Pawn);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(A2, A3, Piece::Pawn),
                Move::new(A2, A4, Piece::Pawn),
            ],
        );

        let mut board = Board::new_empty();
        board.set(B2, Color::White, Piece::Pawn);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(B2, B3, Piece::Pawn),
                Move::new(B2, B4, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn black_pawn_double_push() {
        let mut board = Board::new_empty();
        board.set(A7, Color::Black, Piece::Pawn);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(A7, A6, Piece::Pawn),
                Move::new(A7, A5, Piece::Pawn),
            ],
        );

        let mut board = Board::new_empty();
        board.set(B7, Color::Black, Piece::Pawn);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(B7, B6, Piece::Pawn),
                Move::new(B7, B5, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn white_pawns_promote() {
        for i in 8..16usize {
            let mut board = Board::new_empty();
            board.set(i, Color::White, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(i, i - bit_board::NORTH as usize, Piece::Bishop),
                    Move::new_prom(i, i - bit_board::NORTH as usize, Piece::Knight),
                    Move::new_prom(i, i - bit_board::NORTH as usize, Piece::Queen),
                    Move::new_prom(i, i - bit_board::NORTH as usize, Piece::Rook),
                ],
            );
        }
    }

    #[test]
    fn black_pawns_promote() {
        for i in 48..55usize {
            let mut board = Board::new_empty();
            board.set(i, Color::Black, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Bishop),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Knight),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Queen),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Rook),
                ],
            );
        }
    }

    #[test]
    fn pawn_pushes_blocked() {
        let board = Board::from_fen("8/5p1p/7P/5p2/2P5/p7/P1P5/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(C4, C5, Piece::Pawn),
                Move::new(C2, C3, Piece::Pawn),
                Move::new(F7, F6, Piece::Pawn),
                Move::new(F5, F4, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn white_pawns_capture() {
        let board = Board::from_fen("8/8/8/8/p1p5/1P7/8/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(A4, A3, Piece::Pawn),
                Move::new(A4, B3, Piece::Pawn),
                Move::new(B3, A4, Piece::Pawn),
                Move::new(B3, B4, Piece::Pawn),
                Move::new(B3, C4, Piece::Pawn),
                Move::new(C4, B3, Piece::Pawn),
                Move::new(C4, C3, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn black_pawns_capture() {
        let board = Board::from_fen("8/8/1p6/P1P5/8/8/8/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B6, B5, Piece::Pawn),
                Move::new(B6, A5, Piece::Pawn),
                Move::new(B6, C5, Piece::Pawn),
                Move::new(A5, A6, Piece::Pawn),
                Move::new(A5, B6, Piece::Pawn),
                Move::new(C5, B6, Piece::Pawn),
                Move::new(C5, C6, Piece::Pawn),
            ],
        );
    }

    fn assert_moves_eq(left: &[Move], right: &[Move]) {
        let mut left = left.to_vec();
        left.sort_by(display_value);
        let mut right = right.to_vec();
        right.sort_by(display_value);

        assert_eq!(
            left,
            right,
            "\nexpected:\n\t{}\nto equal:\n\t{}\n",
            display(&left),
            display(&right),
        );

        fn display(moves: &[Move]) -> String {
            let display_val = moves
                .iter()
                .map(|m| format!("{}", m))
                .collect::<Vec<_>>()
                .join(", ");

            if display_val.chars().count() == 0 {
                return "<no moves>".to_owned();
            }

            display_val
        }

        fn display_value(a: &Move, b: &Move) -> Ordering {
            format!("{}", a).cmp(&format!("{}", b))
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub struct Move {
    destination: usize,
    is_castle: bool,
    is_double_push: bool,
    is_en_passant: bool,
    piece: Piece,
    promote_to: Option<Piece>,
    source: usize,
}

impl Move {
    pub fn destination(&self) -> usize {
        self.destination
    }

    pub fn is_castle(&self) -> bool {
        self.is_castle
    }

    pub fn is_double_push(&self) -> bool {
        self.is_double_push
    }

    pub fn is_en_passant(&self) -> bool {
        self.is_en_passant
    }

    pub fn new(src: impl Into<usize>, dst: impl Into<usize>, piece: Piece) -> Self {
        Self {
            destination: dst.into(),
            is_castle: false,
            is_double_push: false,
            is_en_passant: false,
            piece,
            promote_to: None,
            source: src.into(),
        }
    }

    pub fn new_prom(src: impl Into<usize>, dst: impl Into<usize>, promote_to: Piece) -> Self {
        Self {
            destination: dst.into(),
            is_castle: false,
            is_double_push: false,
            is_en_passant: false,
            piece: Piece::Pawn,
            promote_to: Some(promote_to),
            source: src.into(),
        }
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn promote_to(&self) -> Option<Piece> {
        self.promote_to
    }

    pub fn set_is_castle(&mut self, val: bool) {
        self.is_castle = val;
    }

    pub fn set_is_double_push(&mut self, val: bool) {
        self.is_double_push = val;
    }

    pub fn set_is_en_passant(&mut self, val: bool) {
        self.is_en_passant = val;
    }

    pub fn set_piece(&mut self, val: Piece) {
        self.piece = val;
    }

    pub fn set_promote_to(&mut self, val: Option<Piece>) {
        self.promote_to = val;
    }

    pub fn set_source(&mut self, val: usize) {
        self.source = val;
    }

    pub fn set_target(&mut self, val: usize) {
        self.destination = val;
    }

    pub fn source(&self) -> usize {
        self.source
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: {:?}->{:?}",
            self.piece,
            Square::try_from(self.source()).unwrap(),
            Square::try_from(self.destination()).unwrap(),
        )
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}
