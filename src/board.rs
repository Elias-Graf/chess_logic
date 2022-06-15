use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

use crate::{bit_board, move_generator::Move, piece, square::Square, Color, Piece};

pub type BitBoardPerColor = [u64; 2];

impl Index<Color> for BitBoardPerColor {
    type Output = u64;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl IndexMut<Color> for BitBoardPerColor {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

pub trait BoardPos: Into<usize> + Copy {}
impl BoardPos for usize {}
impl BoardPos for Square {}

// TODO: Consider refactoring to use `i8` everywhere and save a bunch of casting.

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    pub bishops: BitBoardPerColor,
    pub can_black_castle_king_side: bool,
    pub can_black_castle_queen_side: bool,
    pub can_white_castle_king_side: bool,
    pub can_white_castle_queen_side: bool,
    pub en_passant_target_idx: Option<usize>,
    pub is_whites_turn: bool,
    pub king: BitBoardPerColor,
    pub knights: BitBoardPerColor,
    pub pawns: BitBoardPerColor,
    pub promote_idx: Option<usize>,
    pub queens: BitBoardPerColor,
    pub rooks: BitBoardPerColor,
}

impl Board {
    pub const HEIGHT: usize = 8;
    pub const WIDTH: usize = 8;
    pub const SIZE: usize = Self::HEIGHT * Self::WIDTH;

    /// Combines all bit boards into a single one.
    ///
    /// This is achieved using the `|` (bitwise or) operator.
    pub fn all_occupancies(&self) -> u64 {
        self.bishops[Color::Black]
            | self.king[Color::Black]
            | self.knights[Color::Black]
            | self.pawns[Color::Black]
            | self.queens[Color::Black]
            | self.rooks[Color::Black]
            | self.bishops[Color::White]
            | self.king[Color::White]
            | self.knights[Color::White]
            | self.pawns[Color::White]
            | self.queens[Color::White]
            | self.rooks[Color::White]
    }

    /// Clear (remove) a piece on the specified location
    pub fn clear(&mut self, color: Color, piece: Piece, pos: impl BoardPos) {
        let bit_board = match piece {
            Piece::Bishop => &mut self.bishops,
            Piece::King => &mut self.king,
            Piece::Knight => &mut self.knights,
            Piece::Pawn => &mut self.pawns,
            Piece::Queen => &mut self.queens,
            Piece::Rook => &mut self.rooks,
        };

        bit_board::clear_bit(&mut bit_board[color], pos.into());
    }

    pub fn do_move(&mut self, mv: Move) {
        self.clear(mv.piece_color(), mv.piece(), mv.src());
        self.set(mv.piece_color(), mv.piece(), mv.dst());
    }

    /// Get the pice ([`PieceInstance`]) on the specified location
    ///
    /// In case you know what piece of what color you are looking for, you should
    /// access the bit boards directly.
    pub fn get(&self, pos: impl BoardPos) -> Option<PieceInstance> {
        let i = pos.into();

        for color in [Color::Black, Color::White] {
            if bit_board::is_bit_set(self.bishops[color], i) {
                return Some(PieceInstance::new(color, Piece::Bishop));
            }
            if bit_board::is_bit_set(self.king[color], i) {
                return Some(PieceInstance::new(color, Piece::King));
            }
            if bit_board::is_bit_set(self.knights[color], i) {
                return Some(PieceInstance::new(color, Piece::Knight));
            }
            if bit_board::is_bit_set(self.pawns[color], i) {
                return Some(PieceInstance::new(color, Piece::Pawn));
            }
            if bit_board::is_bit_set(self.queens[color], i) {
                return Some(PieceInstance::new(color, Piece::Queen));
            }
            if bit_board::is_bit_set(self.rooks[color], i) {
                return Some(PieceInstance::new(color, Piece::Rook));
            }
        }

        None
    }

