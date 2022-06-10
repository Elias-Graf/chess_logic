use std::{
    fmt::{Debug, Display},
    ops::{ControlFlow, Sub},
};

use crate::{
    bit_board::{self, NORTH, SOUTH, WEST},
    board::BoardPos,
    piece::{self, get_pawn_attacks_for},
    Board,
    Color::{self, *},
    Piece, Square,
};

use Square::*;

pub fn all_moves(board: &Board) -> Vec<Move> {
    let all_pieces = board.all_pieces();
    let atk_color = match board.is_whites_turn {
        true => Color::White,
        false => Color::Black,
    };
    let opp_color = atk_color.opposing();

    let mut moves = Vec::new();

    add_pawn_moves(board, all_pieces, atk_color, opp_color, &mut moves);
    add_king_moves(board, all_pieces, opp_color, &mut moves);
    add_knight_moves(board, &mut moves);

    return moves;
}

fn add_pawn_moves(
    board: &Board,
    all_pieces: u64,
    atk_color: Color,
    opp_color: Color,
    moves: &mut Vec<Move>,
) {
    let (dir, can_do_double_push, is_prom): (_, fn(usize) -> bool, fn(usize) -> bool) =
        match atk_color {
            Black => (SOUTH as i8, can_black_do_dbl_push, is_black_prom),
            White => (-(NORTH as i8), can_white_do_dbl_push, is_white_prom),
        };

    let pawns = board.pawns[atk_color];

    for src_i in SetBitsIter(pawns) {
        let dst_i = (src_i as i8 + dir) as usize;

        if is_prom(dst_i) {
            // Promotions
            if !bit_board::is_bit_set(all_pieces, dst_i) {
                moves.push(Move::new_prom(src_i, dst_i, Piece::Bishop));
                moves.push(Move::new_prom(src_i, dst_i, Piece::Knight));
                moves.push(Move::new_prom(src_i, dst_i, Piece::Queen));
                moves.push(Move::new_prom(src_i, dst_i, Piece::Rook));
            }

            // Capturing promotions
            let captures = get_pawn_attacks_for(src_i, &atk_color) & board.pawns[opp_color];
            for capture in SetBitsIter(captures) {
                moves.push(Move::new_prom(src_i, capture, Piece::Bishop));
                moves.push(Move::new_prom(src_i, capture, Piece::Knight));
                moves.push(Move::new_prom(src_i, capture, Piece::Queen));
                moves.push(Move::new_prom(src_i, capture, Piece::Rook));
            }
        } else {
            // Push
            if !bit_board::is_bit_set(all_pieces, dst_i) {
                moves.push(Move::new(src_i, dst_i, Piece::Pawn));

                // Double push
                if can_do_double_push(src_i) {
                    let dst_idx = (src_i as i8 + dir * 2) as usize;

                    if !bit_board::is_bit_set(all_pieces, dst_idx) {
                        moves.push(Move::new(src_i, dst_idx, Piece::Pawn));
                    }
                }
            }

            // Captures
            let captures = get_pawn_attacks_for(src_i, &atk_color) & board.pawns[opp_color];
            for capture in SetBitsIter(captures) {
                moves.push(Move::new(src_i, capture, Piece::Pawn));
            }

            // En passant
            if let Some(en_passant_target_idx) = board.en_passant_target_idx {
                if bit_board::is_bit_set(
                    get_pawn_attacks_for(src_i, &atk_color),
                    en_passant_target_idx,
                ) {
                    moves.push(Move::new_en_pass(src_i, en_passant_target_idx));
                }
            }
        }
    }

    fn can_black_do_dbl_push(i: usize) -> bool {
        i > 7 && i < 15
    }

    fn can_white_do_dbl_push(i: usize) -> bool {
        i > 47 && i < 56
    }

    fn is_white_prom(i: usize) -> bool {
        i < 8
    }

    fn is_black_prom(i: usize) -> bool {
        i > 55 && i < 64
    }
}

