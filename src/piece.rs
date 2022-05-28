use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, ColoredMovMask, MoveMask},
    board::MoveByIdx,
    square::Square,
    Board, Color,
};

// TODO: Investigate if this constants should really be defined here.

pub type ToEdgeOffset = [[usize; 8]; Board::SIZE as usize];

pub const DIR_NORTH: usize = 0;
pub const DIR_NORTH_EAST: usize = 1;
pub const DIR_EAST: usize = 2;
pub const DIR_SOUTH_EAST: usize = 3;
pub const DIR_SOUTH: usize = 4;
pub const DIR_SOUTH_WEST: usize = 5;
pub const DIR_WEST: usize = 6;
pub const DIR_NORTH_WEST: usize = 7;

pub const DIR_OFFSETS: [i8; 8] = [-8, -7, 1, 9, 8, 7, -1, -9];

pub const TO_EDGE_OFFSETS: ToEdgeOffset = generate_to_edge_map();

const NOT_FILE_A: u64 = 18374403900871474942;
const NOT_FILE_AB: u64 = 18229723555195321596;
const NOT_FILE_GH: u64 = 4557430888798830399;
const NOT_FILE_H: u64 = 9187201950435737471;

const BISHOP_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_bishop_move_mask);
const KING_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_king_move_mask);
const KNIGHT_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_knight_move_mask);
const PAWN_MOVE_MASK: Lazy<ColoredMovMask> = Lazy::new(generate_pawn_move_mask);
const ROOK_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_rook_move_mask);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