    pub fn is_pos_attacked_by(&self, pos: impl BoardPos, atk_color: &Color) -> bool {
        // Since the attacks are essentially mirrored for both sides, we just generate
        // the opponent attacks on the square to check. If the attack includes the
        // position if our piece, we can be attacked, and the reverse is also true.
        //
        // Let's say we want to see if a white pawn on E5 can attack the square D6:
        //
        // 8   . . . . . . . .
        // 7   . . . . . . . .
        // 6   . . . . . . . .
        // 5   . . . . 1 . . .
        // 4   . . . . . . . .
        // 3   . . . . . . . .
        // 2   . . . . . . . .
        // 1   . . . . . . . .
        //
        //     a b c d e f g h
        //
        // We now simply lookup the attacks of the **opponent** on the position we
        // want to check (pawn attacks of the square D6):
        //
        // 8   . . . . . . . .
        // 7   . . . . . . . .
        // 6   . . . . . . . .
        // 5   . . 1 . 1 . . .
        // 4   . . . . . . . .
        // 3   . . . . . . . .
        // 2   . . . . . . . .
        // 1   . . . . . . . .
        //
        //     a b c d e f g h
        //
        // We can see that the bit on E5 is set on both boards, thus the square
        // D6 can be attacked by the white pawn on E5.

        let all_pieces = self.all_occupancies();

        if bit_board::has_set_bits(
            piece::get_bishop_attacks_for(pos, all_pieces) & self.bishops[*atk_color],
        ) {
            return true;
        }

        if bit_board::has_set_bits(piece::get_king_attack_mask_for(pos) & self.king[*atk_color]) {
            return true;
        }

        if bit_board::has_set_bits(
            piece::get_knight_attack_mask_for(pos) & self.knights[*atk_color],
        ) {
            return true;
        }

        if bit_board::has_set_bits(
            piece::get_pawn_attacks_for(pos, &atk_color.opposing()) & self.pawns[*atk_color],
        ) {
            return true;
        }

        // The Queen attacks are already covered by checking bishops and rooks,
        // and not explicitly checked here.

        if bit_board::has_set_bits(
            piece::get_rook_attacks_for(pos, all_pieces) & self.rooks[*atk_color],
        ) {
            return true;
        }

        false
    }

    pub fn new_empty() -> Self {
        Self {
            bishops: [0; 2],
            can_black_castle_king_side: false,
            can_black_castle_queen_side: false,
            can_white_castle_king_side: false,
            can_white_castle_queen_side: false,
            en_passant_target_idx: None,
            is_whites_turn: true,
            king: [0; 2],
            knights: [0; 2],
            pawns: [0; 2],
            promote_idx: None,
            queens: [0; 2],
            rooks: [0; 2],
        }
    }

    pub fn new_with_standard_formation() -> Self {
        let mut board = Self::new_empty();

        board.can_black_castle_king_side = true;
        board.can_black_castle_queen_side = true;
        board.can_white_castle_king_side = true;
        board.can_white_castle_queen_side = true;

        // Standard chess formation:
        board.set(Color::Black, Piece::Rook, 0);
        board.set(Color::Black, Piece::Knight, 1);
        board.set(Color::Black, Piece::Bishop, 2);
        board.set(Color::Black, Piece::Queen, 3);
        board.set(Color::Black, Piece::King, 4);
        board.set(Color::Black, Piece::Bishop, 5);
        board.set(Color::Black, Piece::Knight, 6);
        board.set(Color::Black, Piece::Rook, 7);

        board.set(Color::Black, Piece::Pawn, 8);
        board.set(Color::Black, Piece::Pawn, 9);
        board.set(Color::Black, Piece::Pawn, 10);
        board.set(Color::Black, Piece::Pawn, 11);
        board.set(Color::Black, Piece::Pawn, 12);
        board.set(Color::Black, Piece::Pawn, 13);
        board.set(Color::Black, Piece::Pawn, 14);
        board.set(Color::Black, Piece::Pawn, 15);

        board.set(Color::White, Piece::Pawn, 48);
        board.set(Color::White, Piece::Pawn, 49);
        board.set(Color::White, Piece::Pawn, 50);
        board.set(Color::White, Piece::Pawn, 51);
        board.set(Color::White, Piece::Pawn, 52);
        board.set(Color::White, Piece::Pawn, 53);
        board.set(Color::White, Piece::Pawn, 54);
        board.set(Color::White, Piece::Pawn, 55);

        board.set(Color::White, Piece::Rook, 56);
        board.set(Color::White, Piece::Knight, 57);
        board.set(Color::White, Piece::Bishop, 58);
        board.set(Color::White, Piece::Queen, 59);
        board.set(Color::White, Piece::King, 60);
        board.set(Color::White, Piece::Bishop, 61);
        board.set(Color::White, Piece::Knight, 62);
        board.set(Color::White, Piece::Rook, 63);

        board
    }