fn add_king_moves(board: &Board, all_pieces: u64, opp_color: Color, moves: &mut Vec<Move>) {
    let mut castle = |required_clear_mask: u64, not_atk: &[Square], src: Square, dst: Square| {
        if bit_board::has_set_bits(all_pieces & required_clear_mask) {
            return;
        }

        if squares_attacked_by(not_atk, board, opp_color) {
            return;
        }

        moves.push(Move::new_castle(src, dst));
    };

    // TODO: Check if it's actually whites turn
    if board.can_white_castle_queen_side {
        castle(1008806316530991104, &[B1, C1, D1, E1], E1, C1);
    }
    if board.can_white_castle_king_side {
        castle(6917529027641081856, &[E1, F1, G1], E1, G1);
    }

    // TODO: Check if it's actually blacks turn
    if board.can_black_castle_queen_side {
        castle(14, &[B8, C8, D8, E8], E8, C8);
    }
    if board.can_black_castle_king_side {
        castle(96, &[E8, F8, G8], E8, G8);
    }
}

fn add_knight_moves(board: &Board, moves: &mut Vec<Move>) {
    if board.is_whites_turn {
        for src_i in SetBitsIter(board.knights[White]) {
            for dst_i in SetBitsIter(piece::get_knight_attacks_for(src_i)) {
                moves.push(Move::new(src_i, dst_i, Piece::Knight));
            }
        }
    } else {
        for src_i in SetBitsIter(board.knights[Black]) {
            for dst_i in SetBitsIter(piece::get_knight_attacks_for(src_i)) {
                moves.push(Move::new(src_i, dst_i, Piece::Knight));
            }
        }
    }
}

