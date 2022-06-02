use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, MoveMask},
    square::BoardPos,
    Board, Piece,
};

const SIZE: u64 = Board::SIZE as u64;
const HEIGHT: u64 = Board::HEIGHT as u64;
const WIDTH: u64 = Board::WIDTH as u64;

const BISHOP_RELEVANT_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_bishop_relevant_move_mask);
const ROOK_RELEVANT_MOVE_MASK: Lazy<MoveMask> = Lazy::new(generate_rook_relevant_move_mask);

/// Generates a move mask with only the relevant squares.
///
/// Essentially this is excluding the very edge of the board, since it would not
/// matter if it's a blocker or not.
fn generate_bishop_relevant_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..SIZE {
        let mut board = bit_board::with_bit_at(i);

        let file = i % HEIGHT;
        let rank = i / HEIGHT;

        let to_no_ea = u64::min((WIDTH - 1) - file, rank);
        let to_so_ea = u64::min(WIDTH - file, HEIGHT - rank) - 1;
        let to_so_we = u64::min(file, (HEIGHT - 1) - rank);
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

/// Same as [generate_bishop_relevant_move_mask], but for rooks.
fn generate_rook_relevant_move_mask() -> MoveMask {
    let mut mask = MoveMask::new();

    for i in 0..SIZE {
        let mut board = bit_board::with_bit_at(i);

        let file = i % HEIGHT;
        let rank = i / HEIGHT;

        let to_north = rank;
        let to_east = (WIDTH - file) - 1;
        let to_south = (HEIGHT - rank) - 1;
        let to_west = file;

        for iter in 1..to_north {
            mask[&i] |= board >> bit_board::NORTH * iter;
        }
        for iter in 1..to_east {
            mask[&i] |= board << bit_board::EAST * iter;
        }
        for iter in 1..to_south {
            mask[&i] |= board << bit_board::SOUTH * iter;
        }
        for iter in 1..to_west {
            mask[&i] |= board >> bit_board::WEST * iter;
        }
    }

    mask
}

fn find_magic_number(pos: &dyn BoardPos, relevant_bits: u64, piece: Piece) -> u64 {
    let mut occupancies = [0u64; 4096];
    let mut attacks = [0u64; 4096];
    let mut used_attacks;

    let (attack_mask, get_attacks_for): (u64, fn(&dyn BoardPos, u64) -> u64) = match piece {
        Piece::Bishop => (
            BISHOP_RELEVANT_MOVE_MASK[pos],
            Piece::get_bishop_attacks_for,
        ),
        Piece::Rook => (ROOK_RELEVANT_MOVE_MASK[pos], Piece::get_rook_attacks_for),
        _ => panic!(
            "only bishops and rooks are valid arguments, found '{:?}'",
            piece
        ),
    };

    let occupancy_indices = 1 << bit_board::count_set_bits(relevant_bits);

    for i in 0..occupancy_indices {
        occupancies[i] =
            bit_board::bb::set_occupancy(i as u64, occupancy_indices as u64, attack_mask);

        attacks[i] = get_attacks_for(pos, occupancies[i]);
    }

    for _ in 0..10000000000000u64 {
        let magic_number = generate_magic_number();

        if bit_board::count_set_bits((attack_mask.wrapping_mul(magic_number)) & 0xFF00000000000000)
            < 6
        {
            continue;
        }

        used_attacks = [0; 4096];

        let mut index = 0;
        let mut failed = false;

        while !failed && index < occupancy_indices {
            let magic_index =
                (occupancies[index].wrapping_mul(magic_number) >> 64 - relevant_bits) as usize;

            if used_attacks[magic_index] == 0 {
                used_attacks[magic_index] = attacks[index];
            } else if used_attacks[magic_index] != attacks[index] {
                failed = true;
            }

            index += 1;
        }

        if !failed {
            return magic_number;
        }
    }

    println!("magic number failed!");

    0
}

/// Generate a number that has a low amount of bits set to one.
fn generate_magic_number() -> u64 {
    random_u64() & random_u64() & random_u64()
}

fn random_u64() -> u64 {
    let n1 = bb::random_u32() as u64 & 0xFFFF;
    let n2 = (bb::random_u32() as u64 & 0xFFFF) << 16;
    let n3 = (bb::random_u32() as u64 & 0xFFFF) << 32;
    let n4 = (bb::random_u32() as u64 & 0xFFFF) << 48;

    n1 | n2 | n3 | n4
}

/// Code that I'm not sure what it does, or why it's used.
///
/// 'bb' = Black Box.
///
/// The goal is to have no code here in the future.
mod bb {
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Generates a pseudo random number.
    ///
    /// Code from:
    /// https://youtu.be/JjFYmkUhLN4?list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&t=476
    pub fn random_u32() -> u32 {
        static STATE: AtomicU32 = AtomicU32::new(108248);

        let mut local_state = STATE.load(Ordering::Relaxed);

        local_state ^= local_state << 13;
        local_state ^= local_state >> 17;
        local_state ^= local_state << 5;

        STATE.store(local_state, Ordering::Relaxed);

        local_state
    }
}
