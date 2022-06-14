use std::{cmp::min, ops::Range};

use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, ColoredU64PerSquare, U64PerSquare},
    board::BoardPos,
    magic_bit_board,
    type_alias_default::TypeAliasDefault,
    Board, Color,
};

const NOT_FILE_A: u64 = 18374403900871474942;
const NOT_FILE_AB: u64 = 18229723555195321596;
const NOT_FILE_GH: u64 = 4557430888798830399;
const NOT_FILE_H: u64 = 9187201950435737471;

static KING_ATTACK_MASK: Lazy<U64PerSquare> = Lazy::new(generate_king_attacks);
static KNIGHT_ATTACK_MASK: Lazy<U64PerSquare> = Lazy::new(generate_knight_attacks);
static PAWN_ATTACK_MASK: Lazy<ColoredU64PerSquare> = Lazy::new(generate_pawn_attacks);

pub fn get_bishop_attacks_for(pos: impl BoardPos, blockers: u64) -> u64 {
    magic_bit_board::get_bishop_attacks_for(pos.into(), blockers)
}

pub fn get_king_attack_mask_for(pos: impl BoardPos) -> u64 {
    KING_ATTACK_MASK[pos.into()]
}

pub fn get_knight_attack_mask_for(pos: impl BoardPos) -> u64 {
    KNIGHT_ATTACK_MASK[pos.into()]
}

pub fn get_pawn_attacks_for(pos: impl BoardPos, color: &Color) -> u64 {
    PAWN_ATTACK_MASK[*color][pos.into()]
}

pub fn get_queen_attacks_for(pos: impl BoardPos, blockers: u64) -> u64 {
    let i = pos.into();

    magic_bit_board::get_bishop_attacks_for(i, blockers)
        | magic_bit_board::get_rook_attacks_for(i, blockers)
}

pub fn get_rook_attacks_for(pos: impl BoardPos, blockers: u64) -> u64 {
    magic_bit_board::get_rook_attacks_for(pos.into(), blockers)
}

pub fn calculate_bishop_attacks_for(pos: impl Into<usize>, blockers: u64) -> u64 {
    use bit_board::{NO_EA, NO_WE, SO_EA, SO_WE};

    calculate_sliding_attacks_for(pos, blockers, |file, rank| {
        [
            // moves to north east
            (1..min(Board::WIDTH - file, rank + 1), |b, i| b >> NO_EA * i),
            // moves to south east
            (1..min(Board::WIDTH - file, Board::HEIGHT - rank), |b, i| {
                b << SO_EA * i
            }),
            // moves to south west
            (1..min(file + 1, Board::HEIGHT - rank), |b, i| {
                b << SO_WE * i
            }),
            // moves to north west
            (1..(min(file, rank) + 1), |b, i| b >> NO_WE * i),
        ]
    })
}

pub fn calculate_rook_attacks_for(pos: impl Into<usize>, blockers: u64) -> u64 {
    use bit_board::{EAST, NORTH, SOUTH, WEST};

    calculate_sliding_attacks_for(pos, blockers, |file, rank| {
        [
            // moves to north
            (1..rank + 1, |b, i| b >> NORTH * i),
            // moves to east
            (1..Board::WIDTH - file, |b, i| b << EAST * i),
            // moves to south
            (1..Board::HEIGHT - rank, |b, i| b << SOUTH * i),
            // moves to west
            (1..file + 1, |b, i| b >> WEST * i),
        ]
    })
}