fn squares_attacked_by(squares: &[Square], board: &Board, color: Color) -> bool {
    for square in squares {
        if board.is_pos_attacked_by(*square, &color) {
            return true;
        }
    }

    false
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

    use Piece::*;

    #[test]
    fn white_pawn_push() {
        for (src, dst) in [(A3, A4), (B3, B4)] {
            let mut board = Board::new_empty();
            board.set(src, Color::White, Piece::Pawn);

            assert_moves_eq(&all_moves(&board), &vec![Move::new(src, dst, Piece::Pawn)]);
        }
    }

    #[test]
    fn black_pawn_push() {
        for (src, dst) in [(A6, A5), (B6, B5)] {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
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
        for src_idx in [usize::from(A7), B7.into()] {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(src_idx, Color::Black, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &vec![
                    Move::new(src_idx, src_idx + bit_board::SOUTH, Piece::Pawn),
                    Move::new(src_idx, src_idx + bit_board::SOUTH * 2, Piece::Pawn),
                ],
            );
        }
    }

    #[test]
    fn white_pawn_promotion() {
        for i in 9..15usize {
            let mut board = Board::new_empty();
            board.set(i, Color::White, Piece::Pawn);
            board.set(i - bit_board::NO_WE, Color::Black, Piece::Pawn);
            board.set(i - bit_board::NO_EA, Color::Black, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(i, i - bit_board::NO_WE, Piece::Bishop),
                    Move::new_prom(i, i - bit_board::NO_WE, Piece::Knight),
                    Move::new_prom(i, i - bit_board::NO_WE, Piece::Queen),
                    Move::new_prom(i, i - bit_board::NO_WE, Piece::Rook),
                    Move::new_prom(i, i - bit_board::NORTH, Piece::Bishop),
                    Move::new_prom(i, i - bit_board::NORTH, Piece::Knight),
                    Move::new_prom(i, i - bit_board::NORTH, Piece::Queen),
                    Move::new_prom(i, i - bit_board::NORTH, Piece::Rook),
                    Move::new_prom(i, i - bit_board::NO_EA, Piece::Bishop),
                    Move::new_prom(i, i - bit_board::NO_EA, Piece::Knight),
                    Move::new_prom(i, i - bit_board::NO_EA, Piece::Queen),
                    Move::new_prom(i, i - bit_board::NO_EA, Piece::Rook),
                ],
            );
        }
    }

    #[test]
    fn white_pawn_promotion_blocked() {
        for i in 0..8usize {
            let mut board = Board::new_empty();
            board.set(i, Color::Black, Piece::Pawn);
            board.set(i + bit_board::SOUTH, Color::White, Piece::Pawn);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_pawn_promotion() {
        for i in 49..54usize {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(i, Color::Black, Piece::Pawn);
            board.set(i + bit_board::SO_WE, Color::White, Piece::Pawn);
            board.set(i + bit_board::SO_EA, Color::White, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(i, i + bit_board::SO_WE, Piece::Bishop),
                    Move::new_prom(i, i + bit_board::SO_WE, Piece::Knight),
                    Move::new_prom(i, i + bit_board::SO_WE, Piece::Queen),
                    Move::new_prom(i, i + bit_board::SO_WE, Piece::Rook),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Bishop),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Knight),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Queen),
                    Move::new_prom(i, i + bit_board::SOUTH as usize, Piece::Rook),
                    Move::new_prom(i, i + bit_board::SO_EA, Piece::Bishop),
                    Move::new_prom(i, i + bit_board::SO_EA, Piece::Knight),
                    Move::new_prom(i, i + bit_board::SO_EA, Piece::Queen),
                    Move::new_prom(i, i + bit_board::SO_EA, Piece::Rook),
                ],
            );
        }
    }

    #[test]
    fn black_pawn_promotion_blocked() {
        for i in 56..64 {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(i, Color::White, Piece::Pawn);
            board.set(i - bit_board::NORTH, Color::Black, Piece::Pawn);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_pawn_pushes_blocked() {
        let board = Board::from_fen("8/8/8/8/1P6/P7/PP6/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B4, B5, Piece::Pawn),
                Move::new(A3, A4, Piece::Pawn),
                Move::new(B2, B3, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn black_pawn_pushes_blocked() {
        let board = Board::from_fen("8/6pp/7p/6p1/8/8/8/8 b - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(G7, G6, Piece::Pawn),
                Move::new(H6, H5, Piece::Pawn),
                Move::new(G5, G4, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn white_pawn_capture() {
        let board = Board::from_fen("8/8/8/8/p1p5/1P7/8/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B3, A4, Piece::Pawn),
                Move::new(B3, B4, Piece::Pawn),
                Move::new(B3, C4, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn black_pawn_capture() {
        let board = Board::from_fen("8/8/1p6/P1P5/8/8/8/8 b - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B6, B5, Piece::Pawn),
                Move::new(B6, A5, Piece::Pawn),
                Move::new(B6, C5, Piece::Pawn),
            ],
        );
    }

    #[test]
    fn white_pawn_en_passant() {
        for i in 24..31 {
            let mut board = Board::new_empty();
            board.en_passant_target_idx = Some(i - bit_board::NORTH);
            board.set(i, Color::Black, Piece::Pawn);
            board.set(i + bit_board::EAST, Color::White, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_en_pass(i + bit_board::EAST, i - bit_board::NORTH),
                    Move::new(i + bit_board::EAST, i - bit_board::NO_EA, Piece::Pawn),
                ],
            );
        }
    }

    #[test]
    fn black_pawn_en_passant() {
        for i in 32..39 {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.en_passant_target_idx = Some(i + bit_board::SOUTH);
            board.set(i, Color::White, Piece::Pawn);
            board.set(i + bit_board::EAST, Color::Black, Piece::Pawn);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_en_pass(i + bit_board::EAST, i + bit_board::SOUTH),
                    Move::new(i + bit_board::EAST, i + bit_board::SO_EA, Piece::Pawn),
                ],
            );
        }
    }

    #[test]
    fn white_king_queen_side_castle() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 0").unwrap();

        assert_moves_eq(&all_moves(&board), &[Move::new_castle(E1, C1)]);
    }

    #[test]
    fn black_king_queen_side_castle() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 b q - 0 0").unwrap();

        assert_moves_eq(&all_moves(&board), &[Move::new_castle(E8, C8)]);
    }

    #[test]
    fn white_king_queen_side_castle_blocked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 0").unwrap();

        for i in 57..60 {
            let mut board = board.clone();
            board.set(i, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_king_queen_side_castle_blocked() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 w q - 0 0").unwrap();

        for i in 1..4 {
            let mut board = board.clone();
            board.set(i, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_king_queen_side_castle_attacked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 0").unwrap();

        for i in 57..61 {
            let mut board = board.clone();
            board.set(i - NORTH, Black, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_king_queen_side_castle_attacked() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 b q - 0 0").unwrap();

        for i in 1..5 {
            let mut board = board.clone();
            board.set(i + SOUTH, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_king_king_side_castle() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        assert_moves_eq(&all_moves(&board), &[Move::new_castle(E1, G1)]);
    }

    #[test]
    fn black_king_king_side_castle() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        assert_moves_eq(&all_moves(&board), &[Move::new_castle(E8, G8)]);
    }

    #[test]
    fn white_king_king_side_castle_blocked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        for i in 61..63 {
            let mut board = board.clone();
            board.set(i, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_king_king_side_castle_blocked() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        for i in 5..7 {
            let mut board = board.clone();
            board.set(i, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_king_king_side_castle_attacked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        for i in 60..63 {
            let mut board = board.clone();
            board.set(i - NORTH, Black, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_king_king_side_castle_attacked() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        for i in 4..7 {
            let mut board = board.clone();
            board.set(i + SOUTH, White, Rook);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_knight() {
        let mut board = Board::new_empty();
        board.set(B1, White, Knight);
        board.set(F3, White, Knight);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B1, A3, Knight),
                Move::new(B1, C3, Knight),
                Move::new(B1, D2, Knight),
                Move::new(F3, D2, Knight),
                Move::new(F3, D4, Knight),
                Move::new(F3, E1, Knight),
                Move::new(F3, E5, Knight),
                Move::new(F3, G1, Knight),
                Move::new(F3, G5, Knight),
                Move::new(F3, H2, Knight),
                Move::new(F3, H4, Knight),
            ],
        );
    }

    #[test]
    fn white_knight_only_white() {
        let mut board = Board::new_empty();
        board.set(B1, White, Knight);
        board.set(F3, Black, Knight);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(B1, A3, Knight),
                Move::new(B1, C3, Knight),
                Move::new(B1, D2, Knight),
            ],
        );
    }

    #[test]
    fn black_knight() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(C6, Black, Knight);
        board.set(G8, Black, Knight);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(C6, A5, Knight),
                Move::new(C6, A7, Knight),
                Move::new(C6, B4, Knight),
                Move::new(C6, B8, Knight),
                Move::new(C6, D4, Knight),
                Move::new(C6, D8, Knight),
                Move::new(C6, E5, Knight),
                Move::new(C6, E7, Knight),
                Move::new(G8, E7, Knight),
                Move::new(G8, F6, Knight),
                Move::new(G8, H6, Knight),
            ],
        );
    }

    #[test]
    fn black_knight_only_black() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(C6, White, Knight);
        board.set(G8, Black, Knight);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(G8, E7, Knight),
                Move::new(G8, F6, Knight),
                Move::new(G8, H6, Knight),
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

    pub fn new(src: impl BoardPos, dst: impl BoardPos, piece: Piece) -> Self {
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

    pub fn new_castle(src: impl BoardPos, dst: impl BoardPos) -> Self {
        Self {
            is_castle: true,
            ..Self::new(src, dst, Piece::King)
        }
    }

    pub fn new_en_pass(src: impl BoardPos, dst: impl BoardPos) -> Self {
        Self {
            is_en_passant: true,
            ..Self::new(src, dst, Piece::Pawn)
        }
    }

    pub fn new_prom(src: impl BoardPos, dst: impl BoardPos, promote_to: Piece) -> Self {
        Self {
            promote_to: Some(promote_to),
            ..Self::new(src, dst, Piece::Pawn)
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
        )?;

        if self.is_en_passant {
            write!(f, " (en passant)")?;
        }

        if let Some(promote_to) = self.promote_to {
            write!(f, " [promote to: '{:?}']", promote_to)?;
        }

        Ok(())
    }
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}
