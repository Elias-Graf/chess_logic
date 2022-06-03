//! Contains the move generation for the more complicated sliding pieces.
//!
//! Sliding pieces have to "stop dead in their tracks" when hitting another piece,
//! discarding all moves that could have followed. Therefore, **lookup tables**
//! are generated for each square, for each variant of pieces (also called occupancies)
//! that could block the progressing of sliding moves.
//!
//! **These tables are index by so called "magic numbers".** There is nothing special
//! or magic about them. They are just a random number that were found to lead to
//! the correct set of moves when processed by certain mathematical operation. So
//! if you read anything related to "magic number" or "magic index" in this module,
//! simply think "randomly generated number" that can be used to generate an index
//! in the move lookup table.

use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, CustomDefault, U64PerSquare, _U64PerSquare},
    piece,
    square::{BoardPos, Square},
    Board, Piece,
};

/// Returns the bishop moves for a given position, with given blockers.
///
/// Abstracts away all the table lookups maths. Read the module-level documentation
/// for more information.
pub fn get_bishop_attacks_for(pos: &dyn BoardPos, blockers: u64) -> u64 {
    let magic_index = magic_index_of(
        BISHOP_MAGIC_NUMBERS[pos],
        blockers,
        NUMBER_OF_RELEVANT_BISHOP_MOVES[pos],
    );

    ALL_BISHOP_ATTACKS[magic_index][pos]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bishop_compare_to_slow_to_generate_source_of_truth() {
        for i in 0..Board::SIZE {
            let truth = piece::calculate_bishop_attacks_for(&i, 0);
            let lookup_result = get_bishop_attacks_for(&i, 0);

            assert_eq!(truth, lookup_result);
        }
    }
}

static BISHOP_MAGIC_NUMBERS: Lazy<_U64PerSquare> = Lazy::new(|| {
    // TODO: get rid of inline body.
    let mut numbers = _U64PerSquare::new();

    for i in 0..Board::SIZE {
        numbers[&i] = find_magic_number(
            &i,
            NUMBER_OF_RELEVANT_BISHOP_MOVES[i as usize],
            Piece::Bishop,
        )
    }

    numbers
});

static ALL_RELEVANT_BISHOP_MOVES: Lazy<U64PerSquare> = Lazy::new(generate_relevant_bishop_moves);

#[rustfmt::skip]
const NUMBER_OF_RELEVANT_BISHOP_MOVES: U64PerSquare = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

static ALL_BISHOP_ATTACKS: Lazy<Box<[U64PerSquare; 512]>> =
    Lazy::new(|| Box::new(generate_all_possible_bishop_attacks()));

fn generate_relevant_bishop_moves() -> U64PerSquare {
    let mut moves = U64PerSquare::default();

    for i in 0..bit_board::SIZE {
        let pos: &dyn BoardPos = &i;

        let board = bit_board::with_bit_at(i);

        let file = i % bit_board::HEIGHT;
        let rank = i / bit_board::HEIGHT;

        let to_no_ea = u64::min((bit_board::WIDTH - 1) - file, rank);
        let to_so_ea = u64::min(bit_board::WIDTH - file, bit_board::HEIGHT - rank) - 1;
        let to_so_we = u64::min(file, (bit_board::HEIGHT - 1) - rank);
        let to_no_we = u64::min(file, rank);

        for iter in 1..to_no_ea {
            moves[pos] |= board >> bit_board::NO_EA * iter;
        }
        for iter in 1..to_so_ea {
            moves[pos] |= board << bit_board::SO_EA * iter;
        }
        for iter in 1..to_so_we {
            moves[pos] |= board << bit_board::SO_WE * iter;
        }
        for iter in 1..to_no_we {
            moves[pos] |= board >> bit_board::NO_WE * iter;
        }
    }

    moves
}

fn generate_all_possible_bishop_attacks() -> [U64PerSquare; 512] {
    let mut all_attacks = [[0; 64]; 512];

    for i in 0..Board::SIZE {
        let relevant_moves = ALL_RELEVANT_BISHOP_MOVES[i];
        let number_of_relevant_moves = NUMBER_OF_RELEVANT_BISHOP_MOVES[i];

        for occupancy_idx in 0..number_of_occupancy_variants(number_of_relevant_moves) {
            let occupancy_variant =
                bb::move_occupancy_variant(occupancy_idx, number_of_relevant_moves, relevant_moves);
            let magic_index = magic_index_of(
                BISHOP_MAGIC_NUMBERS[&i],
                occupancy_variant,
                number_of_relevant_moves,
            );

            all_attacks[magic_index][i] =
                piece::calculate_bishop_attacks_for(&i, occupancy_variant);
        }
    }

    all_attacks
}

/// Calculates how many variants of occupants there are.
///
/// Takes in all the squares a move could reach, and calculates the amount of variants
/// of pieces on those squares.
///
/// TODO: add example.
fn number_of_occupancy_variants(number_of_relevant_moves: u64) -> u64 {
    // TODO: Reduce this number.
    // Currently this results in a vast overestimate of this number (I think).
    // Maybe there is a more sensible way to do this. Hardcode a value maybe?
    1 << number_of_relevant_moves
}