/// Calculates the moves of sliding pieces.
///
/// # Arguments
///
/// * `pos` - the position of the pice
/// * `blockers` - other pieces on the board (they could block moves)
/// * `get_dirs` - closure that returns range to loop over, and the offset for
///    the desired direction
fn calculate_sliding_attacks_for(
    pos: impl Into<usize>,
    blockers: u64,
    get_dirs: fn(usize, usize) -> [(Range<usize>, fn(u64, usize) -> u64); 4],
) -> u64 {
    let i = pos.into();

    // TODO: this file, rank pattern is used often and could be extracted
    // to a assoc function on the board.
    let file = i % Board::HEIGHT;
    let rank = i / Board::HEIGHT;

    let board = bit_board::with_bit_at(i);

    let mut attacks = 0;
    for (range, dir_shift) in get_dirs(file, rank) {
        for iter in range {
            let shift = dir_shift(board, iter);
            attacks |= shift;

            let hit_a_blocker = bit_board::has_set_bits(shift & blockers);
            if hit_a_blocker {
                break;
            }
        }
    }

    attacks
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{testing_utils::assert_bit_boards_eq, Square::*};

    #[test]
    fn bishop_attacks_north_west_corner_without_blockers() {
        assert_bit_boards_eq(get_bishop_attacks_for(B7, 0), 9241421688590368773);
    }

    #[test]
    fn bishop_attacks_south_east_corner_with_blockers() {
        let mut blockers = 0;
        bit_board::set_bit(&mut blockers, E4.into());

        assert_bit_boards_eq(get_bishop_attacks_for(G2, blockers), 11529391036648390656);
    }

    #[test]
    fn queen_attacks_north_west_corner_without_blockers() {
        assert_bit_boards_eq(get_queen_attacks_for(B7, 0), 9386102034266586375);
    }

    #[test]
    fn queen_attacks_south_east_corner_with_blockers() {
        let mut blockers = 0;
        bit_board::set_bit(&mut blockers, F3.into());
        bit_board::set_bit(&mut blockers, G7.into());

        assert_bit_boards_eq(get_queen_attacks_for(G2, blockers), 16194909351608074240);
    }

    #[test]
    fn rook_attacks_north_west_corner_without_blockers() {
        assert_bit_boards_eq(get_rook_attacks_for(B7, 0), 144680345676217602);
    }

    #[test]
    fn rook_attacks_south_east_corner_with_blockers() {
        let mut blockers = 0;
        bit_board::set_bit(&mut blockers, G4.into());

        assert_bit_boards_eq(get_rook_attacks_for(G2, blockers), 4665518382601207808);
    }

    #[test]
    fn bishop_attacks_no_ea_corner() {
        assert_bit_boards_eq(calculate_bishop_attacks_for(G6, 0), 145249955479592976);
    }

    #[test]
    fn bishop_attacks_so_ea_corner() {
        assert_bit_boards_eq(calculate_bishop_attacks_for(G2, 0), 11529391036782871041);
    }

    #[test]
    fn bishop_attacks_so_we_corner() {
        assert_bit_boards_eq(calculate_bishop_attacks_for(B2, 0), 360293502378066048);
    }

    #[test]
    fn bishop_attacks_no_we_corner() {
        assert_bit_boards_eq(calculate_bishop_attacks_for(B7, 0), 9241421688590368773);
    }

    #[test]
    fn bishop_attacks_blocker_no_ea() {
        let blockers = bit_board::with_bit_at(G6.into());

        assert_bit_boards_eq(
            calculate_bishop_attacks_for(E4, blockers),
            9386671504487612929,
        );
    }

    #[test]
    fn rook_attacks_no_ea_corner() {
        assert_bit_boards_eq(calculate_rook_attacks_for(G6, 0), 4629771061645230144);
    }

    #[test]
    fn rook_attacks_so_ea_corner() {
        assert_bit_boards_eq(calculate_rook_attacks_for(G2, 0), 4665518383679160384);
    }

    #[test]
    fn rook_attacks_so_we_corner() {
        assert_bit_boards_eq(calculate_rook_attacks_for(B2, 0), 215330564830528002);
    }

    #[test]
    fn rook_attacks_no_we_corner() {
        assert_bit_boards_eq(calculate_rook_attacks_for(B7, 0), 144680345676217602);
    }

    #[test]
    fn rook_attacks_blocker_north() {
        let blockers = bit_board::with_bit_at(E6.into());

        assert_bit_boards_eq(
            calculate_rook_attacks_for(E4, blockers),
            1157443723186929664,
        );
    }
}

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
    /// Returns the symbol in unicode.
    ///
    /// https://en.wikipedia.org/wiki/Chess_symbols_in_Unicode
    pub fn symbol(&self, color: Color) -> &str {
        match (color, self) {
            (Color::Black, Piece::Bishop) => "♗",
            (Color::Black, Piece::King) => "♔",
            (Color::Black, Piece::Knight) => "♘",
            (Color::Black, Piece::Pawn) => "♙",
            (Color::Black, Piece::Queen) => "♕",
            (Color::Black, Piece::Rook) => "♖",
            (Color::White, Piece::Bishop) => "♝",
            (Color::White, Piece::King) => "♚",
            (Color::White, Piece::Knight) => "♞",
            (Color::White, Piece::Pawn) => "♟",
            (Color::White, Piece::Queen) => "♛",
            (Color::White, Piece::Rook) => "♜",
        }
    }
}

