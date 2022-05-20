use crate::{
    board::{self, Move},
    Board, Player,
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

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    Bishop,
    King,
    Knight,
    Pawn,
    Queen,
    Rook,
}

impl Piece {
    fn get_moves_for_bishop_at(idx: usize, board: &Board) -> Vec<Move> {
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

    fn get_moves_for_king_at(idx: usize, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let north = (idx as i8 + DIR_OFFSETS[DIR_NORTH]) as usize;
        let north_east = (idx as i8 + DIR_OFFSETS[DIR_NORTH_EAST]) as usize;
        let east = (idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize;
        let south_east = (idx as i8 + DIR_OFFSETS[DIR_SOUTH_EAST]) as usize;
        let south = (idx as i8 + DIR_OFFSETS[DIR_SOUTH]) as usize;
        let south_west = (idx as i8 + DIR_OFFSETS[DIR_SOUTH_WEST]) as usize;
        let west = (idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize;
        let north_west = (idx as i8 + DIR_OFFSETS[DIR_NORTH_WEST]) as usize;

        if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
            Self::push_move_or_hit(idx, north, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_NORTH_EAST] > 0 {
            Self::push_move_or_hit(idx, north_east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
            Self::push_move_or_hit(idx, east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH_EAST] > 0 {
            Self::push_move_or_hit(idx, south_east, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
            Self::push_move_or_hit(idx, south, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH_WEST] > 0 {
            Self::push_move_or_hit(idx, south_west, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
            Self::push_move_or_hit(idx, west, &mut moves, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_NORTH_WEST] > 0 {
            Self::push_move_or_hit(idx, north_west, &mut moves, board);
        }

        moves
    }

    fn get_moves_for_knight_at(idx: usize, board: &Board) -> Vec<Move> {
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

    fn get_moves_for_pawn_at(idx: usize, board: &Board) -> Vec<Move> {
        let pawn_ins = match board.get(idx) {
            Some(i) => i,
            None => panic!("no piece found at the position '{}'", idx),
        };
        let mut moves = Vec::new();

        let dir = match Self::get_pawn_direction(pawn_ins) {
            -1 => DIR_NORTH,
            1 => DIR_SOUTH,
            unknown => panic!("unsupported direction {}", unknown),
        };
        let dir_offset = DIR_OFFSETS[dir];

        Self::push_move_if_empty(idx, (idx as i8 + dir_offset) as usize, &mut moves, board);
        if !pawn_ins.was_moved {
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

        moves
    }

    fn get_moves_for_queen_at(idx: usize, board: &Board) -> Vec<Move> {
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

    fn get_moves_for_rook_at(idx: usize, board: &Board) -> Vec<Move> {
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
    fn get_moves_for_sliding_piece_at(src_idx: usize, dir: usize, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        for offset in 0..TO_EDGE_OFFSETS[src_idx][dir] {
            let hit_idx = (src_idx as i8 + (offset + 1) as i8 * DIR_OFFSETS[dir]) as usize;

            if Self::push_move_or_hit(src_idx, hit_idx, &mut moves, board) {
                break;
            }
        }

        moves
    }

    pub fn get_moves_of_piece_at(idx: usize, board: &Board) -> Vec<Move> {
        let ins = match board.get(idx) {
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

    // TODO: This is not at the right place. Check if this can be moved somewhere
    // else.
    // As stated in the comment bellow, this will break as soon as `you` is not at
    // the bottom of the board. Thus this should probably be inside the [`Board`].
    // TODO: additionally, this could be changed to return the cardinal directions.
    pub fn get_pawn_direction(ins: &board::PieceInstance) -> i8 {
        // Currently it is assumed that you are at the bottom of the board.
        // In case this assumption is false in the future, this code WILL not work.
        match ins.player {
            Player::You => -1,
            Player::Opponent => 1,
        }
    }

    pub fn get_symbol(piece: &Self) -> &str {
        match piece {
            Self::Bishop => "BI",
            Self::King => "KI",
            Self::Knight => "KN",
            Self::Pawn => "PA",
            Self::Queen => "QU",
            Self::Rook => "RO",
        }
    }

    fn push_move_if_empty(src_idx: usize, hit_idx: usize, moves: &mut Vec<Move>, board: &Board) {
        if board.get(hit_idx).is_some() {
            return;
        }

        moves.push((src_idx, hit_idx));
    }

    fn push_move_if_opponent(src_idx: usize, hit_idx: usize, moves: &mut Vec<Move>, board: &Board) {
        let src_ins = match board.get(src_idx) {
            Some(i) => i,
            None => panic!("could not find src piece at '{}'", src_idx),
        };
        let hit_ins = match board.get(hit_idx) {
            Some(i) => i,
            // No piece at all is clearly not an opponent, thus just return.
            None => return,
        };

        if src_ins.player == hit_ins.player {
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
        moves: &mut Vec<Move>,
        board: &Board,
    ) -> bool {
        let src_ins = board
            .get(src_idx)
            .as_ref()
            .unwrap_or_else(|| panic!("no src piece at idx '{}'", src_idx));

        if let Some(hit_ins) = board.get(hit_idx) {
            // You can't really take your own piece, thus the player need to be
            // different for a move to be added.
            if src_ins.player != hit_ins.player {
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

#[cfg(test)]
mod tests {
    use crate::{display_board, Color, board::PieceInstance};

    use super::*;

    #[test]
    fn bishop_moves_no_hit() {
        let mut board = board();
        board.set(36, ins_you(Piece::Bishop));

        assert_moves_eq(
            &Piece::get_moves_for_bishop_at(36, &board),
            36,
            &[15, 22, 29, 43, 45, 50, 54, 57, 63, 0, 9, 18, 27],
        );
    }

    #[test]
    fn bishop_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set(27, ins_you(Piece::Pawn));
        board.set(29, ins_opp(Piece::Pawn));
        board.set(36, ins_you(Piece::Bishop));

        assert_moves_eq(
            &Piece::get_moves_for_bishop_at(36, &board),
            36,
            &[29, 43, 45, 50, 54, 57, 63],
        );
    }

    #[test]
    fn king_moves_no_hit() {
        let mut board = board();
        board.set(36, ins_you(Piece::King));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(36, &board),
            36,
            &[27, 28, 29, 35, 37, 43, 44, 45],
        );
    }

    #[test]
    fn king_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set(35, ins_you(Piece::Pawn));
        board.set(36, ins_you(Piece::King));
        board.set(37, ins_opp(Piece::Pawn));

        assert_moves_eq(
            &Piece::get_moves_for_king_at(36, &board),
            36,
            &[27, 28, 29, 37, 43, 44, 45],
        );
    }

    #[test]
    fn king_moves_bottom_left() {
        let mut board = board();
        board.set(56, ins_you(Piece::King));

        assert_moves_eq(&Piece::get_moves_for_king_at(56, &board), 56, &[48, 49, 57]);
    }

    #[test]
    fn king_moves_top_right() {
        let mut board = board();
        board.set(7, ins_you(Piece::King));

        assert_moves_eq(&Piece::get_moves_for_king_at(7, &board), 7, &[6, 14, 15]);
    }

    #[test]
    fn knight_moves_no_hit() {
        let mut board = board();
        board.set(36, ins_you(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(36, &board),
            36,
            &[19, 21, 26, 30, 42, 51, 53, 46],
        );
    }

    #[test]
    fn knight_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set(19, ins_you(Piece::Pawn));
        board.set(21, ins_opp(Piece::Pawn));
        board.set(36, ins_you(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(36, &board),
            36,
            &[21, 26, 30, 42, 51, 53, 46],
        );
    }

    #[test]
    fn knight_moves_top_left() {
        let mut board = board();
        board.set(0, ins_you(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(0, &board), 0, &[10, 17]);
    }

    #[test]
    fn knight_moves_top_right() {
        let mut board = board();
        board.set(7, ins_you(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(7, &board), 7, &[13, 22]);
    }

    #[test]
    fn knight_moves_bottom_left() {
        let mut board = board();
        board.set(56, ins_you(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(56, &board), 56, &[41, 50]);
    }

    #[test]
    fn knight_moves_bottom_right() {
        let mut board = board();
        board.set(63, ins_you(Piece::Knight));

        assert_moves_eq(&Piece::get_moves_for_knight_at(63, &board), 63, &[46, 53]);
    }

    #[test]
    fn knight_moves_1_offset_top_right() {
        let mut board = board();
        board.set(9, ins_you(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(9, &board),
            9,
            &[3, 19, 24, 26],
        );
    }

    #[test]
    fn knight_moves_1_offset_bottom_right() {
        let mut board = board();
        board.set(54, ins_you(Piece::Knight));

        assert_moves_eq(
            &Piece::get_moves_for_knight_at(54, &board),
            54,
            &[37, 39, 44, 60],
        );
    }

    #[test]
    fn pawn_moves_not_moved_yet() {
        let pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
        let mut board = board();
        board.set(48, Some(pawn_ins));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(48, &board), 48, &[32, 40]);
    }

    #[test]
    fn pawn_moves_has_moved_west() {
        let mut pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
        pawn_ins.was_moved = true;

        let mut board = board();
        // This piece will be hit if the bounds check is not done correctly.
        board.set(31, ins_opp(Piece::Pawn));
        board.set(40, Some(pawn_ins));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(40, &board), 40, &[32]);
    }

    #[test]
    fn pawn_moves_has_moved_east() {
        let mut pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
        pawn_ins.was_moved = true;

        let mut board = board();
        // This piece will be hit if the bounds check is not done correctly.
        board.set(40, ins_opp(Piece::Pawn));
        board.set(47, Some(pawn_ins));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(47, &board), 47, &[39]);
    }

    #[test]
    fn pawn_moves_hit_west_and_east() {
        let mut board = board();
        board.set(36, ins_you(Piece::Pawn));
        board.set(27, ins_opp(Piece::Pawn));
        board.set(29, ins_opp(Piece::Pawn));

        assert_moves_eq(&Piece::get_moves_for_pawn_at(36, &board), 36, &[27, 28, 29]);
    }

    #[test]
    fn queen_moves_no_hit() {
        let mut board = board();
        board.set(36, ins_you(Piece::Queen));

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
        board.set(35, ins_you(Piece::Pawn));
        board.set(36, ins_you(Piece::Queen));
        board.set(37, ins_opp(Piece::Pawn));

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
        board.set(36, ins_you(Piece::Rook));

        assert_moves_eq(
            &Piece::get_moves_for_rook_at(36, &board),
            36,
            &[4, 12, 20, 28, 32, 33, 34, 35, 37, 38, 39, 44, 52, 60],
        );
    }

    #[test]
    fn rook_moves_hit_ally_and_opponent() {
        let mut board = board();
        board.set(35, ins_you(Piece::Pawn));
        board.set(36, ins_you(Piece::Rook));
        board.set(37, ins_opp(Piece::Pawn));

        assert_moves_eq(
            &Piece::get_moves_for_rook_at(36, &board),
            36,
            &[4, 12, 20, 28, 37, 44, 52, 60],
        );
    }

    fn assert_moves_eq(to_check: &[Move], src_idx: usize, expected: &[usize]) {
        let mut to_check = to_check.to_owned();
        to_check.sort();
        let mut expected: Vec<Move> = expected.iter().map(|&to_idx| (src_idx, to_idx)).collect();
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
        Board::new(Color::Black, Color::White)
    }

    fn ins_opp(piece: Piece) -> Option<PieceInstance> {
        let mut ins = PieceInstance::new(Player::Opponent, piece);
        ins.was_moved = true;

        Some(ins)
    }

    fn ins_you(piece: Piece) -> Option<PieceInstance> {
        let mut ins = PieceInstance::new(Player::You, piece);
        ins.was_moved = true;

        Some(ins)
    }

    fn display_moves(moves: &[Move]) -> String {
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