    /// Set (add) a piece on the specified location
    pub fn set(&mut self, color: Color, piece: Piece, pos: impl BoardPos) {
        let i = pos.into();

        match piece {
            Piece::Bishop => bit_board::set_bit(&mut self.bishops[color], i),
            Piece::King => bit_board::set_bit(&mut self.king[color], i),
            Piece::Knight => bit_board::set_bit(&mut self.knights[color], i),
            Piece::Pawn => bit_board::set_bit(&mut self.pawns[color], i),
            Piece::Queen => bit_board::set_bit(&mut self.queens[color], i),
            Piece::Rook => bit_board::set_bit(&mut self.rooks[color], i),
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut val = String::new();

        for i in 0..Board::SIZE {
            let file = i % Board::HEIGHT;
            let rank = i / Board::HEIGHT;

            if file == 0 {
                val += &format!("{}  ", Board::HEIGHT - rank);
            }

            let sym = match self.get(i) {
                Some(ins) => ins.piece.symbol(ins.color).to_owned(),
                None => ".".to_owned(),
            };

            val += &format!(" {}", sym);

            if file == 7 {
                val += "\n";
            }
        }

        val += "\n    a b c d e f g h";

        val += "\n    side to move: ";
        val += if self.is_whites_turn {
            "white"
        } else {
            "black"
        };

        val += "\n    en passant target: ";
        val += &self
            .en_passant_target_idx
            .map(|i| format!("{:?}", Square::try_from(i).unwrap()))
            .unwrap_or_else(|| "<None>".to_owned());

        write!(f, "{}", val)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PieceInstance {
    pub color: Color,
    pub piece: Piece,
}

impl PieceInstance {
    pub fn new(color: Color, piece: Piece) -> Self {
        Self { piece, color }
    }
}

#[cfg(test)]
mod tests {
    use crate::fen::Fen;

    use super::*;

    use Color::*;
    use Piece::*;
    use Square::*;

    #[test]
    fn is_pos_attacked_not_attacked() {
        let board = Board::new_empty();

        assert_eq!(board.is_pos_attacked_by(A8, &Color::Black), false);
        assert_eq!(board.is_pos_attacked_by(A8, &Color::White), false);
    }

    #[test]
    fn is_pos_attacked_by_bishop_no_blockers() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(color.clone(), Piece::Bishop, F4);

            for pos in [B8, C7, D6, E5, H6, G5, E3, D2, C1, G3, H2] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_by_bishop_blockers() {
        const ALL_SQUARES_BEHIND: [Square; 3] = [E6, F7, G8];

        for atk_color in &[Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(atk_color.clone(), Piece::Bishop, B3);

            let var_name: [(Color, Piece, &[Square]); 12] = [
                // Opposing blocking pieces
                (atk_color.opposing(), Piece::Bishop, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::King, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Knight, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Pawn, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Queen, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Rook, &ALL_SQUARES_BEHIND),
                // It's a bit more tricky for friendly blocking pieces, since they
                // may attack themselves.
                (*atk_color, Piece::Bishop, &[]),
                (*atk_color, Piece::King, &[F7, G8]),
                (*atk_color, Piece::Knight, &ALL_SQUARES_BEHIND),
                (*atk_color, Piece::Pawn, &[F7, G8]),
                (*atk_color, Piece::Queen, &[]),
                (*atk_color, Piece::Rook, &ALL_SQUARES_BEHIND),
            ];
            for (blocking_color, blocking_piece, blocked_squares) in var_name {
                let mut board = board.clone();
                board.set(blocking_color, blocking_piece, D5);

                for pos in blocked_squares {
                    assert_eq!(
                        board.is_pos_attacked_by(*pos, &atk_color),
                        false,
                        "attacking: {:?}, blocking: {:?} {:?}",
                        atk_color,
                        atk_color,
                        blocking_piece
                    );
                }
            }
        }
    }

    #[test]
    fn is_pos_attacked_by_king() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(color.clone(), Piece::King, F7);

            for pos in [E8, F8, G8, E7, G7, E6, F6, G6] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_by_knight() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(color.clone(), Piece::Knight, B4);

            for pos in [A6, C6, D5, D3, C2, A2] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_attacked_by_white_pawn() {
        let mut board = Board::new_empty();
        board.set(Color::White, Piece::Pawn, E5);

        assert_eq!(board.is_pos_attacked_by(D6, &Color::White), true);
        assert_eq!(board.is_pos_attacked_by(F6, &Color::White), true);
    }

    #[test]
    fn is_pos_attacked_by_black_pawn() {
        let mut board = Board::new_empty();
        board.set(Color::Black, Piece::Pawn, C6);

        assert_eq!(board.is_pos_attacked_by(B5, &Color::Black), true);
        assert_eq!(board.is_pos_attacked_by(D5, &Color::Black), true);
    }

    // The queen checks are covered by the bishop and rook checks.
    // So they are not checked explicitly here.

    #[test]
    fn is_pos_attacked_by_rook_no_blockers() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(color.clone(), Piece::Rook, G7);

            for pos in [A7, B7, C7, D7, E7, F7, H7, G8, G6, G5, G4, G3, G2, G1] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_by_white_rook() {
        const ALL_SQUARES_BEHIND: [Square; 4] = [D5, D6, D7, D8];

        for atk_color in &[Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(atk_color.clone(), Piece::Rook, D2);

            let var_name: [(Color, Piece, &[Square]); 12] = [
                // Opposing blocking pieces
                (atk_color.opposing(), Piece::Bishop, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::King, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Knight, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Pawn, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Queen, &ALL_SQUARES_BEHIND),
                (atk_color.opposing(), Piece::Rook, &ALL_SQUARES_BEHIND),
                // It's a bit more tricky for friendly blocking pieces, since they
                // may attack themselves.
                (*atk_color, Piece::Bishop, &ALL_SQUARES_BEHIND),
                (*atk_color, Piece::King, &[D6, D7, D8]),
                (*atk_color, Piece::Knight, &ALL_SQUARES_BEHIND),
                (*atk_color, Piece::Pawn, &ALL_SQUARES_BEHIND),
                (*atk_color, Piece::Queen, &[]),
                (*atk_color, Piece::Rook, &[]),
            ];
            for (blocking_color, blocking_piece, blocked_squares) in var_name {
                let mut board = board.clone();
                board.set(blocking_color, blocking_piece, D4);

                for pos in blocked_squares {
                    assert_eq!(
                        board.is_pos_attacked_by(*pos, &atk_color),
                        false,
                        "attacking: {:?}, blocking: {:?} {:?}",
                        atk_color,
                        atk_color,
                        blocking_piece
                    );
                }
            }
        }
    }

    #[test]
    fn do_move() {
        let mut board = Board::new_empty();
        board.set(White, Pawn, A2);
        board.set(Black, Pawn, H7);
        board.set(Black, Knight, B8);

        board.do_move(Move::new(White, Pawn, A2, A3));
        assert_eq!(board.get_fen(), "1n6/7p/8/8/8/P7/8/8 w - - 0 0");

        board.do_move(Move::new(Black, Pawn, H7, H6));
        assert_eq!(board.get_fen(), "1n6/8/7p/8/8/P7/8/8 w - - 0 0");
        
        board.do_move(Move::new(Black, Knight, B8, A6));
        assert_eq!(board.get_fen(), "8/8/n6p/8/8/P7/8/8 w - - 0 0");
    }
}