impl Piece {
    fn get_moves_for_bishop_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_NORTH_EAST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_SOUTH_EAST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_SOUTH_WEST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_NORTH_WEST,
            board,
        ));

        moves
    }

    fn get_moves_for_king_at(king_idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        let north = (king_idx as i8 + DIR_OFFSETS[DIR_NORTH]) as usize;
        let north_east = (king_idx as i8 + DIR_OFFSETS[DIR_NORTH_EAST]) as usize;
        let east = (king_idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize;
        let south_east = (king_idx as i8 + DIR_OFFSETS[DIR_SOUTH_EAST]) as usize;
        let south = (king_idx as i8 + DIR_OFFSETS[DIR_SOUTH]) as usize;
        let south_west = (king_idx as i8 + DIR_OFFSETS[DIR_SOUTH_WEST]) as usize;
        let west = (king_idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize;
        let north_west = (king_idx as i8 + DIR_OFFSETS[DIR_NORTH_WEST]) as usize;

        if TO_EDGE_OFFSETS[king_idx][DIR_NORTH] > 0 {
            Self::push_move_or_hit(king_idx, north, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_NORTH_EAST] > 0 {
            Self::push_move_or_hit(king_idx, north_east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_EAST] > 0 {
            Self::push_move_or_hit(king_idx, east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_SOUTH_EAST] > 0 {
            Self::push_move_or_hit(king_idx, south_east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_SOUTH] > 0 {
            Self::push_move_or_hit(king_idx, south, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_SOUTH_WEST] > 0 {
            Self::push_move_or_hit(king_idx, south_west, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_WEST] > 0 {
            Self::push_move_or_hit(king_idx, west, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[king_idx][DIR_NORTH_WEST] > 0 {
            Self::push_move_or_hit(king_idx, north_west, &mut moves, board);
        }

        let king_ins = board
            .get(&king_idx)
            .unwrap_or_else(|| panic!("castle check failed, no piece at index '{}'", king_idx));
        let atk_color = king_ins.color.get_opposing();

        let push_castle_move_if_applicable =
            &mut |can_castle: bool, poses_to_validate: &[usize], move_to_add: usize| {
                if can_castle {
                    for &pos in poses_to_validate {
                        if board.get(&pos).is_some() || board.is_pos_attacked_by(&pos, &atk_color) {
                            return;
                        }
                    }

                    moves.push((king_idx, move_to_add));
                }
            };

        if king_ins.color == Color::Black {
            push_castle_move_if_applicable(board.can_black_castle_queen_side, &[1, 2, 3], 2);
            push_castle_move_if_applicable(board.can_black_castle_king_side, &[5, 6], 6);
        } else {
            push_castle_move_if_applicable(board.can_white_castle_queen_side, &[57, 58, 59], 58);
            push_castle_move_if_applicable(board.can_white_castle_king_side, &[61, 62], 62);
        }

        moves
    }

    fn get_moves_for_knight_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] * 2 + DIR_OFFSETS[DIR_WEST]) as usize,
                    &mut moves,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] * 2 + DIR_OFFSETS[DIR_EAST]) as usize,
                    &mut moves,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_EAST] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] + DIR_OFFSETS[DIR_EAST] * 2) as usize,
                    &mut moves,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] + DIR_OFFSETS[DIR_EAST] * 2) as usize,
                    &mut moves,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] * 2 + DIR_OFFSETS[DIR_EAST]) as usize,
                    &mut moves,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] * 2 + DIR_OFFSETS[DIR_WEST]) as usize,
                    &mut moves,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_WEST] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] + DIR_OFFSETS[DIR_WEST] * 2) as usize,
                    &mut moves,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
                Self::push_move_or_hit(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] + DIR_OFFSETS[DIR_WEST] * 2) as usize,
                    &mut moves,
                    board,
                );
            }
        }

        moves
    }

    pub fn is_pawn_at_start_idx(idx: usize, color: &Color) -> bool {
        if color == &Color::White {
            return idx > 47 && idx < 56;
        }

        idx > 7 && idx < 16
    }

    fn get_moves_for_pawn_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let pawn_ins = match board.get(&idx) {
            Some(i) => i,
            None => panic!("no piece found at the position '{}'", idx),
        };
        let mut moves = Vec::new();

        let dir = Board::get_attack_dir_of(&pawn_ins.color);
        let dir_offset = DIR_OFFSETS[dir];

        Self::push_move_if_empty(idx, (idx as i8 + dir_offset) as usize, &mut moves, board);
        if Piece::is_pawn_at_start_idx(idx, &pawn_ins.color) {
            Self::push_move_if_empty(
                idx,
                (idx as i8 + dir_offset * 2) as usize,
                &mut moves,
                board,
            );
        }

        if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
            Self::push_move_if_opponent(
                idx,
                (idx as i8 + dir_offset + DIR_OFFSETS[DIR_WEST]) as usize,
                &mut moves,
                &board,
            );
        }
        if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
            Self::push_move_if_opponent(
                idx,
                (idx as i8 + dir_offset + DIR_OFFSETS[DIR_EAST]) as usize,
                &mut moves,
                &board,
            );
        }

        for (en_passant_src, en_passant_hit) in &board.piece_eligible_for_en_passant {
            moves.push((
                *en_passant_src,
                (*en_passant_hit as i8 + DIR_OFFSETS[dir]) as usize,
            ));
        }

        moves
    }

    fn get_moves_for_queen_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_NORTH, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_NORTH_EAST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_EAST, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_SOUTH_EAST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_SOUTH, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_SOUTH_WEST,
            board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_WEST, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx,
            DIR_NORTH_WEST,
            board,
        ));

        moves
    }

    fn get_moves_for_rook_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_NORTH, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_EAST, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_SOUTH, board,
        ));
        moves.append(&mut Self::get_moves_for_sliding_piece_at(
            idx, DIR_WEST, board,
        ));

        moves
    }

    // TODO: pass vec that the moves will be pushed to
    fn get_moves_for_sliding_piece_at(src_idx: usize, dir: usize, board: &Board) -> Vec<MoveByIdx> {
        let mut moves = Vec::new();

        for offset in 0..TO_EDGE_OFFSETS[src_idx][dir] {
            let hit_idx = (src_idx as i8 + (offset + 1) as i8 * DIR_OFFSETS[dir]) as usize;

            if Self::push_move_or_hit(src_idx, hit_idx, &mut moves, board) {
                break;
            }
        }

        moves
    }

    pub fn get_moves_of_piece_at(idx: usize, board: &Board) -> Vec<MoveByIdx> {
        let ins = match board.get(&idx) {
            Some(i) => i,
            None => panic!(
                "cannot add move at index '{}' because the position is empty",
                idx
            ),
        };

        match &ins.piece {
            Self::Bishop => Self::get_moves_for_bishop_at(idx, board),
            Self::King => Self::get_moves_for_king_at(idx, board),
            Self::Knight => Self::get_moves_for_knight_at(idx, board),
            Self::Pawn => Self::get_moves_for_pawn_at(idx, board),
            Self::Queen => Self::get_moves_for_queen_at(idx, board),
            Self::Rook => Self::get_moves_for_rook_at(idx, board),
        }
    }

    pub const fn get_symbol(&self) -> &'static str {
        match self {
            Self::Bishop => "BI",
            Self::King => "KI",
            Self::Knight => "KN",
            Self::Pawn => "PA",
            Self::Queen => "QU",
            Self::Rook => "RO",
        }
    }

    /// Calculates all possible moves for the piece at the source position, and
    /// checks if it can reach the specified hit position.
    pub fn is_valid_move(mv: MoveByIdx, board: &Board) -> bool {
        let (src_idx, _) = mv;
        let moves = Self::get_moves_of_piece_at(src_idx, &board);

        moves.contains(&mv)
    }

    fn push_move_if_empty(
        src_idx: usize,
        hit_idx: usize,
        moves: &mut Vec<MoveByIdx>,
        board: &Board,
    ) {
        if board.get(&hit_idx).is_some() {
            return;
        }

        moves.push((src_idx, hit_idx));
    }

    fn push_move_if_opponent(
        src_idx: usize,
        hit_idx: usize,
        moves: &mut Vec<MoveByIdx>,
        board: &Board,
    ) {
        let src_ins = match board.get(&src_idx) {
            Some(i) => i,
            None => panic!("could not find src piece at '{}'", src_idx),
        };
        let hit_ins = match board.get(&hit_idx) {
            Some(i) => i,
            // No piece at all is clearly not an opponent, thus just return.
            None => return,
        };

        if src_ins.color == hit_ins.color {
            return;
        }

        moves.push((src_idx, hit_idx));
    }

    /// Pushes a new move to the provided vec.
    ///
    /// Returns `true` if a piece was hit.
    fn push_move_or_hit(
        src_idx: usize,
        hit_idx: usize,
        moves: &mut Vec<MoveByIdx>,
        board: &Board,
    ) -> bool {
        let src_ins = board
            .get(&src_idx)
            .unwrap_or_else(|| panic!("no src piece at idx '{}'", src_idx));

        if let Some(hit_ins) = board.get(&hit_idx) {
            // You can't really take your own piece, thus the player need to be
            // different for a move to be added.
            if src_ins.color != hit_ins.color {
                moves.push((src_idx, hit_idx));
            }

            return true;
        }

        moves.push((src_idx, hit_idx));

        false
    }
}

