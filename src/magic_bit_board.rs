//! Contains the move generation for the more complicated sliding pieces.
//!
//! Sliding pieces have to "stop dead in their tracks" when hitting another piece,
//! discarding all moves that could have followed. Therefore, **lookup tables**
//! are generated for each square, for each variant of pieces that could block the
//! progressing of sliding moves (also called occupancies).
//!
//! **These tables are index by so called "magic numbers".** There is nothing special
//! or magic about them. They are just a random number that were found to lead to
//! the correct set of moves when processed by certain mathematical operations.
//! So if you read anything related to "magic number" or "magic index" in this module,
//! simply think "randomly generated number" that can be used to generate an index
//! in the move lookup table.

use std::{cmp::min, collections::HashMap};

use once_cell::sync::Lazy;

use crate::{
    bit_board::{self, U64PerSquare},
    piece,
    type_alias_default::TypeAliasDefault,
    Board, Piece,
};

/// Returns the bishop moves for a given position, with given blockers.
///
/// Abstracts away all the table lookups maths. Read the module-level documentation
/// for more information.
pub fn get_bishop_attacks_for(idx: usize, blockers: u64) -> u64 {
    let magic_index = magic_index_of(
        BISHOP_MAGIC_NUMBERS[idx],
        blockers,
        RELEVANT_BISHOP_MOVES_PER_SQUARE[idx],
        NUMBER_OF_RELEVANT_BISHOP_MOVES_PER_SQUARE[idx] as usize,
    );

    ALL_POSSIBLE_BISHOP_ATTACKS[magic_index][idx]
}

/// Same as [`get_bishop_attacks_for`], but for rooks.
pub fn get_rook_attacks_for(idx: usize, blockers: u64) -> u64 {
    let magic_index = magic_index_of(
        ROOK_MAGIC_NUMBERS[idx],
        blockers,
        RELEVANT_ROOK_MOVES_PER_SQUARE[idx],
        NUMBER_OF_RELEVANT_ROOK_MOVES_PER_SQUARE[idx] as usize,
    );

    ALL_POSSIBLE_ROOK_ATTACKS[magic_index][idx]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bishop_compare_to_slow_to_generate_source_of_truth() {
        for i in 0..Board::SIZE {
            let truth = piece::calculate_bishop_attacks_for(i, 0);
            let lookup_result = get_bishop_attacks_for(i, 0);

            assert_eq!(truth, lookup_result);
        }
    }

    #[test]
    fn rook_compare_to_slow_to_generate_source_of_truth() {
        for i in 0..Board::SIZE {
            let truth = piece::calculate_rook_attacks_for(i, 0);
            let lookup_result = get_rook_attacks_for(i, 0);

            assert_eq!(truth, lookup_result);
        }
    }
}