fn generate_king_attacks() -> U64PerSquare {
    let mut mask = U64PerSquare::default();

    for i in 0..Board::SIZE {
        let board = bit_board::with_bit_at(i);

        mask[i] |= board >> bit_board::NORTH;
        if bit_board::is_bit_set(board & NOT_FILE_H, i) {
            mask[i] |= board >> bit_board::NO_EA;
            mask[i] |= board << bit_board::EAST;
            mask[i] |= board << bit_board::SO_EA;
        }
        mask[i] |= board << bit_board::SOUTH;
        if bit_board::is_bit_set(board & NOT_FILE_A, i) {
            mask[i] |= board << bit_board::SO_WE;
            mask[i] |= board >> bit_board::WEST;
            mask[i] |= board >> bit_board::NO_WE;
        }
    }

    mask
}

fn generate_knight_attacks() -> U64PerSquare {
    let mut mask = U64PerSquare::default();

    for i in 0..Board::SIZE {
        let board = bit_board::with_bit_at(i);

        if bit_board::is_bit_set(board & NOT_FILE_A, i) {
            mask[i] |= board >> bit_board::NORTH >> bit_board::NO_WE;
        }
        if bit_board::is_bit_set(board & NOT_FILE_H, i) {
            mask[i] |= board >> bit_board::NORTH >> bit_board::NO_EA;
        }
        if bit_board::is_bit_set(board & NOT_FILE_GH, i) {
            mask[i] |= board << bit_board::EAST >> bit_board::NO_EA;
            mask[i] |= board << bit_board::EAST << bit_board::SO_EA;
        }
        if bit_board::is_bit_set(board & NOT_FILE_A, i) {
            mask[i] |= board << bit_board::SOUTH << bit_board::SO_WE;
        }
        if bit_board::is_bit_set(board & NOT_FILE_H, i) {
            mask[i] |= board << bit_board::SOUTH << bit_board::SO_EA;
        }
        if bit_board::is_bit_set(board & NOT_FILE_AB, i) {
            mask[i] |= board >> bit_board::WEST << bit_board::SO_WE;
            mask[i] |= board >> bit_board::WEST >> bit_board::NO_WE;
        }
    }

    mask
}

fn generate_pawn_attacks() -> ColoredU64PerSquare {
    let mut mask = ColoredU64PerSquare::default();

    for i in 0..Board::SIZE {
        let board = bit_board::with_bit_at(i);

        if bit_board::is_bit_set(board & NOT_FILE_A, i) {
            mask[Color::White][i] |= board >> bit_board::NO_WE;
        }
        if bit_board::is_bit_set(board & NOT_FILE_H, i) {
            mask[Color::White][i] |= board >> bit_board::NO_EA;
        }

        if bit_board::is_bit_set(board & NOT_FILE_A, i) {
            mask[Color::Black][i] |= board << bit_board::SO_WE;
        }
        if bit_board::is_bit_set(board & NOT_FILE_H, i) {
            mask[Color::Black][i] |= board << bit_board::SO_EA;
        }
    }

    mask
}