const fn generate_to_edge_map() -> ToEdgeOffset {
    const fn min(left: usize, right: usize) -> usize {
        if left < right {
            return left;
        }

        right
    }

    let mut map: ToEdgeOffset = [[0; 8]; Board::SIZE as usize];

    let mut file = 0;
    let mut rank = 0;

    while file < Board::WIDTH {
        while rank < Board::HEIGHT {
            let idx = (rank + file * Board::HEIGHT) as usize;
            let to_north = file;
            let to_east = Board::WIDTH - 1 - rank;
            let to_south = Board::HEIGHT - 1 - file;
            let to_west = rank;

            map[idx][DIR_NORTH] = to_north;
            map[idx][DIR_NORTH_EAST] = min(to_north, to_east);
            map[idx][DIR_EAST] = to_east;
            map[idx][DIR_SOUTH_EAST] = min(to_south, to_east);
            map[idx][DIR_SOUTH] = to_south;
            map[idx][DIR_SOUTH_WEST] = min(to_south, to_west);
            map[idx][DIR_WEST] = to_west;
            map[idx][DIR_NORTH_WEST] = min(to_north, to_west);

            rank += 1;
        }

        file += 1;
        rank = 0;
    }

    map
}

fn generate_bishop_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..Board::SIZE as u64 {
        let mut board = 0;
        bit_board::set_bit(&mut board, i);

        let file = i % Board::HEIGHT as u64;
        let rank = i / Board::HEIGHT as u64;

        let to_no_ea = u64::min((Board::WIDTH - 1) as u64 - file, rank);
        let to_so_ea = u64::min(Board::WIDTH as u64 - file, Board::HEIGHT as u64 - rank) - 1;
        let to_so_we = u64::min(file, (Board::HEIGHT - 1) as u64 - rank);
        let to_no_we = u64::min(file, rank);

        for iter in 1..to_no_ea {
            mask[&i] |= board >> bit_board::NO_EA * iter;
        }
        for iter in 1..to_so_ea {
            mask[&i] |= board << bit_board::SO_EA * iter;
        }
        for iter in 1..to_so_we {
            mask[&i] |= board << bit_board::SO_WE * iter;
        }
        for iter in 1..to_no_we {
            mask[&i] |= board >> bit_board::NO_WE * iter;
        }
    }

    mask
}