/// Generated using [`generate_bishop_magic_numbers`].
///
/// # Example
///
/// ```ignore
/// static BISHOP_MAGIC_NUMBERS: Lazy<U64PerSquare> = Lazy::new(generate_bishop_magic_numbers);
/// ```
const BISHOP_MAGIC_NUMBERS: U64PerSquare = [
    0x5020198102048600,
    0x514690801010400,
    0xc250340050482842,
    0x8002408101006082,
    0x804030800200001,
    0x8210404c7300,
    0x4407209010182088,
    0x2000120082084080,
    0x82c101010214840,
    0x608802008029,
    0x4002106732002100,
    0x14250a003020,
    0x201011040502000,
    0x220110888240,
    0x3182088808090404,
    0x4410902088084800,
    0x80100c409001a102,
    0x144012104040050,
    0x4208008100440a80,
    0x2088808802014004,
    0x4000215200804,
    0x1802840602000,
    0x20010001441a2100,
    0x101824108980922,
    0x448840009200840,
    0x204300084501480,
    0x2280004080022,
    0x44004088081100,
    0xa101010000104003,
    0x10010013828081,
    0x4102020084111109,
    0x2104010020288212,
    0x11102804102090,
    0x102104502900104,
    0x102044044040102,
    0x20100820040400,
    0x4080209a0200,
    0x4042004100220080,
    0x4082040020102,
    0x40a08200808200,
    0x2022022180401,
    0x2023044228202,
    0x406410003101,
    0x80002093000801,
    0x2024200a4002200,
    0x2020408112000640,
    0x2160880081012081,
    0x8109102110044,
    0x944020802091044,
    0x480410088200000,
    0x4400002c02080080,
    0x800800042020000,
    0x820000903040200,
    0x30914a1001120400,
    0xc2451010260a8020,
    0x80508204812a0004,
    0x440411011085,
    0x8004a0602110400,
    0x229001900880400,
    0x2000000918208840,
    0x4600080210202205,
    0xc20000e143118210,
    0x410a002040840,
    0x47c2410420200,
];
/// Generated using [`generate_rook_magic_numbers`].
///
/// # Example
///
/// ```ignore
/// static ROOK_MAGIC_NUMBERS: Lazy<U64PerSquare> = Lazy::new(generate_rook_magic_numbers);
/// ```
const ROOK_MAGIC_NUMBERS: U64PerSquare = [
    0x28002a010400180,
    0xc040001000200040,
    0x880100220000880,
    0x2480041000c80080,
    0x1001002040108008,
    0x5300080a15000400,
    0x2100450004820014,
    0x200082702420084,
    0x1980218000c004,
    0x1400802000804002,
    0x110802000801008,
    0xa801000900100020,
    0x9800400810800,
    0x880200100200c448,
    0x8034002401300802,
    0x101d00010002a0ca,
    0x40008000208044,
    0x63000c000c02000,
    0x40420012002880,
    0x10004008004400,
    0x1044008080080004,
    0x120808002000400,
    0x4000040011308208,
    0x200020005540881,
    0x1040008180004224,
    0x180200240005001,
    0x1a004200102089,
    0x800090100201000,
    0x8040080800800,
    0x40080020080,
    0x40010400081002,
    0x2001000100004082,
    0x8040804000800024,
    0x8420002080804000,
    0x4801802004801002,
    0x90021001000,
    0x4326080080800400,
    0x2000400800280,
    0x480108220c000110,
    0x802006082000104,
    0x420400224858000,
    0x25000c1a0024000,
    0x200010008080,
    0x9400100100210008,
    0x4201020800110004,
    0x440040002008080,
    0x84100210c8040001,
    0xc4010100a0420004,
    0x120980004008e180,
    0x5160200040100040,
    0x608441200802200,
    0x13800800100380,
    0x200480100104500,
    0x200200410400801,
    0x8000092208100c00,
    0x1220204910840200,
    0x21308008210041,
    0x10020043002080b6,
    0x144406a11200101,
    0x2001120400806,
    0x45001004020801,
    0x2000104081082,
    0x2082100220a10804,
    0x6042084104102,
];

static RELEVANT_BISHOP_MOVES_PER_SQUARE: Lazy<U64PerSquare> =
    Lazy::new(generate_relevant_bishop_moves_per_square);
static RELEVANT_ROOK_MOVES_PER_SQUARE: Lazy<U64PerSquare> =
    Lazy::new(generate_relevant_rook_moves_per_square);