/// Calculates the number that can be used to index the attacks table.
///
/// # Arguments
///
/// * `magic_number` - "magic" (**generated** random) number, used to differentiate similar
///    `occupancies`, see [`find_magic_number`] to checkout the generation.
/// * `occupancies` - what squares are occupied
/// * `number_of_occupied_sports` - used to further differentiate indexes
fn magic_index_of(magic_number: u64, occupancies: u64, number_of_relevant_bits: u64) -> usize {
    // magic_number
    //     .wrapping_mul(magic_number)
    //     .wrapping_mul(occupancies) as usize
    //     >> 65 - number_of_relevant_bits
    occupancies.wrapping_mul(magic_number) as usize >> 64 - number_of_relevant_bits
}

fn find_magic_number(pos: &dyn BoardPos, relevant_bits: u64, piece: Piece) -> u64 {
    let mut occupancies = [0u64; 4096];
    let mut attacks = [0u64; 4096];
    let mut used_attacks;

    let (attack_mask, get_attacks_for): (u64, fn(&dyn BoardPos, u64) -> u64) = match piece {
        Piece::Bishop => (
            ALL_RELEVANT_BISHOP_MOVES[pos],
            piece::calculate_bishop_attacks_for,
        ),
        Piece::Rook => (
            piece::ROOK_RELEVANT_MOVE_MASK[pos],
            Piece::get_rook_attacks_for,
        ),
        _ => panic!(
            "only bishops and rooks are valid arguments, found '{:?}'",
            piece
        ),
    };

    let occupancy_indices = (1 << relevant_bits) as usize;

    for i in 0..occupancy_indices {
        occupancies[i] = bit_board::bb::set_occupancy(i as u64, relevant_bits, attack_mask);
        attacks[i] = get_attacks_for(pos, occupancies[i]);
    }

    for x in 0..10000000000000u64 {
        let magic_number = generate_magic_number();

        if bit_board::count_set_bits(
            (attack_mask.overflowing_mul(magic_number).0) & 0xFF00000000000000,
        ) < 6
        {
            continue;
        }

        used_attacks = [0; 4096];

        let mut index = 0;
        let mut failed = false;

        while !failed && index < occupancy_indices {
            if failed {
                dbg!(failed);
            }
            // let magic_index =
            //     (occupancies[index].overflowing_mul(magic_number).0 >> 64 - relevant_bits) as usize;
            // let magic_index =
            //     occupancies[index].wrapping_mul(magic_number) as usize >> 64 - relevant_bits;
            let magic_index = magic_index_of(magic_number, occupancies[index], relevant_bits);

            if pos.idx() == Square::F6.idx() && magic_index == 0 {
                // dbg!(
                //     occupancies[index].wrapping_mul(magic_number),
                //     64 - relevant_bits,
                //     occupancies[index].wrapping_mul(magic_number) >> 64 - relevant_bits,
                //     used_attacks[magic_index] != attacks[index],
                // );
                // println!(
                //     "--------------------\nmagic index: {}\nmagic number: {}\noccupancies:\n{}",
                //     magic_index,
                //     magic_number,
                //     bit_board::display(occupancies[index]),
                // );
                // println!("attacks\n{}", bit_board::display(attacks[index]));
            }

            if used_attacks[magic_index] == 0 {
                used_attacks[magic_index] = attacks[index];
            } else if used_attacks[magic_index] != attacks[index] {
                // I think this condition means that two magic indexes resulted
                // in different attacks which should not happen.
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
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Generates all the possible variants of move occupancy bases on an index.
    ///
    /// See [`number_of_occupancy_variants`] for additional information.
    pub fn move_occupancy_variant(
        occupancy_idx: u64,
        number_of_relevant_moves: u64,
        mut relevant_moves: u64,
    ) -> u64 {
        let mut variant = 0;

        for iter in 0..number_of_relevant_moves {
            if let Some(square) = bit_board::get_first_set_bit(relevant_moves) {
                bit_board::clear_bit(&mut relevant_moves, square);

                if occupancy_idx & (1 << iter) > 0 {
                    // set_bit(&mut occupancy, square);
                    variant |= 1 << square;
                }
            }
        }

        variant
    }

    /// Generates a pseudo random number.
    ///
    /// Code from:
    /// https://youtu.be/JjFYmkUhLN4?list=PLmN0neTso3Jxh8ZIylk74JpwfiWNI76Cs&t=476
    pub fn random_u32() -> u32 {
        static STATE: AtomicU32 = AtomicU32::new(1082485);

        let mut local_state = STATE.load(Ordering::Relaxed);

        local_state ^= local_state << 13;
        local_state ^= local_state >> 17;
        local_state ^= local_state << 5;

        STATE.store(local_state, Ordering::Relaxed);

        local_state
    }
}