fn generate_king_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..Board::SIZE as u64 {
        let mut board = 0;
        bit_board::set_bit(&mut board, i);

        mask[&i] |= board >> bit_board::NORTH;
        if bit_board::is_set(board & NOT_FILE_H, i) {
            mask[&i] |= board >> bit_board::NO_EA;
            mask[&i] |= board << bit_board::EAST;
            mask[&i] |= board << bit_board::SO_EA;
        }
        mask[&i] |= board << bit_board::SOUTH;
        if bit_board::is_set(board & NOT_FILE_A, i) {
            mask[&i] |= board << bit_board::SO_WE;
            mask[&i] |= board >> bit_board::WEST;
            mask[&i] |= board >> bit_board::NO_WE;
        }
    }

    mask
}

fn generate_knight_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..Board::SIZE as u64 {
        let mut board = 0;
        bit_board::set_bit(&mut board, i);

        if bit_board::is_set(board & NOT_FILE_A, i) {
            mask[&i] |= board >> bit_board::NORTH >> bit_board::NO_WE;
        }
        if bit_board::is_set(board & NOT_FILE_H, i) {
            mask[&i] |= board >> bit_board::NORTH >> bit_board::NO_EA;
        }
        if bit_board::is_set(board & NOT_FILE_GH, i) {
            mask[&i] |= board << bit_board::EAST >> bit_board::NO_EA;
            mask[&i] |= board << bit_board::EAST << bit_board::SO_EA;
        }
        if bit_board::is_set(board & NOT_FILE_A, i) {
            mask[&i] |= board << bit_board::SOUTH << bit_board::SO_WE;
        }
        if bit_board::is_set(board & NOT_FILE_H, i) {
            mask[&i] |= board << bit_board::SOUTH << bit_board::SO_EA;
        }
        if bit_board::is_set(board & NOT_FILE_AB, i) {
            mask[&i] |= board >> bit_board::WEST << bit_board::SO_WE;
            mask[&i] |= board >> bit_board::WEST >> bit_board::NO_WE;
        }
    }

    mask
}

fn generate_pawn_move_mask() -> ColoredMovMask {
    let mut mask = ColoredMovMask::new();

    for i in 0..Board::SIZE as u64 {
        let mut board = 0;
        bit_board::set_bit(&mut board, i);

        if bit_board::is_set(board & NOT_FILE_A, i) {
            mask[Color::White][&i] |= board >> bit_board::NO_WE;
        }
        if bit_board::is_set(board & NOT_FILE_H, i) {
            mask[Color::White][&i] |= board >> bit_board::NO_EA;
        }

        if bit_board::is_set(board & NOT_FILE_A, i) {
            mask[Color::Black][&i] |= board << bit_board::SO_WE;
        }
        if bit_board::is_set(board & NOT_FILE_H, i) {
            mask[Color::Black][&i] |= board << bit_board::SO_EA;
        }
    }

    mask
}

