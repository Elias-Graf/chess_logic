use crate::Board;

pub fn is_set(board: &u64, i: &u64) -> bool {
    get_bit(board, i) > 0
}

pub fn get_bit(board: &u64, i: &u64) -> u64 {
    board & (1 << i)
}

pub fn set_bit(board: &mut u64, i: &u64) {
    *board |= 1 << i
}

pub fn clear_bit(board: &mut u64, i: &u64) {
    *board &= !(1 << i)
}

pub fn display(board: &u64) -> String {
    let mut val = String::new();

    for i in 0..Board::SIZE {
        let file = i % Board::HEIGHT;
        let rank = i / Board::HEIGHT;

        if file == 0 {
            val += &format!("{}  ", Board::HEIGHT - rank);
        }

        val += match get_bit(board, &(i as u64)) {
            0 => " 0",
            _ => " 1",
        };

        if file == 7 {
            val += "\n";
        }
    }

    val += "\n    a b c d e f g h";

    val
}
