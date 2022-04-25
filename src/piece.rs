use crate::{Board, Player};

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

    pub fn add_moves_to_board(x: i8, y: i8, player: &Player, piece: &Self, board: &mut Board) {
        assert!(
            board.is_in_bounds(x, y),
            "can not add moves for piece out of bounds ({}/{})",
            x,
            y
        );

        match piece {
            Self::Bishop => Self::add_bishop_moves_to_board(x, y, board),
            Self::King => Self::add_king_moves_to_board(x, y, board),
            Self::Knight => Self::add_knight_moves_to_board(x, y, board),
            Self::Pawn => Self::add_pawn_moves_to_board(x, y, player, board),
            Self::Queen => Self::add_queen_moves_to_board(x, y, board),
            Self::Rook => Self::add_rook_moves_to_board(x, y, board),
        }
    }

    fn add_bishop_moves_to_board(x: i8, y: i8, board: &mut Board) {
        Self::add_moves_by_direction(x, y, Direction::NorthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthWest, board);
        Self::add_moves_by_direction(x, y, Direction::NorthWest, board);
    }

    fn add_king_moves_to_board(x: i8, y: i8, board: &mut Board) {
        Self::add_moves_by_direction_and_length(x, y, Direction::North, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::East, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::South, 1, board);
        Self::add_moves_by_direction_and_length(x, y, Direction::West, 1, board);
    }

    fn add_knight_moves_to_board(piece_x: i8, piece_y: i8, board: &mut Board) {
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

        for (hit_x, hit_y) in abs_moves {
            let piece_was_hit =
                board.set_for_piece_at_move_or_hit_at(piece_x, piece_y, hit_x, hit_y);

            if piece_was_hit {
                continue;
            }
        }
    }

    fn add_pawn_moves_to_board(x: i8, y: i8, player: &Player, board: &mut Board) {
        let move_y = match player {
            Player::You => (y as i8) - 1,
            Player::Opponent => (y as i8) + 1,
        };

        board.set_for_piece_at_move_or_hit_at(x, y, x, move_y);
    }

    fn add_queen_moves_to_board(x: i8, y: i8, board: &mut Board) {
        Self::add_moves_by_direction(x, y, Direction::NorthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthEast, board);
        Self::add_moves_by_direction(x, y, Direction::SouthWest, board);
        Self::add_moves_by_direction(x, y, Direction::NorthWest, board);

        Self::add_moves_by_direction(x, y, Direction::North, board);
        Self::add_moves_by_direction(x, y, Direction::East, board);
        Self::add_moves_by_direction(x, y, Direction::South, board);
        Self::add_moves_by_direction(x, y, Direction::West, board);
    }

    fn add_rook_moves_to_board(x: i8, y: i8, board: &mut Board) {
        Self::add_moves_by_direction(x, y, Direction::North, board);
        Self::add_moves_by_direction(x, y, Direction::East, board);
        Self::add_moves_by_direction(x, y, Direction::South, board);
        Self::add_moves_by_direction(x, y, Direction::West, board);
    }

    fn add_moves_by_direction(x: i8, y: i8, direction: Direction, board: &mut Board) {
        Self::add_moves_by_direction_and_length(
            x,
            y,
            direction,
            i8::max(board.width(), board.height()),
            board,
        )
    }

    fn add_moves_by_direction_and_length(
        x: i8,
        y: i8,
        direction: Direction,
        length: i8,
        board: &mut Board,
    ) {
        for i in 1..length + 1 {
            let (mov_x, mov_y) = match direction {
                Direction::North => {
                    if i > y {
                        break;
                    }

                    (x, y - i)
                }
                Direction::NorthEast => {
                    if i > y {
                        break;
                    }

                    (x + i, y - i)
                }
                Direction::East => (x + i, y),
                Direction::SouthEast => (x + i, y + i),
                Direction::South => (x, y + i),
                Direction::SouthWest => {
                    if i > x {
                        break;
                    }

                    (x - i, y + i)
                }
                Direction::West => {
                    if i > x {
                        break;
                    }

                    (x - i, y)
                }
                Direction::NorthWest => {
                    if i > x || i > y {
                        break;
                    }

                    (x - i, y - i)
                }
            };

            if !board.is_in_bounds(mov_x, mov_y) {
                break;
            }

            let hit_piece = board.set_for_piece_at_move_or_hit_at(x, y, mov_x, mov_y);

            if hit_piece {
                break;
            }
        }
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
