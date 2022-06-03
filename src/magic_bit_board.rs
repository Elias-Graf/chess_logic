use std::{ops::Mul, ptr::read_volatile};

use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, U64PerSquare},
    piece,
    square::{BoardPos, Square},
    Board, Piece,
};

pub static BISHOP_MAGIC_NUMBERS: Lazy<U64PerSquare> = Lazy::new(|| {
    let mut numbers = U64PerSquare::new();

    for i in 0..Board::SIZE {
        numbers[&i] = find_magic_number(
            &i,
            piece::BISHOP_RELEVANT_OCCUPANCY_BIT_COUNT[i as usize],
            Piece::Bishop,
        )
    }

    numbers
});

/// Calculates the number that can be used to index the attacks table.
///
/// * `magic_number` - "magic" (**generated** random) number, used to differentiate similar
///    `occupancies`, see [`find_magic_number`] to checkout the generation.
/// * `occupancies` - what squares are occupied
/// * `number_of_occupied_sports` - used to further differentiate indexes
pub fn calculate_magic_index(
    magic_number: u64,
    occupancies: u64,
    number_of_relevant_bits: u64,
) -> usize {
    // magic_number
    //     .wrapping_mul(magic_number)
    //     .wrapping_mul(occupancies) as usize
    //     >> 65 - number_of_relevant_bits
    occupancies.wrapping_mul(magic_number) as usize >> 64 - number_of_relevant_bits
}

fn find_magic_number(pos: &dyn BoardPos, relevant_bits: u64, piece: Piece) -> u64 {
    let is_da_thing = pos.idx() == Square::D4.idx()
        && relevant_bits == piece::BISHOP_RELEVANT_OCCUPANCY_BIT_COUNT[Square::D4.idx() as usize];

    let mut occupancies = [0u64; 4096];
    let mut attacks = [0u64; 4096];
    let mut used_attacks;

    let (attack_mask, get_attacks_for): (u64, fn(&dyn BoardPos, u64) -> u64) = match piece {
        Piece::Bishop => (
            piece::BISHOP_RELEVANT_MOVE_MASK[pos],
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
            let magic_index =
                calculate_magic_index(magic_number, occupancies[index], relevant_bits);

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
    use std::sync::atomic::{AtomicU32, Ordering};

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
