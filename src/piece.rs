use crate::{
    board::{self, PieceInstance},
    info_board::{self, PosInfo},
    Board, InfoBoard, Player,
};

// TODO: Investigate if this constants should really be defined here.

pub type ToEdgeOffset = [[u8; 8]; Board::SIZE as usize];

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
    fn add_bishop_moves_to_board(idx: usize, board: &mut InfoBoard) {
        Self::add_sliding_piece_moves(idx, DIR_NORTH_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH_WEST, board);
        Self::add_sliding_piece_moves(idx, DIR_NORTH_WEST, board);
    }

    fn add_castle_moves(king_idx: usize, board: &mut InfoBoard) {
        let king_ins = match board.get(king_idx) {
            // TODO: castle not allowed when piece is in check
            PosInfo::Piece(ins) | PosInfo::PieceHit(ins) => ins,
            _ => panic!("could not find king at '{}' in castle check", king_idx),
        };

        // TODO: castling can only be done when no square up to the rook is blocked
        // TODO: castling can only be done when no square up to the rook is challenged

        // Castle is only possible when the king hasn't been moved yet.
        if king_ins.was_moved {
            return;
        }

        let west_pos = (king_idx as i8
            + TO_EDGE_OFFSETS[king_idx][DIR_WEST] as i8 * DIR_OFFSETS[DIR_WEST])
            as usize;

        if let PosInfo::Piece(west_rook_ins) = board.get(west_pos) {
            // Castle is only possible when the rook hasn't been moved yet.
            if !west_rook_ins.was_moved {
                Self::add_move_if_empty(
                    king_idx,
                    (king_idx as i8 + DIR_OFFSETS[DIR_WEST] * 2) as usize,
                    board,
                );
            }
        }

        let east_pos = (king_idx as i8
            + TO_EDGE_OFFSETS[king_idx][DIR_EAST] as i8 * DIR_OFFSETS[DIR_EAST])
            as usize;

        if let PosInfo::Piece(east_rook_ins) = board.get(east_pos) {
            // Castle is only possible when the rook hasn't been moved yet.
            if !east_rook_ins.was_moved {
                Self::add_move_if_empty(
                    king_idx,
                    (king_idx as i8 + DIR_OFFSETS[DIR_EAST] * 2) as usize,
                    board,
                );
            }
        }
    }

    pub fn add_en_passant_moves_to_board(
        piece_idx: usize,
        ins: &PieceInstance,
        board: &mut InfoBoard,
    ) {
        let dir = match Piece::get_pawn_direction(ins) {
            -1 => DIR_NORTH,
            _ => DIR_SOUTH,
        } as usize;

        let east_idx = (piece_idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize;

        println!("{} {}", east_idx, dir);

        if matches!(board.get(east_idx), PosInfo::Piece(_)) {
            board.set((east_idx as i8 + DIR_OFFSETS[dir]) as usize, PosInfo::Move);
        }

        let west_idx = (piece_idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize;
        if matches!(board.get(west_idx), PosInfo::Piece(_)) {
            board.set((west_idx as i8 + DIR_OFFSETS[dir]) as usize, PosInfo::Move);
        }
    }

    fn add_hit_if_piece_is_enemy(orig_idx: usize, hit_idx: usize, board: &mut InfoBoard) -> bool {
        let orig_ins = match board.get(orig_idx) {
            PosInfo::Piece(ins) => ins,
            info => panic!("expected info to be piece but was '{:?}'", info),
        };
        let hit_ins = match board.get(hit_idx) {
            PosInfo::Piece(ins) => ins,
            _ => return false,
        };

        if orig_ins.player == hit_ins.player {
            return false;
        }

        match board.take(hit_idx) {
            PosInfo::Piece(ins) => {
                board.set(hit_idx, PosInfo::PieceHit(ins));
            }
            _ => (),
        }

        false
    }

    fn add_king_moves_to_board(idx: usize, board: &mut InfoBoard) {
        let north = (idx as i8 + DIR_OFFSETS[DIR_NORTH]) as usize;
        let north_east = (idx as i8 + DIR_OFFSETS[DIR_NORTH_EAST]) as usize;
        let east = (idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize;
        let south_east = (idx as i8 + DIR_OFFSETS[DIR_SOUTH_EAST]) as usize;
        let south = (idx as i8 + DIR_OFFSETS[DIR_SOUTH]) as usize;
        let south_west = (idx as i8 + DIR_OFFSETS[DIR_SOUTH_WEST]) as usize;
        let west = (idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize;
        let north_west = (idx as i8 + DIR_OFFSETS[DIR_NORTH_WEST]) as usize;

        if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
            Self::add_move_or_hit_by_idx(idx, north, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_NORTH_EAST] > 0 {
            Self::add_move_or_hit_by_idx(idx, north_east, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
            Self::add_move_or_hit_by_idx(idx, east, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH_EAST] > 0 {
            Self::add_move_or_hit_by_idx(idx, south_east, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
            Self::add_move_or_hit_by_idx(idx, south, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH_WEST] > 0 {
            Self::add_move_or_hit_by_idx(idx, south_west, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
            Self::add_move_or_hit_by_idx(idx, west, board);
        }
        if TO_EDGE_OFFSETS[idx][DIR_NORTH_WEST] > 0 {
            Self::add_move_or_hit_by_idx(idx, north_west, board);
        }

        Self::add_castle_moves(idx, board);
    }

    fn add_knight_moves_to_board(idx: usize, board: &mut InfoBoard) {
        if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] * 2 + DIR_OFFSETS[DIR_WEST]) as usize,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] * 2 + DIR_OFFSETS[DIR_EAST]) as usize,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_EAST] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] + DIR_OFFSETS[DIR_EAST] * 2) as usize,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] + DIR_OFFSETS[DIR_EAST] * 2) as usize,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_EAST] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] * 2 + DIR_OFFSETS[DIR_EAST]) as usize,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_WEST] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] * 2 + DIR_OFFSETS[DIR_WEST]) as usize,
                    board,
                );
            }
        }
        if TO_EDGE_OFFSETS[idx][DIR_WEST] > 1 {
            if TO_EDGE_OFFSETS[idx][DIR_SOUTH] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_SOUTH] + DIR_OFFSETS[DIR_WEST] * 2) as usize,
                    board,
                );
            }
            if TO_EDGE_OFFSETS[idx][DIR_NORTH] > 0 {
                Self::add_move_or_hit_by_idx(
                    idx,
                    (idx as i8 + DIR_OFFSETS[DIR_NORTH] + DIR_OFFSETS[DIR_WEST] * 2) as usize,
                    board,
                );
            }
        }
    }

    fn add_move_if_empty(_orig_idx: usize, hit_idx: usize, board: &mut InfoBoard) -> bool {
        if matches!(board.get(hit_idx), PosInfo::None) {
            board.set(hit_idx, PosInfo::Move);
            return true;
        }

        false
    }

    fn add_move_or_hit_by_idx(orig_idx: usize, hit_idx: usize, board: &mut InfoBoard) -> bool {
        let orig_ins = match board.get(orig_idx) {
            PosInfo::Piece(ins) => ins,
            _ => panic!(),
        };

        if let PosInfo::Piece(target_ins) = board.get(hit_idx) {
            if orig_ins.player == target_ins.player {
                return true;
            }

            let new_ins = PosInfo::PieceHit(target_ins.clone());
            board.set(hit_idx, new_ins);

            return true;
        }

        board.set(hit_idx, PosInfo::Move);

        false
    }

    pub fn add_moves_for_piece(idx: usize, board: &mut InfoBoard) {
        let ins = match board.get(idx) {
            info_board::PosInfo::Piece(piece) | info_board::PosInfo::PieceHit(piece) => {
                piece.clone()
            }
            info => panic!(
                "can only add moves for pieces, but piece at '{}' was {:?}",
                idx, info
            ),
        };

        match &ins.piece {
            Self::Bishop => Self::add_bishop_moves_to_board(idx, board),
            Self::King => Self::add_king_moves_to_board(idx, board),
            Self::Knight => Self::add_knight_moves_to_board(idx, board),
            Self::Pawn => Self::add_pawn_moves_to_board(idx, &ins, board),
            Self::Queen => Self::add_queen_moves_to_board(idx, board),
            Self::Rook => Self::add_rook_moves_to_board(idx, board),
        }
    }

    fn add_pawn_moves_to_board(idx: usize, ins: &board::PieceInstance, board: &mut InfoBoard) {
        let (dir_offset_left, dir_offset, dir_offset_right) = match Self::get_pawn_direction(ins) {
            -1 => (
                DIR_OFFSETS[DIR_NORTH_WEST],
                DIR_OFFSETS[DIR_NORTH],
                DIR_OFFSETS[DIR_NORTH_EAST],
            ),
            1 => (
                DIR_OFFSETS[DIR_SOUTH_WEST],
                DIR_OFFSETS[DIR_SOUTH],
                DIR_OFFSETS[DIR_SOUTH_EAST],
            ),
            unknown => panic!("unsupported direction {}", unknown),
        };

        Self::add_move_if_empty(idx, (idx as i8 + dir_offset) as usize, board);
        if !ins.was_moved {
            Self::add_move_if_empty(idx, (idx as i8 + dir_offset * 2) as usize, board);
        }

        Self::add_hit_if_piece_is_enemy(idx, (idx as i8 + dir_offset_left) as usize, board);
        Self::add_hit_if_piece_is_enemy(idx, (idx as i8 + dir_offset_right) as usize, board);
    }

    fn add_queen_moves_to_board(idx: usize, board: &mut InfoBoard) {
        Self::add_sliding_piece_moves(idx, DIR_NORTH, board);
        Self::add_sliding_piece_moves(idx, DIR_NORTH_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH_WEST, board);
        Self::add_sliding_piece_moves(idx, DIR_WEST, board);
        Self::add_sliding_piece_moves(idx, DIR_NORTH_WEST, board);
    }

    fn add_rook_moves_to_board(idx: usize, board: &mut InfoBoard) {
        Self::add_sliding_piece_moves(idx, DIR_NORTH, board);
        Self::add_sliding_piece_moves(idx, DIR_EAST, board);
        Self::add_sliding_piece_moves(idx, DIR_SOUTH, board);
        Self::add_sliding_piece_moves(idx, DIR_WEST, board);
    }

    fn add_sliding_piece_moves(piece_idx: usize, dir: usize, board: &mut InfoBoard) {
        for move_idx in 0..TO_EDGE_OFFSETS[piece_idx][dir] {
            if Self::add_move_or_hit_by_idx(
                piece_idx,
                (piece_idx as i8 + (move_idx + 1) as i8 * DIR_OFFSETS[dir]) as usize,
                board,
            ) {
                return;
            }
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
}

const fn generate_to_edge_map() -> ToEdgeOffset {
    const fn min(left: usize, right: usize) -> u8 {
        if left < right {
            return left as u8;
        }

        right as u8
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

            map[idx][DIR_NORTH] = to_north as u8;
            map[idx][DIR_NORTH_EAST] = min(to_north, to_east) as u8;
            map[idx][DIR_EAST] = to_east as u8;
            map[idx][DIR_SOUTH_EAST] = min(to_south, to_east);
            map[idx][DIR_SOUTH] = to_south as u8;
            map[idx][DIR_SOUTH_WEST] = min(to_south, to_west);
            map[idx][DIR_WEST] = to_west as u8;
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
    use crate::Color;

    use super::*;

    #[test]
    fn bishop_moves() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(37, ins(Player::You, Piece::Bishop));

        {
            let mut board = board.clone();
            Piece::add_bishop_moves_to_board(37, &mut board);

            assert_moves(&board, &[1, 10, 19, 23, 28, 30, 44, 46, 51, 55, 58]);
        }

        {
            let mut board = board.clone();
            board.set(28, ins(Player::You, Piece::Pawn));
            board.set(30, ins(Player::Opponent, Piece::Pawn));

            Piece::add_bishop_moves_to_board(37, &mut board);

            assert_moves(&board, &[30, 44, 46, 51, 55, 58]);
        }
    }

    #[test]
    fn king_moves() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(36, ins(Player::You, Piece::King));

        {
            let mut board = board.clone();
            Piece::add_king_moves_to_board(36, &mut board);

            assert_moves(&board, &[27, 28, 29, 35, 37, 43, 44, 45]);
        }

        {
            let mut board = board.clone();
            board.set(35, ins(Player::You, Piece::Pawn));
            board.set(37, ins(Player::Opponent, Piece::Pawn));

            Piece::add_king_moves_to_board(36, &mut board);

            assert_moves(&board, &[27, 28, 29, 37, 43, 44, 45]);
        }

        {
            // Check that moves are correctly added in the bottom left corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(56, ins(Player::You, Piece::King));

            Piece::add_king_moves_to_board(56, &mut board);

            assert_moves(&board, &[48, 49, 57]);
        }

        {
            // Check that moves are correctly added in the top right corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(7, ins(Player::You, Piece::King));

            Piece::add_king_moves_to_board(7, &mut board);

            assert_moves(&board, &[6, 14, 15]);
        }
    }

    #[test]
    fn king_moves_castle_king_was_moves() {
        let mut king_ins = PieceInstance::new(Player::You, Piece::King);
        king_ins.was_moved = true;
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::Rook));
        board.set(60, PosInfo::Piece(king_ins));
        board.set(63, ins(Player::You, Piece::Rook));

        Piece::add_king_moves_to_board(60, &mut board);

        assert_moves(&board, &[51, 52, 53, 59, 61]);
    }

    #[test]
    fn king_moves_castle_west_rook_was_moved() {
        let mut west_rook = PieceInstance::new(Player::You, Piece::Rook);
        west_rook.was_moved = true;
        let east_rook = PieceInstance::new(Player::You, Piece::Rook);
        let mut board = InfoBoard::new(Color::Black, Color::White);

        board.set(56, PosInfo::Piece(west_rook));
        board.set(
            60,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::King)),
        );
        board.set(63, PosInfo::Piece(east_rook));

        Piece::add_king_moves_to_board(60, &mut board);

        assert_moves(&board, &[51, 52, 53, 59, 61, 62]);
    }

    #[test]
    fn king_moves_castle_east_rook_was_moved() {
        let west_rook = PieceInstance::new(Player::You, Piece::Rook);
        let mut east_rook = PieceInstance::new(Player::You, Piece::Rook);
        east_rook.was_moved = true;
        let mut board = InfoBoard::new(Color::Black, Color::White);

        board.set(56, PosInfo::Piece(west_rook));
        board.set(
            60,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::King)),
        );
        board.set(63, PosInfo::Piece(east_rook));

        Piece::add_king_moves_to_board(60, &mut board);

        assert_moves(&board, &[51, 52, 53, 58, 59, 61]);
    }

    #[test]
    fn king_moves_castle_west_you() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(
            60,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::King)),
        );
        board.set(
            56,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::Rook)),
        );

        Piece::add_king_moves_to_board(60, &mut board);

        assert_moves(&board, &[51, 52, 53, 58, 59, 61]);
    }

    #[test]
    fn king_moves_castle_west_opponent() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(
            4,
            PosInfo::Piece(PieceInstance::new(Player::Opponent, Piece::King)),
        );
        board.set(
            0,
            PosInfo::Piece(PieceInstance::new(Player::Opponent, Piece::Rook)),
        );

        Piece::add_king_moves_to_board(4, &mut board);

        assert_moves(&board, &[2, 3, 5, 11, 12, 13]);
    }

    #[test]
    fn king_moves_castle_east_you() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(
            60,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::King)),
        );
        board.set(
            63,
            PosInfo::Piece(PieceInstance::new(Player::You, Piece::Rook)),
        );

        Piece::add_king_moves_to_board(60, &mut board);

        assert_moves(&board, &[51, 52, 53, 59, 61, 62]);
    }

    #[test]
    fn king_moves_castle_east_opponent() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(
            4,
            PosInfo::Piece(PieceInstance::new(Player::Opponent, Piece::King)),
        );
        board.set(
            7,
            PosInfo::Piece(PieceInstance::new(Player::Opponent, Piece::Rook)),
        );

        Piece::add_king_moves_to_board(4, &mut board);

        assert_moves(&board, &[3, 5, 6, 11, 12, 13]);
    }

    #[test]
    fn knight_moves() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(36, ins(Player::You, Piece::Knight));

        {
            let mut board = board.clone();
            Piece::add_knight_moves_to_board(36, &mut board);

            assert_moves(&board, &[19, 21, 26, 30, 42, 51, 53, 46]);
        }

        {
            let mut board = board.clone();
            board.set(19, ins(Player::You, Piece::Pawn));
            board.set(21, ins(Player::Opponent, Piece::Pawn));

            Piece::add_knight_moves_to_board(36, &mut board);

            assert_moves(&board, &[21, 26, 30, 42, 51, 53, 46]);
        }

        {
            // Check that moves are correctly added in the top left corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(0, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(0, &mut board);

            assert_moves(&board, &[10, 17]);
        }

        {
            // Check that moves are correctly added in the top right corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(7, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(7, &mut board);

            assert_moves(&board, &[13, 22]);
        }

        {
            // Check that moves are correctly added in the bottom left corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(56, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(56, &mut board);

            assert_moves(&board, &[41, 50]);
        }

        {
            // Check that moves are correctly added in the bottom right corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(63, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(63, &mut board);

            assert_moves(&board, &[46, 53]);
        }

        {
            // Check that moves are correctly added 1 offset the top left corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(9, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(9, &mut board);

            assert_moves(&board, &[3, 19, 24, 26]);
        }

        {
            // Check that moves are correctly added 1 offset the bottom right corner
            // (out of bounds check).

            let mut board = InfoBoard::new(Color::Black, Color::White);
            board.set(54, ins(Player::You, Piece::Knight));

            Piece::add_knight_moves_to_board(54, &mut board);

            assert_moves(&board, &[37, 39, 44, 60]);
        }
    }

    #[test]
    fn pawn_moves() {
        let board = InfoBoard::new(Color::Black, Color::White);

        {
            let mut board = board.clone();
            let pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
            board.set(48, PosInfo::Piece(pawn_ins.clone()));

            Piece::add_pawn_moves_to_board(48, &pawn_ins, &mut board);

            assert_moves(&board, &[32, 40]);
        }

        {
            let mut pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
            pawn_ins.was_moved = true;
            let mut board = board.clone();
            board.set(40, PosInfo::Piece(pawn_ins.clone()));

            Piece::add_pawn_moves_to_board(40, &pawn_ins, &mut board);

            assert_moves(&board, &[32]);
        }

        {
            let mut pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
            pawn_ins.was_moved = true;
            let mut board = board.clone();
            board.set(36, PosInfo::Piece(pawn_ins.clone()));
            board.set(27, ins(Player::Opponent, Piece::Pawn));
            board.set(29, ins(Player::Opponent, Piece::Pawn));

            Piece::add_pawn_moves_to_board(36, &pawn_ins, &mut board);

            assert_moves(&board, &[27, 28, 29]);
        }

        {
            let mut pawn_ins = PieceInstance::new(Player::You, Piece::Pawn);
            pawn_ins.was_moved = true;
            let mut board = board.clone();
            board.set(36, PosInfo::Piece(pawn_ins.clone()));
            board.set(27, ins(Player::You, Piece::Pawn));

            Piece::add_pawn_moves_to_board(36, &pawn_ins, &mut board);

            assert_moves(&board, &[28]);
        }

        // TODO: out of bounds check

        // TODO: tests for en passant
    }

    #[test]
    fn queen_moves() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(36, ins(Player::You, Piece::Queen));

        {
            let mut board = board.clone();
            Piece::add_queen_moves_to_board(36, &mut board);

            assert_moves(
                &board,
                &[
                    0, 4, 9, 12, 15, 18, 20, 22, 27, 28, 29, 32, 33, 34, 35, 37, 38, 39, 43, 44,
                    45, 50, 52, 54, 57, 60, 63,
                ],
            );
        }

        {
            let mut board = board.clone();
            board.set(35, ins(Player::You, Piece::Pawn));
            board.set(37, ins(Player::Opponent, Piece::Pawn));

            Piece::add_queen_moves_to_board(36, &mut board);

            assert_moves(
                &board,
                &[
                    0, 4, 9, 12, 15, 18, 20, 22, 27, 28, 29, 37, 43, 44, 45, 50, 52, 54, 57, 60, 63,
                ],
            );
        }
    }

    #[test]
    fn rook_moves() {
        let mut board = InfoBoard::new(Color::Black, Color::White);
        board.set(36, ins(Player::You, Piece::Rook));

        {
            let mut board = board.clone();
            Piece::add_rook_moves_to_board(36, &mut board);

            assert_moves(
                &board,
                &[4, 12, 20, 28, 32, 33, 34, 35, 37, 38, 39, 44, 52, 60],
            );
        }

        {
            let mut board = board.clone();
            board.set(35, ins(Player::You, Piece::Pawn));
            board.set(37, ins(Player::Opponent, Piece::Pawn));

            Piece::add_rook_moves_to_board(36, &mut board);

            assert_moves(&board, &[4, 12, 20, 28, 37, 44, 52, 60]);
        }
    }

    fn assert_moves(board: &InfoBoard, moves: &[usize]) {
        let mut board_moves: Vec<usize> = board.moves.iter().map(|pos_info| pos_info.1).collect();
        board_moves.sort();

        let mut moves = moves.to_owned();
        moves.sort();

        let mut expected = InfoBoard::new(board.you_color.clone(), board.opponent_color.clone());

        for move_idx in &moves {
            expected.set(*move_idx, PosInfo::Move);
        }

        assert_eq!(
            board_moves, moves,
            "expected moves {}, but found {}",
            expected, board
        );
    }

    fn ins(player: Player, piece: Piece) -> PosInfo {
        let mut ins = PieceInstance::new(player, piece);
        ins.was_moved = true;

        PosInfo::Piece(ins)
    }
}