// TODO: potentially convert to [usize; 64], and remove casts where this constant
// is used.
/// The number of relevant moves (see [`generate_relevant_bishop_moves_per_square`]) that are
/// possible on each square.
#[rustfmt::skip]
const NUMBER_OF_RELEVANT_BISHOP_MOVES_PER_SQUARE: U64PerSquare = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];
// TODO: potentially convert to [usize; 64], and remove casts where this constant
// is used.
/// Same as [`NUMBER_OF_RELEVANT_BISHOP_MOVES_PER_SQUARE`], but for rooks
#[rustfmt::skip]
const NUMBER_OF_RELEVANT_ROOK_MOVES_PER_SQUARE: [u64; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

static ALL_POSSIBLE_BISHOP_ATTACKS: Lazy<Box<[U64PerSquare; 4096]>> =
    Lazy::new(generate_all_possible_bishop_attacks);
static ALL_POSSIBLE_ROOK_ATTACKS: Lazy<Box<[U64PerSquare; 4096]>> =
    Lazy::new(generate_all_possible_rook_attacks);

/// Read the module-level documentation for more information.
///
/// (This code is "dead" because the generated results are cached. See
/// [`BISHOP_MAGIC_NUMBERS`]).
#[allow(dead_code)]
fn generate_bishop_magic_numbers() -> U64PerSquare {
    let mut numbers = U64PerSquare::default();

    for i in 0..Board::SIZE {
        numbers[i] = generate_magic_number_for(i, Piece::Bishop);
    }

    numbers
}

/// Same as [`generate_bishop_magic_numbers`], but for rooks.
#[allow(dead_code)]
fn generate_rook_magic_numbers() -> U64PerSquare {
    let mut numbers = U64PerSquare::default();

    for i in 0..Board::SIZE {
        numbers[i] = generate_magic_number_for(i, Piece::Rook);
    }

    numbers
}

/// Generates all the possible (relevant moves) for all squares.
///
/// Relevant moves are essentially all the move expect the very outer ones. That
/// is, because it doesn't really matter if the square is empty or occupied by a
/// piece. In both scenarios it's possible to move to the square. Either to an empty
/// one, or simply move and take the piece.
///
/// The squares on the very edge of the board are considered "irrelevant" for some
/// of the calculations in this module.
///
/// # Example (bishop would be on E4)
///
/// ```text
/// 8   . . . . . . . .
/// 7   . 1 . . . . . .
/// 6   . . 1 . . . 1 .
/// 5   . . . 1 . 1 . .
/// 4   . . . . . . . .
/// 3   . . . 1 . 1 . .
/// 2   . . 1 . . . 1 .
/// 1   . . . . . . . .
///
///     a b c d e f g h
/// ```
fn generate_relevant_bishop_moves_per_square() -> U64PerSquare {
    let mut moves = U64PerSquare::default();

    for i in 0..Board::SIZE {
        let board = bit_board::with_bit_at(i);

        let file = i % Board::HEIGHT;
        let rank = i / Board::HEIGHT;

        let to_no_ea = min((Board::WIDTH - 1) - file, rank);
        let to_so_ea = min(Board::WIDTH - file, Board::HEIGHT - rank) - 1;
        let to_so_we = min(file, (Board::HEIGHT - 1) - rank);
        let to_no_we = min(file, rank);

        for iter in 1..to_no_ea {
            moves[i] |= board >> bit_board::NO_EA * iter;
        }
        for iter in 1..to_so_ea {
            moves[i] |= board << bit_board::SO_EA * iter;
        }
        for iter in 1..to_so_we {
            moves[i] |= board << bit_board::SO_WE * iter;
        }
        for iter in 1..to_no_we {
            moves[i] |= board >> bit_board::NO_WE * iter;
        }
    }

    moves
}

/// Same as [`generate_relevant_bishop_moves_per_square`], but for rooks.
fn generate_relevant_rook_moves_per_square() -> U64PerSquare {
    let mut moves = U64PerSquare::default();

    for i in 0..Board::SIZE {
        let board = bit_board::with_bit_at(i);

        let file = i % Board::HEIGHT;
        let rank = i / Board::HEIGHT;

        let to_north = rank;
        let to_east = (Board::WIDTH - file) - 1;
        let to_south = (Board::HEIGHT - rank) - 1;
        let to_west = file;

        for iter in 1..to_north {
            moves[i] |= board >> bit_board::NORTH * iter;
        }
        for iter in 1..to_east {
            moves[i] |= board << bit_board::EAST * iter;
        }
        for iter in 1..to_south {
            moves[i] |= board << bit_board::SOUTH * iter;
        }
        for iter in 1..to_west {
            moves[i] |= board >> bit_board::WEST * iter;
        }
    }

    moves
}

fn generate_all_possible_bishop_attacks() -> Box<[U64PerSquare; 4096]> {
    generate_all_possible_attacks_for(
        &RELEVANT_BISHOP_MOVES_PER_SQUARE,
        &NUMBER_OF_RELEVANT_BISHOP_MOVES_PER_SQUARE,
        &BISHOP_MAGIC_NUMBERS,
        piece::calculate_bishop_attacks_for,
    )
}

fn generate_all_possible_rook_attacks() -> Box<[U64PerSquare; 4096]> {
    generate_all_possible_attacks_for(
        &RELEVANT_ROOK_MOVES_PER_SQUARE,
        &NUMBER_OF_RELEVANT_ROOK_MOVES_PER_SQUARE,
        &ROOK_MAGIC_NUMBERS,
        piece::calculate_rook_attacks_for,
    )
}

fn generate_all_possible_attacks_for(
    all_relevant_moves: &U64PerSquare,
    number_of_all_relevant_moves: &U64PerSquare,
    magic_numbers: &U64PerSquare,
    calculate_attacks_for: fn(usize, u64) -> u64,
) -> Box<[U64PerSquare; 4096]> {
    let mut all_attacks: Box<[U64PerSquare; 4096]> = vec![U64PerSquare::default(); 4096]
        .into_boxed_slice()
        .try_into()
        .unwrap();

    for i in 0..Board::SIZE {
        let relevant_moves = all_relevant_moves[i];
        let number_of_relevant_moves = number_of_all_relevant_moves[i] as usize;

        for occupancy_idx in 0..number_of_occupancy_variants(number_of_relevant_moves) {
            let occupancy_variant =
                bb::move_occupancy_variant(occupancy_idx, number_of_relevant_moves, relevant_moves);
            let magic_index = magic_index_of(
                magic_numbers[i],
                occupancy_variant,
                relevant_moves,
                number_of_relevant_moves,
            );

            all_attacks[magic_index][i] = calculate_attacks_for(i, occupancy_variant);
        }
    }

    all_attacks
}

fn generate_magic_number_for(idx: usize, piece: Piece) -> u64 {
    let idx = idx.into();

    let (relevant_moves, number_of_relevant_moves, get_attacks_for): (
        u64,
        usize,
        fn(usize, u64) -> u64,
    ) = match piece {
        Piece::Bishop => (
            RELEVANT_BISHOP_MOVES_PER_SQUARE[idx],
            NUMBER_OF_RELEVANT_BISHOP_MOVES_PER_SQUARE[idx] as usize,
            piece::calculate_bishop_attacks_for,
        ),
        Piece::Rook => (
            RELEVANT_ROOK_MOVES_PER_SQUARE[idx],
            NUMBER_OF_RELEVANT_ROOK_MOVES_PER_SQUARE[idx] as usize,
            piece::calculate_rook_attacks_for,
        ),
        _ => panic!(
            "this function is only callable for bishops and rooks, was called with '{:?}'",
            piece
        ),
    };

    let number_of_occupancy_variants = number_of_occupancy_variants(number_of_relevant_moves);
    let mut attacks = Box::new([0u64; 4096]);
    let mut occupancy_variants = Box::new([0u64; 4096]);

    for occupancy_idx in 0..number_of_occupancy_variants {
        let variant =
            bb::move_occupancy_variant(occupancy_idx, number_of_relevant_moves, relevant_moves);

        attacks[occupancy_idx] = get_attacks_for(idx, variant);
        occupancy_variants[occupancy_idx] = variant;
    }

    let mut tested_indexes: HashMap<usize, u64> = HashMap::with_capacity(512);

    const GENERATION_TRIES: u64 = 10000000000000;
    'generation_try: for _ in 0..GENERATION_TRIES {
        let magic_number = get_magic_number_candidate();

        // TODO: figure out what the point of this is
        if bit_board::count_set_bits(
            (relevant_moves.overflowing_mul(magic_number).0) & 0xFF00000000000000,
        ) < 6
        {
            continue;
        }

        tested_indexes.clear();

        for i in 0..number_of_occupancy_variants {
            let magic_index = magic_index_of(
                magic_number,
                occupancy_variants[i],
                relevant_moves,
                number_of_relevant_moves,
            );

            if let Some(previous_attack) = tested_indexes.get(&magic_index) {
                if previous_attack != &attacks[i] {
                    continue 'generation_try;
                }
            } else {
                tested_indexes.insert(magic_index, attacks[i]);
            }
        }

        return magic_number;
    }

    panic!("magic number generation failed")
}

/// Calculates how many variants of occupants there are.
///
/// Takes in all the squares a move could reach, and calculates the amount of variants
/// of pieces on those squares.
///
/// TODO: add example.
fn number_of_occupancy_variants(number_of_relevant_moves: usize) -> usize {
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
fn magic_index_of(
    magic_number: u64,
    mut occupancies: u64,
    relevant_move_mask: u64,
    number_of_relevant_moves: usize,
) -> usize {
    occupancies &= relevant_move_mask;
    occupancies.wrapping_mul(magic_number) as usize >> 64 - number_of_relevant_moves
}

/// Generate a number that has a low amount of bits set to one.
fn get_magic_number_candidate() -> u64 {
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
        occupancy_idx: usize,
        number_of_relevant_moves: usize,
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
