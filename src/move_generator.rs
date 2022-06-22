use std::fmt::{Debug, Display};

use crate::{
    bit_board::{self, NORTH, SOUTH},
    board::BoardPos,
    piece::{self},
    Board,
    Color::{self, *},
    Piece, Square,
};

use Piece::*;
use Square::*;

pub fn all_moves(board: &Board) -> Vec<Move> {
    let all_occupancies = board.all_occupancies();
    let friendly_color = match board.is_whites_turn {
        true => Color::White,
        false => Color::Black,
    };
    let opp_color = friendly_color.opposing();
    let friendly_occupancies = board.bishops[friendly_color]
        | board.king[friendly_color]
        | board.knights[friendly_color]
        | board.pawns[friendly_color]
        | board.queens[friendly_color]
        | board.rooks[friendly_color];
    let opp_occupancies = board.bishops[opp_color]
        | board.king[opp_color]
        | board.knights[opp_color]
        | board.pawns[opp_color]
        | board.queens[opp_color]
        | board.rooks[opp_color];

    let mut moves = Vec::new();

    add_bishop_moves(
        board,
        friendly_color,
        all_occupancies,
        friendly_occupancies,
        &mut moves,
    );
    add_king_moves(
        board,
        friendly_color,
        all_occupancies,
        opp_color,
        &mut moves,
    );
    add_knight_moves(board, friendly_color, &mut moves);
    add_pawn_moves(
        board,
        all_occupancies,
        opp_occupancies,
        friendly_color,
        &mut moves,
    );
    add_queen_moves(
        board,
        friendly_color,
        all_occupancies,
        friendly_occupancies,
        &mut moves,
    );
    add_rook_moves(
        board,
        friendly_color,
        all_occupancies,
        friendly_occupancies,
        &mut moves,
    );

    return moves;
}

fn add_bishop_moves(
    board: &Board,
    friendly_color: Color,
    all_occupancies: u64,
    friendly_occupancies: u64,
    moves: &mut Vec<Move>,
) {
    add_sliding_moves(
        board.bishops[friendly_color],
        all_occupancies,
        piece::get_bishop_attacks_for,
        friendly_occupancies,
        friendly_color,
        Bishop,
        moves,
    );
}

