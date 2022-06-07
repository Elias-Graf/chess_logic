use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

use crate::{
    bit_board,
    piece::{self, DIR_EAST, DIR_NORTH, DIR_OFFSETS, DIR_SOUTH, DIR_WEST, TO_EDGE_OFFSETS},
    square::{Square, _BoardPos},
    Color, Piece,
};

pub trait Move: Debug {
    fn src_idx(&self) -> i8;
    fn dst_idx(&self) -> i8;
}

pub type MoveByIdx = (usize, usize);
pub type MoveBySquare = (Square, Square);

impl Move for MoveByIdx {
    fn src_idx(&self) -> i8 {
        self.0 as i8
    }

    fn dst_idx(&self) -> i8 {
        self.1 as i8
    }
}

impl Move for MoveBySquare {
    fn src_idx(&self) -> i8 {
        self.0.into()
    }

    fn dst_idx(&self) -> i8 {
        self.1.into()
    }
}

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
    #[deprecated(note = "in the future the `en_passant_target_idx` should be used")]
    pub piece_eligible_for_en_passant: Vec<(usize, usize)>,
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
    pub fn all_pieces(&self) -> u64 {
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

    fn check_and_add_en_passant_eligibility(
        &mut self,
        src_ins: &PieceInstance,
        src_idx: usize,
        hit_idx: usize,
    ) {
        if src_ins.piece != Piece::Pawn {
            return;
        }

        let src_y = src_idx / Self::WIDTH;
        let hit_y = hit_idx / Self::WIDTH;

        let move_could_result_in_en_passant = src_y.abs_diff(hit_y) == 2;
        if !move_could_result_in_en_passant {
            return;
        }

        for hit_pos in [
            (hit_idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize,
            (hit_idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize,
        ] {
            if let Some(ins) = self.get(hit_pos) {
                if ins.color != src_ins.color {
                    self.piece_eligible_for_en_passant.push((hit_pos, hit_idx));
                }
            }
        }
    }

    /// Clear (remove) a piece on the specified location
    ///
    /// This function is not very performant and more of a pretty interface. If
    /// speed is important, one might consider accessing the piece fields (bit boards)
    /// directly.
    pub fn clear(&mut self, pos: impl BoardPos) {
        let i = pos.into();

        for color in [Color::Black, Color::White] {
            bit_board::clear_bit(&mut self.bishops[color], i);
            bit_board::clear_bit(&mut self.king[color], i);
            bit_board::clear_bit(&mut self.knights[color], i);
            bit_board::clear_bit(&mut self.pawns[color], i);
            bit_board::clear_bit(&mut self.queens[color], i);
            bit_board::clear_bit(&mut self.rooks[color], i);
        }
    }

    fn check_and_execute_castle(
        &mut self,
        src_ins: &PieceInstance,
        src_idx: usize,
        hit_idx: usize,
    ) {
        if !matches!(src_ins.piece, Piece::King) {
            return;
        }

        if src_idx.abs_diff(hit_idx) != 2 {
            return;
        }

        let king_move_dir = match src_idx as i8 - hit_idx as i8 > 0 {
            true => DIR_WEST,
            false => DIR_EAST,
        };

        let rook_src_idx = (src_idx as i8
            + (TO_EDGE_OFFSETS[src_idx][king_move_dir] as i8) * DIR_OFFSETS[king_move_dir])
            as usize;
        let rook_ins = self.get(rook_src_idx);
        let rook_des_idx = (hit_idx as i8 - DIR_OFFSETS[king_move_dir]) as usize;

        self.set_by_idx(rook_src_idx, None);
        self.set_by_idx(rook_des_idx, rook_ins);
    }

    fn check_and_execute_en_passant(&mut self, src_ins: &PieceInstance, hit_idx: usize) {
        if matches!(src_ins.piece, Piece::Pawn) && matches!(self.get(hit_idx), None) {
            let idx_behind_hit =
                ((hit_idx as i8) - DIR_OFFSETS[Self::get_attack_dir_of(&src_ins.color)]) as usize;

            self.set_by_idx(idx_behind_hit, None);
        }
    }

    /// Execute a move.
    /// Notably this does not check the move's validity, (if you want to do that use
    /// [`Piece::is_valid_move()`]). **However**, this function needs to check if
    /// given move is a special one (en passant or castle). So calling with invalid
    /// moves might result in severe side effects and crash the program.
    /// As an API consumer, it is advisable to only call with moves generated by
    /// [`Piece::get_moves_for_piece_at()`].
    ///
    /// # Panics
    /// If no piece is at the source index.
    pub fn do_move(&mut self, mv: &impl Move) {
        let src_idx = mv.src_idx();
        let dst_idx = mv.dst_idx();

        let src_ins = match self.get(src_idx as usize) {
            Some(i) => i,
            None => panic!(
                "cannot execute move '{:?}' because there is no piece at the source index",
                mv
            ),
        };

        self.remove_old_en_passant_moves();
        // Adds en passant moves for the next turn
        self.check_and_add_en_passant_eligibility(&src_ins, src_idx as usize, dst_idx as usize);
        // Execute en passant moves for this turn
        self.check_and_execute_en_passant(&src_ins, dst_idx as usize);

        if src_ins.piece == Piece::King {
            match src_ins.color {
                Color::Black => {
                    self.can_black_castle_king_side = false;
                    self.can_black_castle_queen_side = false;
                }
                Color::White => {
                    self.can_white_castle_king_side = false;
                    self.can_white_castle_queen_side = false;
                }
            }
        }

        if src_ins.piece == Piece::Rook {
            match (&src_ins.color, src_idx) {
                (Color::Black, 0) => self.can_black_castle_queen_side = false,
                (Color::Black, 7) => self.can_black_castle_king_side = false,
                (Color::White, 56) => self.can_white_castle_queen_side = false,
                (Color::White, 63) => self.can_white_castle_king_side = false,
                _ => (),
            }
        }

        self.check_and_execute_castle(&src_ins, src_idx as usize, dst_idx as usize);
        self.set_by_idx(src_idx as usize, None);
        self.set_by_idx(dst_idx as usize, Some(src_ins));
    }

    /// Get the pice ([`PieceInstance`]) on the specified location
    ///
    /// This function is not very performant and more of a pretty interface. If
    /// speed is important, one might consider accessing the piece fields (bit boards)
    /// directly.
    pub fn get(&self, pos: impl BoardPos) -> Option<PieceInstance> {
        let i = pos.into();

        for color in [Color::Black, Color::White] {
            if bit_board::is_set(self.bishops[color], i) {
                return Some(PieceInstance::new(color, Piece::Bishop));
            }
            if bit_board::is_set(self.king[color], i) {
                return Some(PieceInstance::new(color, Piece::King));
            }
            if bit_board::is_set(self.knights[color], i) {
                return Some(PieceInstance::new(color, Piece::Knight));
            }
            if bit_board::is_set(self.pawns[color], i) {
                return Some(PieceInstance::new(color, Piece::Pawn));
            }
            if bit_board::is_set(self.queens[color], i) {
                return Some(PieceInstance::new(color, Piece::Queen));
            }
            if bit_board::is_set(self.rooks[color], i) {
                return Some(PieceInstance::new(color, Piece::Rook));
            }
        }

        None
    }

    /// Get the direction ([`DIR_NORTH`]/[`DIR_SOUTH`]) in which the pieces are
    /// attacking. Basically the opposite side of where the pieces started.
    pub fn get_attack_dir_of(color: &Color) -> usize {
        match color {
            Color::Black => DIR_SOUTH,
            Color::White => DIR_NORTH,
        }
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

        let all_pieces = self.all_pieces();

        if bit_board::has_set_bits(
            piece::get_bishop_attacks_for(pos, all_pieces) & self.bishops[*atk_color],
        ) {
            return true;
        }

        if bit_board::has_set_bits(piece::get_king_attacks_for(pos) & self.king[*atk_color]) {
            return true;
        }

        if bit_board::has_set_bits(piece::get_knight_attacks_for(pos) & self.knights[*atk_color]) {
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
            piece_eligible_for_en_passant: Vec::with_capacity(2),
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
        board.set_by_idx(0, Some(PieceInstance::new(Color::Black, Piece::Rook)));
        board.set_by_idx(1, Some(PieceInstance::new(Color::Black, Piece::Knight)));
        board.set_by_idx(2, Some(PieceInstance::new(Color::Black, Piece::Bishop)));
        board.set_by_idx(3, Some(PieceInstance::new(Color::Black, Piece::Queen)));
        board.set_by_idx(4, Some(PieceInstance::new(Color::Black, Piece::King)));
        board.set_by_idx(5, Some(PieceInstance::new(Color::Black, Piece::Bishop)));
        board.set_by_idx(6, Some(PieceInstance::new(Color::Black, Piece::Knight)));
        board.set_by_idx(7, Some(PieceInstance::new(Color::Black, Piece::Rook)));

        board.set_by_idx(8, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(9, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(10, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(11, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(12, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(13, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(14, Some(PieceInstance::new(Color::Black, Piece::Pawn)));
        board.set_by_idx(15, Some(PieceInstance::new(Color::Black, Piece::Pawn)));

        board.set_by_idx(48, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(49, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(50, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(51, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(52, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(53, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(54, Some(PieceInstance::new(Color::White, Piece::Pawn)));
        board.set_by_idx(55, Some(PieceInstance::new(Color::White, Piece::Pawn)));

        board.set_by_idx(56, Some(PieceInstance::new(Color::White, Piece::Rook)));
        board.set_by_idx(57, Some(PieceInstance::new(Color::White, Piece::Knight)));
        board.set_by_idx(58, Some(PieceInstance::new(Color::White, Piece::Bishop)));
        board.set_by_idx(59, Some(PieceInstance::new(Color::White, Piece::Queen)));
        board.set_by_idx(60, Some(PieceInstance::new(Color::White, Piece::King)));
        board.set_by_idx(61, Some(PieceInstance::new(Color::White, Piece::Bishop)));
        board.set_by_idx(62, Some(PieceInstance::new(Color::White, Piece::Knight)));
        board.set_by_idx(63, Some(PieceInstance::new(Color::White, Piece::Rook)));

        board
    }

    /// Fullfil the outstanding promotion.
    ///
    /// Promote a pawn that has reached the end of the board to the specified piece.
    /// Possibilities to promote to are:
    /// - [`Piece::Bishop`]
    /// - [`Piece::Knight`]
    /// - [`Piece::Queen`]
    /// - [`Piece::Rook`]
    pub fn promote_piece_to(&mut self, promote_to: Piece) {
        assert!(
            matches!(
                promote_to,
                Piece::Bishop | Piece::Knight | Piece::Queen | Piece::Rook,
            ),
            "pawn cannot be promoted to '{:?}'",
            promote_to
        );

        let idx = self
            .promote_idx
            .expect("cannot promote as no outstanding promotion was detected");

        let ins = match self.get(idx) {
            Some(i) => i.clone(),
            None => panic!("no piece to promote at '{}'", idx),
        };

        assert!(
            matches!(ins.piece, Piece::Pawn),
            "can only promote pawns but piece ('{}') was '{:?}'",
            idx,
            ins.piece,
        );

        self.set_by_idx(idx, Some(PieceInstance::new(ins.color, promote_to)));

        self.promote_idx = None
    }

    fn remove_old_en_passant_moves(&mut self) {
        self.piece_eligible_for_en_passant.clear()
    }

    /// Set (add) a piece on the specified location
    ///
    /// This function is not very performant and more of a pretty interface. If
    /// speed is important, one might consider accessing the piece fields (bit boards)
    /// directly.
    pub fn set(&mut self, pos: impl BoardPos, color: Color, piece: Piece) {
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

    #[deprecated(note = "use `set` instead")]
    pub fn set_by_idx(&mut self, i: usize, pos: Option<PieceInstance>) {
        if let Some(PieceInstance { color, piece }) = pos {
            self.set(i, color, piece);
        } else {
            self.clear(i);
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
    use super::*;

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
            board.set(F4, color.clone(), Piece::Bishop);

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
            board.set(B3, atk_color.clone(), Piece::Bishop);

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
                board.set(D5, blocking_color, blocking_piece);

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
            board.set(F7, color.clone(), Piece::King);

            for pos in [E8, F8, G8, E7, G7, E6, F6, G6] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_by_knight() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(B4, color.clone(), Piece::Knight);

            for pos in [A6, C6, D5, D3, C2, A2] {
                assert_eq!(board.is_pos_attacked_by(pos, &color), true, "{:?}", &color);
            }
        }
    }

    #[test]
    fn is_pos_attacked_attacked_by_white_pawn() {
        let mut board = Board::new_empty();
        board.set(E5, Color::White, Piece::Pawn);

        assert_eq!(board.is_pos_attacked_by(D6, &Color::White), true);
        assert_eq!(board.is_pos_attacked_by(F6, &Color::White), true);
    }

    #[test]
    fn is_pos_attacked_by_black_pawn() {
        let mut board = Board::new_empty();
        board.set(C6, Color::Black, Piece::Pawn);

        assert_eq!(board.is_pos_attacked_by(B5, &Color::Black), true);
        assert_eq!(board.is_pos_attacked_by(D5, &Color::Black), true);
    }

    // The queen checks are covered by the bishop and rook checks.
    // So they are not checked explicitly here.

    #[test]
    fn is_pos_attacked_by_rook_no_blockers() {
        for color in [Color::Black, Color::White] {
            let mut board = Board::new_empty();
            board.set(G7, color.clone(), Piece::Rook);

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
            board.set(D2, atk_color.clone(), Piece::Rook);

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
                board.set(D4, blocking_color, blocking_piece);

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

    // TODO: move or remove pre bit board stuff
    // pre bit board stuff below

    #[test]
    fn en_passant_removes_the_other_piece_you() {
        let mut board = board();
        board.set_by_idx(8, ins_black(Piece::Pawn));
        board.set_by_idx(25, ins_white(Piece::Pawn));

        board.do_move(&(8, 24));
        board.do_move(&(25, 16));

        assert!(
            matches!(board.get(Square::A5), None),
            "piece was not removed {}",
            board
        );
    }

    #[test]
    fn en_passant_removes_the_other_piece_opponent() {
        let mut board = board();
        board.set_by_idx(33, ins_black(Piece::Pawn));
        board.set_by_idx(48, ins_white(Piece::Pawn));

        board.do_move(&(48, 32));
        board.do_move(&(33, 40));

        assert!(
            matches!(board.get(Square::A4), None),
            "piece was not removed {}",
            board
        );
    }

    #[test]
    fn en_passant_is_not_done_for_other_pieces() {
        let mut board = board();
        board.set_by_idx(33, ins_black(Piece::Bishop));
        board.set_by_idx(32, ins_white(Piece::Pawn));

        board.do_move(&(33, 40));

        assert!(
            !matches!(board.get(Square::A4), None),
            "piece was removed {}",
            board
        );
    }

    #[test]
    fn en_passant_is_not_executed_on_normal_take() {
        let mut board = board();
        board.set_by_idx(33, ins_white(Piece::Pawn));
        board.set_by_idx(34, ins_black(Piece::Pawn));
        board.set_by_idx(41, ins_white(Piece::Pawn));

        board.do_move(&(34, 41));

        assert!(
            !matches!(board.get(Square::B4), None),
            "piece was removed {}",
            board
        );
    }

    #[test]
    fn castle_moves_the_rook_you_west() {
        let mut board = board_castle_you_west();

        board.do_move(&(60, 58));

        assert_eq!(
            board.get(Square::A1).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(Square::D1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board
        );
    }

    #[test]
    fn castle_moves_the_rook_you_east() {
        let mut board = board_castle_you_east();

        board.do_move(&(60, 62));

        assert_eq!(
            board.get(Square::H1).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(Square::F1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board,
        );
    }

    #[test]
    fn castle_moves_the_rook_opponent_west() {
        let mut board = board();
        board.set_by_idx(0, ins_black(Piece::Rook));
        board.set_by_idx(4, ins_black(Piece::King));

        board.do_move(&(4, 2));

        assert_eq!(
            board.get(Square::A8).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(Square::D8).as_ref(),
            ins_black(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board,
        );
    }

    #[test]
    fn castle_moves_the_rook_opponent_east() {
        let mut board = board();
        board.set_by_idx(4, ins_black(Piece::King));
        board.set_by_idx(7, ins_black(Piece::Rook));

        board.do_move(&(4, 6));

        assert_eq!(
            board.get(Square::A7).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(Square::F8).as_ref(),
            ins_black(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board
        );
    }

    #[test]
    fn castle_only_moves_the_correct_rook_west() {
        let mut board = board_castle_you_west();
        board.set_by_idx(63, ins_white(Piece::Rook));

        board.do_move(&(60, 58));

        assert_eq!(
            board.get(Square::H1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "east rook was moved {}",
            board,
        );
    }

    #[test]
    fn castle_only_moves_the_correct_rook_east() {
        let mut board = board_castle_you_east();
        board.set_by_idx(56, ins_white(Piece::Rook));

        board.do_move(&(60, 62));

        assert_eq!(
            board.get(Square::A1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "west rook was moved {}",
            board,
        );
    }

    #[test]
    fn castle_does_not_happen_on_normal_king_moves() {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::Rook));
        board.set_by_idx(60, ins_white(Piece::King));

        board.do_move(&(60, 59));

        assert_eq!(
            board.get(Square::A1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "rook was removed",
        );
    }

    #[test]
    fn castle_move_is_not_done_for_other_pieces() {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::Rook));
        board.set_by_idx(60, ins_white(Piece::Queen));

        board.do_move(&(60, 58));

        assert_eq!(
            board.get(Square::A1).as_ref(),
            ins_white(Piece::Rook).as_ref(),
            "rook was replaced {}",
            board
        );
    }

    fn board_castle_you_west() -> Board {
        let mut board = board();
        board.set_by_idx(56, ins_white(Piece::Rook));
        board.set_by_idx(60, ins_white(Piece::King));
        board
    }

    fn board_castle_you_east() -> Board {
        let mut board = board();
        board.set_by_idx(60, ins_white(Piece::King));
        board.set_by_idx(63, ins_white(Piece::Rook));
        board
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins_black(piece: Piece) -> Option<PieceInstance> {
        ins(Color::Black, piece)
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins_white(piece: Piece) -> Option<PieceInstance> {
        ins(Color::White, piece)
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins(color: Color, piece: Piece) -> Option<PieceInstance> {
        Some(PieceInstance::new(color, piece))
    }

    fn board() -> Board {
        Board::new_empty()
    }
}
