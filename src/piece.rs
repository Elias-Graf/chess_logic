use crate::{
    board::{self, PieceInstance},
    info_board::{self, PosInfo},
    InfoBoard, Player,
};

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
    fn add_bishop_moves_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        Self::add_moves_by_direction(x, y, Direction::NorthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthWest, board);
        Self::add_moves_by_direction(x, y, Direction::NorthWest, board);
    }

    fn add_castle_moves_to_board(king_x: i8, king_y: i8, board: &mut InfoBoard) {
        let king_instance = match board.get(king_x, king_y) {
            info_board::PosInfo::Piece(instance) => instance,
            // Castle is not allowed when the king is in check.
            info_board::PosInfo::PieceHit(_) => return,
            info => panic!(
                "cannot add castle moves as no piece was on the specified position ({}/{}), instead received '{:?}'",
                king_x, king_y, info
            ),
        };

        assert!(
            matches!(king_instance.piece, Piece::King),
            "can only castle with kings, specified piece ({}/{}) was '{:?}'",
            king_x,
            king_y,
            king_instance.piece
        );

        if king_instance.was_moved {
            return;
        }

        if Self::check_if_rook_can_be_castled(king_x, king_y, Direction::West, board) {
            board.set(king_x - 2, king_y, info_board::PosInfo::Move);
        }

        if Self::check_if_rook_can_be_castled(king_x, king_y, Direction::East, board) {
            board.set(king_x + 2, king_y, info_board::PosInfo::Move);
        }
    }

    pub fn add_en_passant_moves_to_board(
        piece_x: i8,
        piece_y: i8,
        ins: &PieceInstance,
        board: &mut InfoBoard,
    ) {
        let dir = Piece::get_pawn_direction(ins);

        let east_x = piece_x + 1;
        if board.is_in_bounds(east_x, piece_y)
            && matches!(board.get(east_x, piece_y), PosInfo::Piece(_))
        {
            board.set(east_x, piece_y + dir, PosInfo::Move);
        }

        let west_x = piece_x - 1;
        if board.is_in_bounds(west_x, piece_y)
            && matches!(board.get(west_x, piece_y), PosInfo::Piece(_))
        {
            board.set(west_x, piece_y + dir, PosInfo::Move);
        }
    }

    fn add_king_moves_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        Self::add_moves_by_direction_and_length(x, y, Direction::North, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::NorthEast, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::East, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::SouthEast, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::South, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::SouthWest, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::West, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::NorthWest, 1, board);

        Self::add_castle_moves_to_board(x, y, board);
    }

    fn add_knight_moves_to_board(piece_x: i8, piece_y: i8, board: &mut InfoBoard) {
        const REL_MOVES: [(i8, i8); 8] = [
            (1, -2),
            (2, -1),
            (2, 1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
        ];

        let board_width = board.width();
        let board_height = board.height();

        let abs_moves = REL_MOVES
            .iter()
            .map(|&(rel_x, rel_y)| (piece_x + rel_x, piece_y + rel_y))
            .filter(|&(x, y)| x >= 0 && y >= 0 && x < board_width && y < board_height);

        for (target_x, target_y) in abs_moves {
            let piece_was_hit = Self::set_for_piece_at_move_or_hit_at_in_board(
                piece_x, piece_y, target_x, target_y, board,
            );

            if piece_was_hit {
                continue;
            }
        }
    }

    fn add_moves_by_direction(x: i8, y: i8, direction: Direction, board: &mut InfoBoard) {
        Self::add_moves_by_direction_and_length(
            x,
            y,
            direction,
            i8::max(board.width(), board.height()),
            board,
        )
    }

    fn add_moves_by_direction_and_length(
        piece_x: i8,
        piece_y: i8,
        direction: Direction,
        length: i8,
        board: &mut InfoBoard,
    ) {
        for i in 1..length + 1 {
            let (target_x, target_y) = match direction {
                Direction::North => {
                    if i > piece_y {
                        break;
                    }

                    (piece_x, piece_y - i)
                }
                Direction::NorthEast => {
                    if i > piece_y {
                        break;
                    }

                    (piece_x + i, piece_y - i)
                }
                Direction::East => (piece_x + i, piece_y),
                Direction::SouthEast => (piece_x + i, piece_y + i),
                Direction::South => (piece_x, piece_y + i),
                Direction::SouthWest => {
                    if i > piece_x {
                        break;
                    }

                    (piece_x - i, piece_y + i)
                }
                Direction::West => {
                    if i > piece_x {
                        break;
                    }

                    (piece_x - i, piece_y)
                }
                Direction::NorthWest => {
                    if i > piece_x || i > piece_y {
                        break;
                    }

                    (piece_x - i, piece_y - i)
                }
            };

            if !board.is_in_bounds(target_x, target_y) {
                break;
            }

            let piece_was_hit = Self::set_for_piece_at_move_or_hit_at_in_board(
                piece_x, piece_y, target_x, target_y, board,
            );

            if piece_was_hit {
                break;
            }
        }
    }

    pub fn add_moves_for_piece_at_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        assert!(
            board.is_in_bounds(x, y),
            "cannot add moves to board for piece out of bounds ({}/{})",
            x,
            y,
        );

        let instance = match board.get(x, y) {
            info_board::PosInfo::Piece(piece) | info_board::PosInfo::PieceHit(piece) => {
                piece.clone()
            }
            info => panic!(
                "can only add moves for pieces, but piece at position {}/{} was {:?}",
                x, y, info
            ),
        };

        match &instance.piece {
            Self::Bishop => Self::add_bishop_moves_to_board(x, y, board),
            Self::King => Self::add_king_moves_to_board(x, y, board),
            Self::Knight => Self::add_knight_moves_to_board(x, y, board),
            Self::Pawn => Self::add_pawn_moves_to_board(x, y, &instance, board),
            Self::Queen => Self::add_queen_moves_to_board(x, y, board),
            Self::Rook => Self::add_rook_moves_to_board(x, y, board),
        }
    }

    fn add_pawn_moves_to_board(x: i8, y: i8, ins: &board::PieceInstance, board: &mut InfoBoard) {
        let direction = Self::get_pawn_direction(ins);

        Self::check_and_add_pawn_move_at_position_to_board(x, y + direction, board);
        // The pawn is allowed to move two positions on it's first move.
        // The pawn can't "jump" over a pice, thus the first square needs to be free (movable to).
        if !ins.was_moved && matches!(board.get(x, y + direction), info_board::PosInfo::Move) {
            Self::check_and_add_pawn_move_at_position_to_board(x, y + direction * 2, board);
        }

        Self::check_and_add_pawn_hit_at_position_to_board(&ins, x - 1, y + direction, board);
        Self::check_and_add_pawn_hit_at_position_to_board(&ins, x + 1, y + direction, board);
    }

    fn add_queen_moves_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        Self::add_moves_by_direction(x, y, Direction::NorthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthWest, board);
        Self::add_moves_by_direction(x, y, Direction::NorthWest, board);

        Self::add_moves_by_direction(x, y, Direction::North, board);
        Self::add_moves_by_direction(x, y, Direction::East, board);
        Self::add_moves_by_direction(x, y, Direction::South, board);
        Self::add_moves_by_direction(x, y, Direction::West, board);
    }

    fn add_rook_moves_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        Self::add_moves_by_direction(x, y, Direction::North, board);
        Self::add_moves_by_direction(x, y, Direction::East, board);
        Self::add_moves_by_direction(x, y, Direction::South, board);
        Self::add_moves_by_direction(x, y, Direction::West, board);
    }

    /// * `orig_ins` - The originating instance of the hit.
    fn check_and_add_pawn_hit_at_position_to_board(
        orig_ins: &PieceInstance,
        x: i8,
        y: i8,
        board: &mut InfoBoard,
    ) {
        if !board.is_in_bounds(x, y) {
            return;
        }

        if let info_board::PosInfo::Piece(target_ins) = board.get(x, y) {
            if orig_ins.player == target_ins.player {
                return;
            }

            let instance = target_ins.clone();

            board.set(x, y, info_board::PosInfo::PieceHit(instance));
        }
    }

    fn check_and_add_pawn_move_at_position_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        if !board.is_in_bounds(x, y) {
            return;
        }

        if matches!(board.get(x, y), info_board::PosInfo::None) {
            board.set(x, y, info_board::PosInfo::Move);
        }
    }

    fn check_if_rook_can_be_castled(
        king_x: i8,
        king_y: i8,
        direction: Direction,
        board: &mut InfoBoard,
    ) -> bool {
        let (range_to_check, rook_x) = match direction {
            Direction::East => (king_x + 1..board.width() - 1, board.width() - 1),
            Direction::West => (1..king_x, 0),
            _ => panic!(
                "direction '{:?}' not valid when checking if rook can be castled",
                direction
            ),
        };

        let rook_instance = match board.get(rook_x, king_y) {
            info_board::PosInfo::Piece(instance) => instance,
            _ => return false,
        };

        if rook_instance.was_moved {
            return false;
        }

        for i in range_to_check {
            if let info_board::PosInfo::Piece(_) = board.get(i, king_y) {
                return false;
            }
        }

        true
    }

    // TODO: This is not at the right place. Check if this can be moved somewhere
    // else.
    // As stated in the comment bellow, this will break as soon as `you` is not at
    // the bottom of the board. Thus this should probably be inside the [`Board`].
    pub fn get_pawn_direction(ins: &board::PieceInstance) -> i8 {
        // Currently it is assumed that you are at the bottom of the board.
        // In case this assumption is false in the future, this code WILL not work.
        match ins.player {
            Player::You => -1,
            Player::Opponent => 1,
        }
    }

    pub fn get_symbol(piece: &Self) -> String {
        match piece {
            Self::Bishop => "BI",
            Self::King => "KI",
            Self::Knight => "KN",
            Self::Pawn => "PA",
            Self::Queen => "QU",
            Self::Rook => "RO",
        }
        .to_owned()
    }

    /// For a piece at a given position, set a move or a hit at a given position.
    ///
    /// It will determined if a hit or a move is registered, depending on if the
    /// target position is empty or not.
    ///
    /// Returns `true` when a piece was hit. In that case, the piece should "stop"
    /// adding further moves in that direction.
    fn set_for_piece_at_move_or_hit_at_in_board(
        orig_x: i8,
        orig_y: i8,
        target_x: i8,
        target_y: i8,
        board: &mut InfoBoard,
    ) -> bool {
        assert!(
            board.is_in_bounds(orig_x, orig_y),
            "cannot set move for piece out of bounds ({}/{})",
            orig_x,
            orig_y
        );
        assert!(
            board.is_in_bounds(target_x, target_y),
            "cannot set move or hit outside bounds ({}/{})",
            target_x,
            target_y
        );

        let target_pos = board.get(target_x, target_y);

        let target_ins = match target_pos {
            info_board::PosInfo::Move => return false,
            info_board::PosInfo::None => {
                board.set(target_x, target_y, info_board::PosInfo::Move);
                return false;
            }
            info_board::PosInfo::Piece(i) => i,
            info_board::PosInfo::PieceHit(_) => return false,
        };

        let origin_ins = match board.get(orig_x, orig_y) {
            info_board::PosInfo::Piece(i) | info_board::PosInfo::PieceHit(i) => i,
            info => panic!(
                "there is no origin piece at position: {}/{}, info is: '{:?}'",
                orig_x, orig_y, info
            ),
        };

        if target_ins.player != origin_ins.player {
            let info = info_board::PosInfo::PieceHit(target_ins.clone());
            board.set(target_x, target_y, info);
        }

        return true;
    }
}

#[derive(Debug)]
enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}