fn add_king_moves(
    board: &Board,
    friendly_color: Color,
    all_occupancies: u64,
    opp_color: Color,
    moves: &mut Vec<Move>,
) {
    let mut castle = |required_clear_mask: u64, not_atk: &[Square], src: Square, dst: Square| {
        if bit_board::has_set_bits(all_occupancies & required_clear_mask) {
            return;
        }

        if squares_attacked_by(not_atk, board, opp_color) {
            return;
        }

        moves.push(Move::new_castle(friendly_color, src, dst));
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

fn add_knight_moves(board: &Board, friendly_color: Color, moves: &mut Vec<Move>) {
    for src_i in SetBitsIter(board.knights[friendly_color]) {
        for dst_i in SetBitsIter(piece::get_knight_attack_mask_for(src_i)) {
            moves.push(Move::new(friendly_color, Knight, src_i, dst_i));
        }
    }
}

fn add_pawn_moves(
    board: &Board,
    all_occupancies: u64,
    opp_occupancies: u64,
    friendly_color: Color,
    moves: &mut Vec<Move>,
) {
    let (dir, can_do_double_push, is_prom): (_, fn(usize) -> bool, fn(usize) -> bool) =
        match friendly_color {
            Black => (SOUTH as i8, can_black_do_dbl_push, is_black_prom),
            White => (-(NORTH as i8), can_white_do_dbl_push, is_white_prom),
        };

    let pawns = board.pawns[friendly_color];

    for src_i in SetBitsIter(pawns) {
        let dst_i = (src_i as i8 + dir) as usize;

        if is_prom(dst_i) {
            // Promotions
            if !bit_board::is_bit_set(all_occupancies, dst_i) {
                moves.push(Move::new_prom(friendly_color, src_i, dst_i, Bishop));
                moves.push(Move::new_prom(friendly_color, src_i, dst_i, Knight));
                moves.push(Move::new_prom(friendly_color, src_i, dst_i, Queen));
                moves.push(Move::new_prom(friendly_color, src_i, dst_i, Rook));
            }

            // Capturing promotions
            let captures = piece::get_pawn_attacks_for(src_i, &friendly_color) & opp_occupancies;
            for capture in SetBitsIter(captures) {
                moves.push(Move::new_prom(friendly_color, src_i, capture, Bishop));
                moves.push(Move::new_prom(friendly_color, src_i, capture, Knight));
                moves.push(Move::new_prom(friendly_color, src_i, capture, Queen));
                moves.push(Move::new_prom(friendly_color, src_i, capture, Rook));
            }
        } else {
            // Push
            if !bit_board::is_bit_set(all_occupancies, dst_i) {
                moves.push(Move::new(friendly_color, Pawn, src_i, dst_i));

                // Double push
                if can_do_double_push(src_i) {
                    let dst_idx = (src_i as i8 + dir * 2) as usize;

                    if !bit_board::is_bit_set(all_occupancies, dst_idx) {
                        moves.push(Move::new(friendly_color, Pawn, src_i, dst_idx));
                    }
                }
            }

            // Captures
            let captures = piece::get_pawn_attacks_for(src_i, &friendly_color)
                & board.pawns[friendly_color.opposing()];
            for capture in SetBitsIter(captures) {
                moves.push(Move::new(friendly_color, Pawn, src_i, capture));
            }

            // En passant
            if let Some(en_passant_target_idx) = board.en_passant_target_idx {
                if bit_board::is_bit_set(
                    piece::get_pawn_attacks_for(src_i, &friendly_color),
                    en_passant_target_idx,
                ) {
                    moves.push(Move::new_en_pass(
                        friendly_color,
                        src_i,
                        en_passant_target_idx,
                    ));
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

fn add_queen_moves(
    board: &Board,
    friendly_color: Color,
    all_occupancies: u64,
    friendly_occupancies: u64,
    moves: &mut Vec<Move>,
) {
    add_sliding_moves(
        board.queens[friendly_color],
        all_occupancies,
        piece::get_queen_attacks_for,
        friendly_occupancies,
        friendly_color,
        Queen,
        moves,
    );
}

fn add_rook_moves(
    board: &Board,
    friendly_color: Color,
    all_occupancies: u64,
    friendly_occupancies: u64,
    moves: &mut Vec<Move>,
) {
    add_sliding_moves(
        board.rooks[friendly_color],
        all_occupancies,
        piece::get_rook_attacks_for,
        friendly_occupancies,
        friendly_color,
        Rook,
        moves,
    );
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

fn squares_attacked_by(squares: &[Square], board: &Board, color: Color) -> bool {
    for square in squares {
        if board.is_pos_attacked_by(*square, &color) {
            return true;
        }
    }

    false
}

fn add_sliding_moves(
    pieces: u64,
    all_occupancies: u64,
    get_attacks: fn(i: usize, blockers: u64) -> u64,
    friendly_occupancies: u64,
    friendly_color: Color,
    piece_type: Piece,
    moves: &mut Vec<Move>,
) {
    for src_i in SetBitsIter(pieces) {
        for dst_i in SetBitsIter(get_attacks(src_i, all_occupancies) & !friendly_occupancies) {
            moves.push(Move::new(friendly_color, piece_type, src_i, dst_i));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use pretty_assertions::assert_eq;

    use crate::fen::Fen;

    use super::*;

    #[test]
    fn white_pawn_push() {
        for (src, dst) in [(A3, A4), (B3, B4)] {
            let mut board = Board::new_empty();
            board.set(Color::White, Piece::Pawn, src);

            assert_moves_eq(&all_moves(&board), &vec![Move::new(White, Pawn, src, dst)]);
        }
    }

    #[test]
    fn black_pawn_push() {
        for (src, dst) in [(A6, A5), (B6, B5)] {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(Black, Pawn, src);

            assert_moves_eq(&all_moves(&board), &vec![Move::new(Black, Pawn, src, dst)]);
        }
    }

    #[test]
    fn white_pawn_double_push() {
        let mut board = Board::new_empty();
        board.set(Color::White, Piece::Pawn, A2);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(White, Pawn, A2, A3),
                Move::new(White, Pawn, A2, A4),
            ],
        );

        let mut board = Board::new_empty();
        board.set(Color::White, Piece::Pawn, B2);

        assert_moves_eq(
            &all_moves(&board),
            &vec![
                Move::new(White, Pawn, B2, B3),
                Move::new(White, Pawn, B2, B4),
            ],
        );
    }

    #[test]
    fn black_pawn_double_push() {
        for src_idx in [usize::from(A7), B7.into()] {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(Color::Black, Piece::Pawn, src_idx);

            assert_moves_eq(
                &all_moves(&board),
                &vec![
                    Move::new(Black, Pawn, src_idx, src_idx + bit_board::SOUTH),
                    Move::new(Black, Pawn, src_idx, src_idx + bit_board::SOUTH * 2),
                ],
            );
        }
    }

    #[test]
    fn white_pawn_promotion() {
        for i in 9..15usize {
            let mut board = Board::new_empty();
            board.set(Color::White, Piece::Pawn, i);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(White, i, i - NORTH, Bishop),
                    Move::new_prom(White, i, i - NORTH, Knight),
                    Move::new_prom(White, i, i - NORTH, Queen),
                    Move::new_prom(White, i, i - NORTH, Rook),
                ],
            );
        }
    }

    #[test]
    fn white_pawn_promotion_blocked() {
        for i in 0..8usize {
            let mut board = Board::new_empty();
            board.set(Color::Black, Piece::Pawn, i);
            board.set(Color::White, Piece::Pawn, i + bit_board::SOUTH);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn black_pawn_promotion() {
        for i in 49..54usize {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(Color::Black, Piece::Pawn, i);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_prom(Black, i, i + SOUTH, Bishop),
                    Move::new_prom(Black, i, i + SOUTH, Knight),
                    Move::new_prom(Black, i, i + SOUTH, Queen),
                    Move::new_prom(Black, i, i + SOUTH, Rook),
                ],
            );
        }
    }

    #[test]
    fn black_pawn_promotion_blocked() {
        for i in 56..64 {
            let mut board = Board::new_empty();
            board.is_whites_turn = false;
            board.set(Color::White, Piece::Pawn, i);
            board.set(Color::Black, Piece::Pawn, i - bit_board::NORTH);

            assert_moves_eq(&all_moves(&board), &[]);
        }
    }

    #[test]
    fn white_pawn_pushes_blocked() {
        let board = Board::from_fen("8/8/8/8/1P6/P7/PP6/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Pawn, B4, B5),
                Move::new(White, Pawn, A3, A4),
                Move::new(White, Pawn, B2, B3),
            ],
        );
    }

    #[test]
    fn black_pawn_pushes_blocked() {
        let board = Board::from_fen("8/6pp/7p/6p1/8/8/8/8 b - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Pawn, G7, G6),
                Move::new(Black, Pawn, H6, H5),
                Move::new(Black, Pawn, G5, G4),
            ],
        );
    }

    #[test]
    fn white_pawn_capture() {
        let board = Board::from_fen("8/8/8/8/p1p5/1P7/8/8 w - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Pawn, B3, A4),
                Move::new(White, Pawn, B3, B4),
                Move::new(White, Pawn, B3, C4),
            ],
        );
    }

    #[test]
    fn black_pawn_capture() {
        let board = Board::from_fen("8/8/1p6/P1P5/8/8/8/8 b - - 0 0").unwrap();

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Pawn, B6, B5),
                Move::new(Black, Pawn, B6, A5),
                Move::new(Black, Pawn, B6, C5),
            ],
        );
    }

    #[test]
    fn white_pawn_capture_promotion() {
        let mut board = Board::new_empty();
        board.set(Black, Rook, A8);
        board.set(Black, Queen, C8);
        board.set(White, Pawn, B7);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new_prom(White, B7, A8, Bishop),
                Move::new_prom(White, B7, A8, Knight),
                Move::new_prom(White, B7, A8, Queen),
                Move::new_prom(White, B7, A8, Rook),
                Move::new_prom(White, B7, B8, Bishop),
                Move::new_prom(White, B7, B8, Knight),
                Move::new_prom(White, B7, B8, Queen),
                Move::new_prom(White, B7, B8, Rook),
                Move::new_prom(White, B7, C8, Bishop),
                Move::new_prom(White, B7, C8, Knight),
                Move::new_prom(White, B7, C8, Queen),
                Move::new_prom(White, B7, C8, Rook),
            ],
        );
    }

    #[test]
    fn black_pawn_capture_promotion() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Pawn, B2);
        board.set(White, Rook, A1);
        board.set(White, Bishop, C1);

        println!("{}", board);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new_prom(Black, B2, A1, Bishop),
                Move::new_prom(Black, B2, A1, Knight),
                Move::new_prom(Black, B2, A1, Queen),
                Move::new_prom(Black, B2, A1, Rook),
                Move::new_prom(Black, B2, B1, Bishop),
                Move::new_prom(Black, B2, B1, Knight),
                Move::new_prom(Black, B2, B1, Queen),
                Move::new_prom(Black, B2, B1, Rook),
                Move::new_prom(Black, B2, C1, Bishop),
                Move::new_prom(Black, B2, C1, Knight),
                Move::new_prom(Black, B2, C1, Queen),
                Move::new_prom(Black, B2, C1, Rook),
            ],
        );
    }

    #[test]
    fn white_pawn_en_passant() {
        for i in 24..31 {
            let mut board = Board::new_empty();
            board.en_passant_target_idx = Some(i - bit_board::NORTH);
            board.set(Color::Black, Piece::Pawn, i);
            board.set(Color::White, Piece::Pawn, i + bit_board::EAST);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_en_pass(White, i + bit_board::EAST, i - bit_board::NORTH),
                    Move::new(White, Pawn, i + bit_board::EAST, i - bit_board::NO_EA),
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
            board.set(Color::White, Piece::Pawn, i);
            board.set(Color::Black, Piece::Pawn, i + bit_board::EAST);

            assert_moves_eq(
                &all_moves(&board),
                &[
                    Move::new_en_pass(Black, i + bit_board::EAST, i + bit_board::SOUTH),
                    Move::new(Black, Pawn, i + bit_board::EAST, i + bit_board::SO_EA),
                ],
            );
        }
    }

    #[test]
    fn white_king_queen_side_castle() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 0").unwrap();

        let mut expected_moves = Vec::new();
        add_rook_moves(
            &board,
            White,
            board.all_occupancies(),
            board.king[White],
            &mut expected_moves,
        );
        expected_moves.push(Move::new_castle(White, E1, C1));

        assert_moves_eq(&all_moves(&board), &expected_moves);
    }

    #[test]
    fn black_king_queen_side_castle() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 b q - 0 0").unwrap();

        let mut expected_moves = Vec::new();
        add_rook_moves(
            &board,
            Black,
            board.all_occupancies(),
            board.king[Black],
            &mut expected_moves,
        );
        expected_moves.push(Move::new_castle(Black, E8, C8));

        assert_moves_eq(&all_moves(&board), &expected_moves);
    }

    #[test]
    fn white_king_queen_side_castle_blocked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/3K3 w Q - 0 0").unwrap();

        for i in 57..60 {
            let mut board = board.clone();
            board.set(Black, Rook, i);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                White,
                board.all_occupancies(),
                board.king[White],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn black_king_queen_side_castle_blocked() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 b q - 0 0").unwrap();

        for i in 1..4 {
            let mut board = board.clone();
            board.set(White, Rook, i);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                Black,
                board.all_occupancies(),
                board.king[Black],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn white_king_queen_side_castle_attacked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/R3K3 w Q - 0 0").unwrap();

        for i in 57..61 {
            let mut board = board.clone();
            board.set(Black, Rook, i - NORTH);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                White,
                board.all_occupancies(),
                board.king[White],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn black_king_queen_side_castle_attacked() {
        let board = Board::from_fen("r3k3/8/8/8/8/8/8/8 b q - 0 0").unwrap();

        for i in 1..5 {
            let mut board = board.clone();
            board.set(White, Rook, i + SOUTH);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                Black,
                board.all_occupancies(),
                board.king[Black],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn white_king_king_side_castle() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        let mut expected_moves = Vec::new();
        add_rook_moves(
            &board,
            White,
            board.all_occupancies(),
            board.king[White],
            &mut expected_moves,
        );
        expected_moves.push(Move::new_castle(White, E1, G1));

        assert_moves_eq(&all_moves(&board), &expected_moves);
    }

    #[test]
    fn black_king_king_side_castle() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        let mut expected_moves = Vec::new();
        add_rook_moves(
            &board,
            Black,
            board.all_occupancies(),
            board.king[Black],
            &mut expected_moves,
        );
        expected_moves.push(Move::new_castle(Black, E8, G8));

        assert_moves_eq(&all_moves(&board), &expected_moves);
    }

    #[test]
    fn white_king_king_side_castle_blocked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        for i in 61..63 {
            let mut board = board.clone();
            board.set(Black, Rook, i);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                White,
                board.all_occupancies(),
                board.king[White],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn black_king_king_side_castle_blocked() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        for i in 5..7 {
            let mut board = board.clone();
            board.set(White, Rook, i);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                Black,
                board.all_occupancies(),
                board.king[Black],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn white_king_king_side_castle_attacked() {
        let board = Board::from_fen("8/8/8/8/8/8/8/4K2R w K - 0 0").unwrap();

        for i in 60..63 {
            let mut board = board.clone();
            board.set(Black, Rook, i - NORTH);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                White,
                board.all_occupancies(),
                board.king[White],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn black_king_king_side_castle_attacked() {
        let board = Board::from_fen("4k2r/8/8/8/8/8/8/8 b k - 0 0").unwrap();

        for i in 4..7 {
            let mut board = board.clone();
            board.set(White, Rook, i + SOUTH);

            let mut rook_moves = Vec::new();
            add_rook_moves(
                &board,
                Black,
                board.all_occupancies(),
                board.king[Black],
                &mut rook_moves,
            );

            assert_moves_eq(&all_moves(&board), &rook_moves);
        }
    }

    #[test]
    fn white_knight() {
        let mut board = Board::new_empty();
        board.set(White, Knight, B1);
        board.set(White, Knight, F3);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Knight, B1, A3),
                Move::new(White, Knight, B1, C3),
                Move::new(White, Knight, B1, D2),
                Move::new(White, Knight, F3, D2),
                Move::new(White, Knight, F3, D4),
                Move::new(White, Knight, F3, E1),
                Move::new(White, Knight, F3, E5),
                Move::new(White, Knight, F3, G1),
                Move::new(White, Knight, F3, G5),
                Move::new(White, Knight, F3, H2),
                Move::new(White, Knight, F3, H4),
            ],
        );
    }

    #[test]
    fn white_knight_only_white() {
        let mut board = Board::new_empty();
        board.set(White, Knight, B1);
        board.set(Black, Knight, F3);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Knight, B1, A3),
                Move::new(White, Knight, B1, C3),
                Move::new(White, Knight, B1, D2),
            ],
        );
    }

    #[test]
    fn black_knight() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Knight, C6);
        board.set(Black, Knight, G8);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Knight, C6, A5),
                Move::new(Black, Knight, C6, A7),
                Move::new(Black, Knight, C6, B4),
                Move::new(Black, Knight, C6, B8),
                Move::new(Black, Knight, C6, D4),
                Move::new(Black, Knight, C6, D8),
                Move::new(Black, Knight, C6, E5),
                Move::new(Black, Knight, C6, E7),
                Move::new(Black, Knight, G8, E7),
                Move::new(Black, Knight, G8, F6),
                Move::new(Black, Knight, G8, H6),
            ],
        );
    }

    #[test]
    fn black_knight_only_black() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(White, Knight, C6);
        board.set(Black, Knight, G8);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Knight, G8, E7),
                Move::new(Black, Knight, G8, F6),
                Move::new(Black, Knight, G8, H6),
            ],
        );
    }

    #[test]
    fn white_bishop() {
        let mut board = Board::new_empty();
        board.set(White, Bishop, C1);
        board.set(White, Bishop, A6);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Bishop, A6, B5),
                Move::new(White, Bishop, A6, B7),
                Move::new(White, Bishop, A6, C4),
                Move::new(White, Bishop, A6, C8),
                Move::new(White, Bishop, A6, D3),
                Move::new(White, Bishop, A6, E2),
                Move::new(White, Bishop, A6, F1),
                Move::new(White, Bishop, C1, A3),
                Move::new(White, Bishop, C1, B2),
                Move::new(White, Bishop, C1, D2),
                Move::new(White, Bishop, C1, E3),
                Move::new(White, Bishop, C1, F4),
                Move::new(White, Bishop, C1, G5),
                Move::new(White, Bishop, C1, H6),
            ],
        );
    }

    #[test]
    fn black_bishop() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Bishop, F8);
        board.set(Black, Bishop, H3);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Bishop, F8, G7),
                Move::new(Black, Bishop, F8, H6),
                Move::new(Black, Bishop, F8, E7),
                Move::new(Black, Bishop, F8, D6),
                Move::new(Black, Bishop, F8, C5),
                Move::new(Black, Bishop, F8, B4),
                Move::new(Black, Bishop, F8, A3),
                Move::new(Black, Bishop, H3, G4),
                Move::new(Black, Bishop, H3, F5),
                Move::new(Black, Bishop, H3, E6),
                Move::new(Black, Bishop, H3, D7),
                Move::new(Black, Bishop, H3, C8),
                Move::new(Black, Bishop, H3, G2),
                Move::new(Black, Bishop, H3, F1),
            ],
        );
    }

    #[test]
    fn white_bishop_blocked() {
        let mut board = Board::new_empty();
        board.set(White, Bishop, G7);
        board.set(White, Pawn, H6);
        board.set(Black, Bishop, E5);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Bishop, G7, F8),
                Move::new(White, Bishop, G7, H8),
                Move::new(White, Bishop, G7, F6),
                Move::new(White, Bishop, G7, E5),
                Move::new(White, Pawn, H6, H7),
            ],
        );
    }

    #[test]
    fn black_bishop_blocked() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Bishop, C2);
        board.set(White, Bishop, B3);
        board.set(White, Bishop, E4);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Bishop, C2, E4),
                Move::new(Black, Bishop, C2, B3),
                Move::new(Black, Bishop, C2, D3),
                Move::new(Black, Bishop, C2, B1),
                Move::new(Black, Bishop, C2, D1),
            ],
        );
    }

    #[test]
    fn white_queen() {
        let mut board = Board::new_empty();
        board.set(White, Queen, D1);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Queen, D1, A1),
                Move::new(White, Queen, D1, A4),
                Move::new(White, Queen, D1, B1),
                Move::new(White, Queen, D1, B3),
                Move::new(White, Queen, D1, C1),
                Move::new(White, Queen, D1, C2),
                Move::new(White, Queen, D1, D2),
                Move::new(White, Queen, D1, D3),
                Move::new(White, Queen, D1, D4),
                Move::new(White, Queen, D1, D5),
                Move::new(White, Queen, D1, D6),
                Move::new(White, Queen, D1, D7),
                Move::new(White, Queen, D1, D8),
                Move::new(White, Queen, D1, E1),
                Move::new(White, Queen, D1, E2),
                Move::new(White, Queen, D1, F1),
                Move::new(White, Queen, D1, F3),
                Move::new(White, Queen, D1, G1),
                Move::new(White, Queen, D1, G4),
                Move::new(White, Queen, D1, H1),
                Move::new(White, Queen, D1, H5),
            ],
        );
    }

    #[test]
    fn black_queen() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Queen, D8);

        println!("{}", board);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Queen, D8, A5),
                Move::new(Black, Queen, D8, A8),
                Move::new(Black, Queen, D8, B6),
                Move::new(Black, Queen, D8, B8),
                Move::new(Black, Queen, D8, C7),
                Move::new(Black, Queen, D8, C8),
                Move::new(Black, Queen, D8, D1),
                Move::new(Black, Queen, D8, D2),
                Move::new(Black, Queen, D8, D3),
                Move::new(Black, Queen, D8, D4),
                Move::new(Black, Queen, D8, D5),
                Move::new(Black, Queen, D8, D6),
                Move::new(Black, Queen, D8, D7),
                Move::new(Black, Queen, D8, E7),
                Move::new(Black, Queen, D8, E8),
                Move::new(Black, Queen, D8, F6),
                Move::new(Black, Queen, D8, F8),
                Move::new(Black, Queen, D8, G5),
                Move::new(Black, Queen, D8, G8),
                Move::new(Black, Queen, D8, H4),
                Move::new(Black, Queen, D8, H8),
            ],
        );
    }

    #[test]
    fn black_queen_blocked() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Queen, H4);
        board.set(Black, Pawn, H3);
        board.set(White, Queen, E4);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Pawn, H3, H2),
                Move::new(Black, Queen, H4, D8),
                Move::new(Black, Queen, H4, E1),
                Move::new(Black, Queen, H4, E4),
                Move::new(Black, Queen, H4, E7),
                Move::new(Black, Queen, H4, F2),
                Move::new(Black, Queen, H4, F4),
                Move::new(Black, Queen, H4, F6),
                Move::new(Black, Queen, H4, G3),
                Move::new(Black, Queen, H4, G4),
                Move::new(Black, Queen, H4, G5),
                Move::new(Black, Queen, H4, H5),
                Move::new(Black, Queen, H4, H6),
                Move::new(Black, Queen, H4, H7),
                Move::new(Black, Queen, H4, H8),
            ],
        );
    }

    #[test]
    fn white_queen_blocked() {
        let mut board = Board::new_empty();
        board.set(White, Queen, A4);
        board.set(White, Pawn, A5);
        board.set(Black, Queen, C6);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Queen, A4, A1),
                Move::new(White, Queen, A4, A2),
                Move::new(White, Queen, A4, A3),
                Move::new(White, Queen, A4, B3),
                Move::new(White, Queen, A4, B4),
                Move::new(White, Queen, A4, B5),
                Move::new(White, Queen, A4, C2),
                Move::new(White, Queen, A4, C4),
                Move::new(White, Queen, A4, C6),
                Move::new(White, Queen, A4, D1),
                Move::new(White, Queen, A4, D4),
                Move::new(White, Queen, A4, E4),
                Move::new(White, Queen, A4, F4),
                Move::new(White, Queen, A4, G4),
                Move::new(White, Queen, A4, H4),
                Move::new(White, Pawn, A5, A6),
            ],
        );
    }

    #[test]
    fn white_rook() {
        let mut board = Board::new_empty();
        board.set(White, Rook, A1);
        board.set(White, Rook, H8);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Rook, A1, A8),
                Move::new(White, Rook, A1, A7),
                Move::new(White, Rook, A1, A6),
                Move::new(White, Rook, A1, A5),
                Move::new(White, Rook, A1, A4),
                Move::new(White, Rook, A1, A3),
                Move::new(White, Rook, A1, A2),
                Move::new(White, Rook, A1, B1),
                Move::new(White, Rook, A1, C1),
                Move::new(White, Rook, A1, D1),
                Move::new(White, Rook, A1, E1),
                Move::new(White, Rook, A1, F1),
                Move::new(White, Rook, A1, G1),
                Move::new(White, Rook, A1, H1),
                Move::new(White, Rook, H8, A8),
                Move::new(White, Rook, H8, B8),
                Move::new(White, Rook, H8, C8),
                Move::new(White, Rook, H8, D8),
                Move::new(White, Rook, H8, E8),
                Move::new(White, Rook, H8, F8),
                Move::new(White, Rook, H8, G8),
                Move::new(White, Rook, H8, H7),
                Move::new(White, Rook, H8, H6),
                Move::new(White, Rook, H8, H5),
                Move::new(White, Rook, H8, H4),
                Move::new(White, Rook, H8, H3),
                Move::new(White, Rook, H8, H2),
                Move::new(White, Rook, H8, H1),
            ],
        );
    }

    #[test]
    fn white_rook_blocked() {
        let mut board = Board::new_empty();
        board.set(White, Rook, A1);
        board.set(White, Pawn, A3);
        board.set(Black, Rook, C1);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(White, Rook, A1, A2),
                Move::new(White, Rook, A1, B1),
                Move::new(White, Rook, A1, C1),
                Move::new(White, Pawn, A3, A4),
            ],
        );
    }

    #[test]
    fn black_rook() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Rook, A8);
        board.set(Black, Rook, H1);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Rook, A8, A7),
                Move::new(Black, Rook, A8, A6),
                Move::new(Black, Rook, A8, A5),
                Move::new(Black, Rook, A8, A4),
                Move::new(Black, Rook, A8, A3),
                Move::new(Black, Rook, A8, A2),
                Move::new(Black, Rook, A8, A1),
                Move::new(Black, Rook, A8, B8),
                Move::new(Black, Rook, A8, C8),
                Move::new(Black, Rook, A8, D8),
                Move::new(Black, Rook, A8, E8),
                Move::new(Black, Rook, A8, F8),
                Move::new(Black, Rook, A8, G8),
                Move::new(Black, Rook, A8, H8),
                Move::new(Black, Rook, H1, A1),
                Move::new(Black, Rook, H1, B1),
                Move::new(Black, Rook, H1, C1),
                Move::new(Black, Rook, H1, D1),
                Move::new(Black, Rook, H1, E1),
                Move::new(Black, Rook, H1, F1),
                Move::new(Black, Rook, H1, G1),
                Move::new(Black, Rook, H1, H2),
                Move::new(Black, Rook, H1, H3),
                Move::new(Black, Rook, H1, H4),
                Move::new(Black, Rook, H1, H5),
                Move::new(Black, Rook, H1, H6),
                Move::new(Black, Rook, H1, H7),
                Move::new(Black, Rook, H1, H8),
            ],
        );
    }

    #[test]
    fn black_rook_blocked() {
        let mut board = Board::new_empty();
        board.is_whites_turn = false;
        board.set(Black, Rook, H8);
        board.set(Black, Pawn, H6);
        board.set(White, Rook, E8);

        assert_moves_eq(
            &all_moves(&board),
            &[
                Move::new(Black, Rook, H8, H7),
                Move::new(Black, Rook, H8, G8),
                Move::new(Black, Rook, H8, F8),
                Move::new(Black, Rook, H8, E8),
                Move::new(Black, Pawn, H6, H5),
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
    dst: usize,
    is_castle: bool,
    is_double_push: bool,
    is_en_passant: bool,
    piece: Piece,
    piece_color: Color,
    prom_to: Option<Piece>,
    src: usize,
}

impl Move {
    pub fn dst(&self) -> usize {
        self.dst
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

    pub fn new(color: Color, piece: Piece, src: impl BoardPos, dst: impl BoardPos) -> Self {
        Self {
            dst: dst.into(),
            is_castle: false,
            is_double_push: false,
            is_en_passant: false,
            piece,
            piece_color: color,
            prom_to: None,
            src: src.into(),
        }
    }

    pub fn new_castle(color: Color, src: impl BoardPos, dst: impl BoardPos) -> Self {
        Self {
            is_castle: true,
            ..Self::new(color, King, src, dst)
        }
    }

    pub fn new_en_pass(color: Color, src: impl BoardPos, dst: impl BoardPos) -> Self {
        Self {
            is_en_passant: true,
            ..Self::new(color, Pawn, src, dst)
        }
    }

    pub fn new_prom(
        color: Color,
        src: impl BoardPos,
        dst: impl BoardPos,
        promote_to: Piece,
    ) -> Self {
        Self {
            prom_to: Some(promote_to),
            ..Self::new(color, Pawn, src, dst)
        }
    }

    pub fn piece(&self) -> Piece {
        self.piece
    }

    pub fn piece_color(&self) -> Color {
        self.piece_color
    }

    /// The piece that the pawn should be promoted to.
    ///
    /// Is only set when the move is a promotion.
    pub fn prom_to(&self) -> Option<Piece> {
        self.prom_to
    }

    pub fn promote_to(&self) -> Option<Piece> {
        self.prom_to
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

    pub fn set_piece_color(&mut self, val: Color) {
        self.piece_color = val;
    }

    pub fn set_promote_to(&mut self, val: Option<Piece>) {
        self.prom_to = val;
    }

    pub fn set_source(&mut self, val: usize) {
        self.src = val;
    }

    pub fn set_target(&mut self, val: usize) {
        self.dst = val;
    }

    pub fn src(&self) -> usize {
        self.src
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?}: {:?}->{:?}",
            self.piece_color,
            self.piece,
            Square::try_from(self.src()).unwrap(),
            Square::try_from(self.dst()).unwrap(),
        )?;

        if self.is_en_passant {
            write!(f, " (en passant)")?;
        }

        if let Some(promote_to) = self.prom_to {
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