fn generate_rook_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..Board::SIZE as u64 {
        let mut board = 0;
        bit_board::set_bit(&mut board, i);

        let file = i % Board::HEIGHT as u64;
        let rank = i / Board::HEIGHT as u64;

        let to_no_ea = u64::min((Board::WIDTH - 1) as u64 - file, rank);
        let to_so_ea = u64::min(Board::WIDTH as u64 - file, Board::HEIGHT as u64 - rank) - 1;
        let to_so_we = u64::min(file, (Board::HEIGHT - 1) as u64 - rank);
        let to_no_we = u64::min(file, rank);

        for iter in 1..to_no_ea {
            mask[&i] |= board >> bit_board::NO_EA * iter;
        }
        for iter in 1..to_so_ea {
            mask[&i] |= board << bit_board::SO_EA * iter;
        }
        for iter in 1..to_so_we {
            mask[&i] |= board << bit_board::SO_WE * iter;
        }
        for iter in 1..to_no_we {
            mask[&i] |= board >> bit_board::NO_WE * iter;
        }
    }

    mask
}

#[cfg(test)]
mod tests {
    use crate::{board::PieceInstance, display_board, Color};

    use super::*;

    #[test]
    fn bishop_moves_no_hit() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::Bishop));

        assert_moves_eq(
            &Piece::get_moves_for_bishop_at(36, &board),
            36,
            &[15, 22, 29, 43, 45, 50, 54, 57, 63, 0, 9, 18, 27],
        );
    }

    #[test]
    fn bishop_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set_by_idx(27, ins_white(Piece::Pawn));
        board.set_by_idx(29, ins_black(Piece::Pawn));
        board.set_by_idx(36, ins_white(Piece::Bishop));

        assert_moves_eq(
            &Piece::get_moves_for_bishop_at(36, &board),
            36,
            &[29, 43, 45, 50, 54, 57, 63],
        );
    }

    #[test]
    fn knight_moves_no_hit() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(36, &board),
            36,
            &[19, 21, 26, 30, 42, 51, 53, 46],
        );
    }

    #[test]
    fn king_moves_no_hit() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::King));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(36, &board),
            36,
            &[27, 28, 29, 35, 37, 43, 44, 45],
        );
    }

    #[test]
    fn king_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set_by_idx(35, ins_white(Piece::Pawn));
        board.set_by_idx(36, ins_white(Piece::King));
        board.set_by_idx(37, ins_black(Piece::Pawn));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(36, &board),
            36,
            &[27, 28, 29, 37, 43, 44, 45],
        );
    }

    #[test]
    fn king_moves_bottom_left() {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::King));

        assert_moves_eq(&Piece::get_moves_for_king_at(56, &board), 56, &[48, 49, 57]);
    }

    #[test]
    fn king_moves_top_right() {
        let mut board = board();
        board.set_by_idx(7, ins_white(Piece::King));

        assert_moves_eq(&Piece::get_moves_for_king_at(7, &board), 7, &[6, 14, 15]);
    }

    #[test]
    fn king_moves_castle_white_queen_side() {
        let mut board = board_king_moves_castle_white();

        board.do_move(&(63, 55));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(60, &board),
            60,
            &[51, 52, 53, 58, 59, 61],
        );
    }

    #[test]
    fn king_moves_castle_white_king_side() {
        let mut board = board_king_moves_castle_white();
        board.do_move(&(56, 48));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(60, &board),
            60,
            &[51, 52, 53, 59, 61, 62],
        );
    }

    #[test]
    fn king_moves_castle_black_queen_side() {
        let mut board = board_king_moves_castle_black();
        board.do_move(&(7, 15));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(4, &board),
            4,
            &[2, 3, 5, 11, 12, 13],
        );
    }

    #[test]
    fn king_moves_castle_black_king_side() {
        let mut board = board_king_moves_castle_black();
        board.do_move(&(0, 8));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(4, &board),
            4,
            &[3, 5, 6, 11, 12, 13],
        );
    }

    #[test]
    fn king_moves_castle_only_works_if_the_king_has_not_been_moved_yet() {
        let mut board = board_king_moves_castle_white();

        board.do_move(&(60, 52));
        board.do_move(&(52, 60));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(60, &board),
            60,
            &[51, 52, 53, 59, 61],
        );
    }

    #[test]
    fn king_moves_castle_only_works_if_the_rooks_have_not_been_moved_yet() {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::Rook));
        board.set_by_idx(60, ins_white(Piece::King));
        board.set_by_idx(63, ins_white(Piece::Rook));

        board.do_move(&(56, 48));
        board.do_move(&(48, 56));

        board.do_move(&(63, 55));
        board.do_move(&(55, 63));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(60, &board),
            60,
            &[51, 52, 53, 59, 61],
        );
    }

    #[test]
    fn king_moves_castle_only_works_if_no_square_between_is_attacked_white_queen_side() {
        let board = board_king_moves_castle_white();

        for i in 49..52 {
            let mut board = board.clone();
            board.set_by_idx(i, ins_black(Piece::Pawn));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 59, 61, 62],
            );
        }
    }

    #[test]
    fn king_moves_castle_only_works_if_no_square_between_is_attacked_white_king_side() {
        let board = board_king_moves_castle_white();

        for i in 53..55 {
            let mut board = board.clone();
            board.set_by_idx(i, ins_black(Piece::Pawn));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 58, 59, 61],
            );
        }
    }

    #[test]
    fn king_moves_castle_only_works_if_no_square_between_is_attacked_black_queen_side() {
        let board = board_king_moves_castle_black();

        for i in 9..12 {
            let mut board = board.clone();
            board.set_by_idx(i, ins_white(Piece::Pawn));
            println!("{}", board);

            assert_moves_eq(
                &Piece::get_moves_for_king_at(4, &board),
                4,
                &[3, 5, 6, 11, 12, 13],
            );
        }
    }

    #[test]
    fn king_moves_castle_only_works_if_no_square_between_is_attacked_black_king_side() {
        let board = board_king_moves_castle_black();

        for i in 13..15 {
            let mut board = board.clone();
            board.set_by_idx(i, ins_white(Piece::Pawn));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(4, &board),
                4,
                &[2, 3, 5, 11, 12, 13],
            );
        }
    }

    #[test]
    fn king_moves_castle_does_not_work_if_piece_on_square_between_white_queen_side() {
        let board = board_king_moves_castle_white();

        // The following tests are not in a loop, as in the last variant, the blocking
        // pice is right next to the king, and the resulting moves are different.
        {
            let mut board = board.clone();
            board.set_by_idx(57, ins_white(Piece::Rook));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 59, 61, 62],
            );
        }

        {
            let mut board = board.clone();
            board.set_by_idx(58, ins_white(Piece::Rook));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 59, 61, 62],
            );
        }

        {
            let mut board = board.clone();
            board.set_by_idx(59, ins_white(Piece::Rook));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 61, 62],
            );
        }
    }

    #[test]
    fn king_moves_castle_does_not_work_if_piece_on_square_between_white_king_side() {
        let board = board_king_moves_castle_white();

        // The following tests are not in a loop, as in the first variant, the blocking
        // piece is right next to the king, and the resulting moves are different.
        {
            let mut board = board.clone();
            board.set_by_idx(61, ins_white(Piece::Rook));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 58, 59],
            );
        }

        {
            let mut board = board.clone();
            board.set_by_idx(62, ins_white(Piece::Rook));

            assert_moves_eq(
                &Piece::get_moves_for_king_at(60, &board),
                60,
                &[51, 52, 53, 58, 59, 61],
            );
        }
    }

    fn board_king_moves_castle_white() -> Board {
        let mut board = board();
        board.can_white_castle_king_side = true;
        board.can_white_castle_queen_side = true;

        board.set_by_idx(56, ins_white(Piece::Rook));
        board.set_by_idx(60, ins_white(Piece::King));
        board.set_by_idx(63, ins_white(Piece::Rook));

        board
    }

    fn board_king_moves_castle_black() -> Board {
        let mut board = board();
        board.can_black_castle_king_side = true;
        board.can_black_castle_queen_side = true;

        board.set_by_idx(0, ins_black(Piece::Rook));
        board.set_by_idx(4, ins_black(Piece::King));
        board.set_by_idx(7, ins_black(Piece::Rook));

        board
    }

    #[test]
    fn knight_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set_by_idx(19, ins_white(Piece::Pawn));
        board.set_by_idx(21, ins_black(Piece::Pawn));
        board.set_by_idx(36, ins_white(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(36, &board),
            36,
            &[21, 26, 30, 42, 51, 53, 46],
        );
    }

    #[test]
    fn knight_moves_top_left() {
        let mut board = board();
        board.set_by_idx(0, ins_white(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(0, &board), 0, &[10, 17]);
    }

    #[test]
    fn knight_moves_top_right() {
        let mut board = board();
        board.set_by_idx(7, ins_white(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(7, &board), 7, &[13, 22]);
    }

    #[test]
    fn knight_moves_bottom_left() {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(56, &board), 56, &[41, 50]);
    }

    #[test]
    fn knight_moves_bottom_right() {
        let mut board = board();
        board.set_by_idx(63, ins_white(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(63, &board), 63, &[46, 53]);
    }

    #[test]
    fn knight_moves_1_offset_top_right() {
        let mut board = board();
        board.set_by_idx(9, ins_white(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(9, &board),
            9,
            &[3, 19, 24, 26],
        );
    }

    #[test]
    fn knight_moves_1_offset_bottom_right() {
        let mut board = board();
        board.set_by_idx(54, ins_white(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(54, &board),
            54,
            &[37, 39, 44, 60],
        );
    }

    #[test]
    fn pawn_moves_not_moved_yet() {
        let mut board = board();
        board.set_by_idx(48, ins_white(Piece::Pawn));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(48, &board), 48, &[32, 40]);
    }

    #[test]
    fn pawn_moves_has_moved_west() {
        let mut board = board();
        // This piece will be hit if the bounds check is not done correctly.
        board.set_by_idx(31, ins_black(Piece::Pawn));
        board.set_by_idx(40, ins_white(Piece::Pawn));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(40, &board), 40, &[32]);
    }

    #[test]
    fn pawn_moves_has_moved_east() {
        let mut board = board();
        // This piece will be hit if the bounds check is not done correctly.
        board.set_by_idx(40, ins_black(Piece::Pawn));
        board.set_by_idx(47, ins_white(Piece::Pawn));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(47, &board), 47, &[39]);
    }

    #[test]
    fn pawn_moves_en_passant_you_west() {
        let mut board = board();
        board.set_by_idx(9, ins_black(Piece::Pawn));
        board.set_by_idx(24, ins_white(Piece::Pawn));

        board.do_move(&(9, 25));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(24, &board), 24, &[16, 17]);
    }

    #[test]
    fn pawn_moves_en_passant_you_east() {
        let mut board = board();
        board.set_by_idx(9, ins_black(Piece::Pawn));
        board.set_by_idx(26, ins_white(Piece::Pawn));

        board.do_move(&(9, 25));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(26, &board), 26, &[17, 18]);
    }

    #[test]
    fn pawn_moves_en_passant_opponent_west() {
        let mut board = board();
        board.set_by_idx(32, ins_black(Piece::Pawn));
        board.set_by_idx(49, ins_white(Piece::Pawn));

        board.do_move(&(49, 33));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(32, &board), 32, &[40, 41]);
    }

    #[test]
    fn pawn_moves_en_passant_opponent_east() {
        let mut board = board();
        board.set_by_idx(34, ins_black(Piece::Pawn));
        board.set_by_idx(49, ins_white(Piece::Pawn));

        board.do_move(&(49, 33));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(34, &board), 34, &[41, 42]);
    }

    #[test]
    fn pawn_move_en_passant_can_only_be_done_in_the_turn_immediately_after() {
        let mut board = board();
        board.set_by_idx(34, ins_black(Piece::Pawn));
        board.set_by_idx(49, ins_white(Piece::Pawn));
        board.set_by_idx(63, ins_white(Piece::King));

        board.do_move(&(49, 33));
        board.do_move(&(63, 55));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(34, &board), 34, &[42]);
    }

    #[test]
    fn pawn_moves_en_passant_is_not_added_for_other_pieces() {
        let mut board = board();
        board.set_by_idx(1, ins_white(Piece::Knight));
        board.set_by_idx(19, ins_black(Piece::Pawn));

        board.do_move(&(1, 18));

        assert!(
            !Piece::is_valid_move((19, 26), &board),
            "en passant was added for knight"
        );
    }

    #[test]
    fn pawn_moves_en_passant_is_not_added_for_pieces_of_same_color() {
        let mut board = board();
        board.set_by_idx(9, ins_black(Piece::Pawn));
        board.set_by_idx(26, ins_black(Piece::Pawn));

        board.do_move(&(9, 25));

        assert!(
            !Piece::is_valid_move((26, 33), &board),
            "en passant was added for piece of same color"
        );
    }

    #[test]
    fn pawn_moves_hit_west_and_east() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::Pawn));
        board.set_by_idx(27, ins_black(Piece::Pawn));
        board.set_by_idx(29, ins_black(Piece::Pawn));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(36, &board), 36, &[27, 28, 29]);
    }

    #[test]
    fn queen_moves_no_hit() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::Queen));

        assert_moves_eq(
            &Piece::get_moves_for_queen_at(36, &board),
            36,
            &[
                0, 4, 9, 12, 15, 18, 20, 22, 27, 28, 29, 32, 33, 34, 35, 37, 38, 39, 43, 44, 45,
                50, 52, 54, 57, 60, 63,
            ],
        );
    }

    #[test]
    fn queen_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set_by_idx(35, ins_white(Piece::Pawn));
        board.set_by_idx(36, ins_white(Piece::Queen));
        board.set_by_idx(37, ins_black(Piece::Pawn));

        assert_moves_eq(
            &Piece::get_moves_for_queen_at(36, &board),
            36,
            &[
                0, 4, 9, 12, 15, 18, 20, 22, 27, 28, 29, 37, 43, 44, 45, 50, 52, 54, 57, 60, 63,
            ],
        );
    }

    #[test]
    fn rook_moves_no_hit() {
        let mut board = board();
        board.set_by_idx(36, ins_white(Piece::Rook));

        assert_moves_eq(
            &Piece::get_moves_for_rook_at(36, &board),
            36,
            &[4, 12, 20, 28, 32, 33, 34, 35, 37, 38, 39, 44, 52, 60],
        );
    }

    #[test]
    fn rook_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set_by_idx(35, ins_white(Piece::Pawn));
        board.set_by_idx(36, ins_white(Piece::Rook));
        board.set_by_idx(37, ins_black(Piece::Pawn));

        assert_moves_eq(
            &Piece::get_moves_for_rook_at(36, &board),
            36,
            &[4, 12, 20, 28, 37, 44, 52, 60],
        );
    }

    fn assert_moves_eq(to_check: &[MoveByIdx], src_idx: usize, expected: &[usize]) {
        let mut to_check = to_check.to_owned();
        to_check.sort();
        let mut expected: Vec<MoveByIdx> =
            expected.iter().map(|&to_idx| (src_idx, to_idx)).collect();
        expected.sort();

        assert_eq!(
            to_check,
            expected,
            "expected {}\nto equal{}",
            display_moves(&to_check),
            display_moves(&expected)
        );
    }

    fn board() -> Board {
        let mut board = Board::new_empty();
        board.can_black_castle_king_side = false;
        board.can_black_castle_queen_side = false;
        board.can_white_castle_king_side = false;
        board.can_white_castle_queen_side = false;
        board
    }

    fn ins_black(piece: Piece) -> Option<PieceInstance> {
        ins(Color::Black, piece)
    }

    fn ins_white(piece: Piece) -> Option<PieceInstance> {
        ins(Color::White, piece)
    }

    fn ins(color: Color, piece: Piece) -> Option<PieceInstance> {
        Some(PieceInstance::new(color, piece))
    }

    fn display_moves(moves: &[MoveByIdx]) -> String {
        let moves: Vec<_> = moves.iter().map(|(_, to)| to).collect();

        let mut out = String::new();

        for idx in 0..Board::SIZE {
            if idx % Board::WIDTH == 0 {
                out.push('\n');
            }

            let bg_color = display_board::get_bg_color_of(idx);
            let value = if moves.contains(&&idx) { "*" } else { "" };

            out.push_str(&format!(
                "{}{: ^4}{}",
                bg_color,
                value,
                display_board::RESET_ANSI,
            ));
        }

        out
    }
}
