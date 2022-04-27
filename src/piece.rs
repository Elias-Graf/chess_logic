use crate::{board, info_board, InfoBoard, Player};

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

    fn add_king_moves_to_board(x: i8, y: i8, board: &mut InfoBoard) {
        Self::add_moves_by_direction_and_length(x, y, Direction::North, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::NorthEast, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::East, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::SouthEast, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::South, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::SouthWest, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::West, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::NorthWest, 1, board);
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
            let piece_was_hit = Self::set_for_piece_at_move_or_hit_at_to_board(
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

            let piece_was_hit = Self::set_for_piece_at_move_or_hit_at_to_board(
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
            info_board::PosInfo::Piece(piece) => piece.clone(),
            info_board::PosInfo::None => return,
            info => panic!(
                "can only add moves for pieces, but piece at position {}/{} was {:?}",
                x, y, info
            ),
        };

        match &instance.piece {
            Self::Bishop => Self::add_bishop_moves_to_board(x, y, board),
            Self::King => Self::add_king_moves_to_board(x, y, board),
            Self::Knight => Self::add_knight_moves_to_board(x, y, board),
            Self::Pawn => Self::add_pawn_moves_to_board(x, y, instance, board),
            Self::Queen => Self::add_queen_moves_to_board(x, y, board),
            Self::Rook => Self::add_rook_moves_to_board(x, y, board),
        }
    }

    fn add_pawn_moves_to_board(
        x: i8,
        y: i8,
        instance: board::PieceInstance,
        board: &mut InfoBoard,
    ) {
        let direction = if instance.player == Player::You {
            -1
        } else {
            1
        };

        // TODO: the following statements are incorrect as the pawn can move forwards,
        // but only hit diagonally.
        Self::set_for_piece_at_move_or_hit_at_to_board(x, y, x, y + direction, board);

        // The pawn is allowed to move two positions on it's first move.
        if !instance.was_moved {
            Self::set_for_piece_at_move_or_hit_at_to_board(x, y, x, y + direction * 2, board);
        }
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
    fn set_for_piece_at_move_or_hit_at_to_board(
        piece_x: i8,
        piece_y: i8,
        target_x: i8,
        target_y: i8,
        board: &mut InfoBoard,
    ) -> bool {
        assert!(
            board.is_in_bounds(piece_x, piece_y),
            "cannot set move for piece out of bounds ({}/{})",
            piece_x,
            piece_y
        );
        assert!(
            board.is_in_bounds(target_x, target_y),
            "cannot set move or hit outside bounds ({}/{})",
            target_x,
            target_y
        );

        let piece_pos = board.get(piece_x, piece_y);
        let target_pos = board.get(target_x, target_y);

        // TODO: refactor.
        match target_pos {
            info_board::PosInfo::None => board.set(target_x, target_y , info_board::PosInfo::Move),
            info_board::PosInfo::Piece(target_piece_instance) => {


                if let info_board::PosInfo::Piece(piece_instance) = piece_pos {
                    if target_piece_instance.player == piece_instance.player {
                        // Cannot really "hit" an own piece, thus no hit is registered.
                        return true;
                    }
                }

                let info = info_board::PosInfo::PieceHit(target_piece_instance.clone());
                board.set(target_x, target_y, info);

                return true;
            },
            _ => panic!("moves or hits can only be set on positions that are empty or pieces are on, position was '{:?}'", target_pos),
        }

        false
    }
}

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
