use std::mem;

use crate::{
    piece::{DIR_EAST, DIR_NORTH, DIR_OFFSETS, DIR_SOUTH, DIR_WEST, TO_EDGE_OFFSETS},
    Color, Piece, Player,
};

pub type Move = (usize, usize);

#[derive(Clone)]
pub struct Board {
    pub opponent_color: Color,
    pub piece_eligible_for_en_passant: Vec<(usize, usize)>,
    pub poses: [Option<PieceInstance>; Self::SIZE as usize],
    pub promote_idx: Option<usize>,
    pub selected_idx: Option<usize>,
    pub you_color: Color,
}

impl Board {
    pub const HEIGHT: usize = 8;
    pub const WIDTH: usize = 8;
    pub const SIZE: usize = Self::HEIGHT * Self::WIDTH;

    fn check_and_add_en_passant_eligibility(&mut self, src_idx: usize, hit_idx: usize) {
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
            if self.get(hit_pos).is_some() {
                self.piece_eligible_for_en_passant.push((hit_pos, hit_idx));
            }
        }
    }

    /// Check if a given piece is a pawn, and if it reached the end of the board.
    /// If the end of the board was reached, the position will be saved in promote
    /// pos.
    fn check_if_pawn_needs_promoting(&mut self, idx: usize) {
        let ins = match &self.poses[idx] {
            Some(i) => i,
            None => return,
        };

        let reached_end_of_board =
            TO_EDGE_OFFSETS[idx][DIR_NORTH] == 0 || TO_EDGE_OFFSETS[idx][DIR_SOUTH] == 0;

        if matches!(ins.piece, Piece::Pawn) && reached_end_of_board {
            // self.promote_pos = Some((x, y));
            panic!("can currently not set the promote piece - convert it to idx first")
        }
    }

    /// Execute a move.
    /// Notably this does not check the moves validity, (if you want to do that use
    /// [`Piece::is_valid_move()`]), but it does...
    ///
    /// # Panics
    /// If no piece is at the source index.
    pub fn do_move(&mut self, mv: Move) {
        let (src_idx, hit_idx) = mv;
        let src_ins = mem::replace(&mut self.poses[src_idx], None);

        if src_ins.is_none() {
            panic!(
                "cannot execute move '{:?}' because there is no piece at the source index",
                mv
            );
        }

        self.remove_old_en_passant_moves();
        self.check_and_add_en_passant_eligibility(src_idx, hit_idx);

        self.set(hit_idx, src_ins);
    }

    pub fn get(&self, idx: usize) -> &Option<PieceInstance> {
        &self.poses[idx]
    }

    pub fn get_color_of_player(&self, player: &Player) -> &Color {
        match player {
            Player::Opponent => &self.opponent_color,
            Player::You => &self.you_color,
        }
    }

    #[deprecated(note = "just use `Piece::get_moves_of_piece_at()` instead")]
    pub fn get_moves_of_selected(&self) -> Vec<Move> {
        let idx = match self.selected_idx {
            Some(i) => i,
            None => panic!("cannot get move of selected, as nothing is selected"),
        };

        Piece::get_moves_of_piece_at(idx, self)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut PieceInstance> {
        self.poses[idx].as_mut()
    }

    pub fn get_selected(&self) -> &Option<usize> {
        &self.selected_idx
    }

    pub fn is_king_in_check(&self, check_player: &Player) -> bool {
        // let mut info_board: InfoBoard = self.into();
        // let mut king_idx = None;

        // for (idx, pos) in self.poses.as_ref().iter().enumerate() {
        //     let ins = match pos {
        //         Some(i) => i,
        //         _ => continue,
        //     };

        //     if matches!(ins.piece, Piece::King) && &ins.player == check_player {
        //         king_idx = Some(idx);
        //     }

        //     if &ins.player != check_player {
        //         Piece::add_moves_for_piece(idx, &mut info_board);
        //     }
        // }

        // let king_idx = king_idx.expect(&format!(
        //     "could not find king of player: '{:?}'",
        //     check_player
        // ));

        // matches!(info_board.get(king_idx), PosInfo::PieceHit(_))
        todo!()
    }

    fn move_rook_for_castle(&mut self, king_idx: usize, king_x_to: i8) {
        // let (rook_x_from, rook_x_to) = if king_x_from - king_x_to < 0 {
        //     (Self::WIDTH as i8 - 1, king_x_from + 1)
        // } else {
        //     (0, king_x_from - 1)
        // };

        // let rook_instance = match self.get_old(rook_x_from, king_y) {
        //     Some(rook_instance) => rook_instance.clone(),
        //     _ => panic!("this function should not be called if no rook is at the castle location"),
        // };

        // self.set_old(rook_x_from, king_y, None);
        // self.set_old(rook_x_to, king_y, Some(rook_instance));
        panic!("currently not implemented")
    }

    // TODO: rework.
    /// Moves the currently selected piece to the specified index (if possible).
    /// One may select a piece using [`Self::set_selected()`].
    ///
    /// Returns `true` if the piece was moved.
    #[deprecated(note = "use `Self::do_move()` instead")]
    pub fn move_selected_to(&mut self, to_idx: usize) -> bool {
        if let Some(promote_idx) = self.promote_idx {
            panic!(
                "cannot move a piece while a promotion (at '{}') is outstanding",
                promote_idx
            );
        }

        let from_idx = match self.selected_idx {
            None => panic!("cannot move when no piece was selected, use `Self::update_selection()` to select a piece"),
            Some(i) => i,
        };

        let mut move_ins = if let Some(instance) = &self.poses[from_idx] {
            instance.clone()
        } else {
            panic!("no piece found to move at index '{}'", from_idx);
        };

        let moves = self.get_moves_of_selected();
        let is_valid_move = moves.contains(&(from_idx, to_idx));

        if !is_valid_move {
            return false;
        }

        let selected_x = from_idx % Board::WIDTH;
        let to_x = to_idx % Board::WIDTH;

        move_ins.was_moved = true;

        let move_is_castle =
            matches!(move_ins.piece, Piece::King) && selected_x.abs_diff(to_x) == 2;

        self.poses[from_idx] = None;
        self.poses[to_idx] = Some(move_ins);

        self.selected_idx = None;

        self.check_if_pawn_needs_promoting(to_idx);

        if move_is_castle {
            self.move_rook_for_castle(from_idx, to_x as i8);
        }

        true
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        // https://github.com/rust-lang/rust/issues/44796
        const INIT_POS: Option<PieceInstance> = None;
        Self {
            opponent_color,
            piece_eligible_for_en_passant: Vec::with_capacity(2),
            poses: [INIT_POS; Self::SIZE as usize],
            promote_idx: None,
            selected_idx: None,
            you_color,
        }
    }

    pub fn new_with_standard_formation(you_color: Color, opponent_color: Color) -> Self {
        let mut board = Self::new(you_color, opponent_color);

        // Standard chess formation:
        board.set(0, Some(PieceInstance::new(Player::Opponent, Piece::Rook)));
        board.set(1, Some(PieceInstance::new(Player::Opponent, Piece::Knight)));
        board.set(2, Some(PieceInstance::new(Player::Opponent, Piece::Bishop)));
        board.set(3, Some(PieceInstance::new(Player::Opponent, Piece::Queen)));
        board.set(4, Some(PieceInstance::new(Player::Opponent, Piece::King)));
        board.set(5, Some(PieceInstance::new(Player::Opponent, Piece::Bishop)));
        board.set(6, Some(PieceInstance::new(Player::Opponent, Piece::Knight)));
        board.set(7, Some(PieceInstance::new(Player::Opponent, Piece::Rook)));

        board.set(8, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(9, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(10, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(11, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(12, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(13, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(14, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));
        board.set(15, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));

        board.set(48, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(49, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(50, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(51, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(52, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(53, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(54, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(55, Some(PieceInstance::new(Player::You, Piece::Pawn)));

        board.set(56, Some(PieceInstance::new(Player::You, Piece::Rook)));
        board.set(57, Some(PieceInstance::new(Player::You, Piece::Knight)));
        board.set(58, Some(PieceInstance::new(Player::You, Piece::Bishop)));
        board.set(59, Some(PieceInstance::new(Player::You, Piece::Queen)));
        board.set(60, Some(PieceInstance::new(Player::You, Piece::King)));
        board.set(61, Some(PieceInstance::new(Player::You, Piece::Bishop)));
        board.set(62, Some(PieceInstance::new(Player::You, Piece::Knight)));
        board.set(63, Some(PieceInstance::new(Player::You, Piece::Rook)));

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

        self.set(idx, Some(PieceInstance::new(ins.player, promote_to)));

        self.promote_idx = None
    }

    fn remove_old_en_passant_moves(&mut self) {
        self.piece_eligible_for_en_passant.clear()
    }

    pub fn set(&mut self, idx: usize, ins: Option<PieceInstance>) {
        self.poses[idx] = ins;
    }

    pub fn set_selected(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
    }
}

#[derive(Clone, Debug)]
pub struct PieceInstance {
    pub piece: Piece,
    pub player: Player,
    pub was_moved: bool,
}

impl PieceInstance {
    pub fn new(player: Player, piece: Piece) -> Self {
        Self {
            piece,
            player,
            was_moved: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_king_in_check_you_not_in_check() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));

        assert!(!board.is_king_in_check(&Player::You));
    }

    #[test]
    fn is_king_in_check_you_in_check() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(49, ins(Player::Opponent, Piece::Pawn));

        assert!(board.is_king_in_check(&Player::You));
    }

    #[test]
    fn is_king_in_check_you_not_in_check_but_opponent_is() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, ins(Player::You, Piece::King));
        board.set(9, ins(Player::You, Piece::Pawn));
        board.set(2, ins(Player::Opponent, Piece::King));

        assert!(!board.is_king_in_check(&Player::You));
        assert!(board.is_king_in_check(&Player::Opponent));
    }

    #[test]
    fn moves_that_result_in_a_piece_bing_taken_are_added() {
        // let mut pawn = PieceInstance::new(Player::You, Piece::Pawn);
        // pawn.was_moved = false;

        // let mut board = Board::new(Color::Black, Color::White);
        // board.set(49, Some(pawn.clone()));
        // board.set(0, ins(Player::Opponent, Piece::King));
        // board.set(13, ins(Player::Opponent, Piece::Bishop));

        // board.selected_idx = Some(13);

        // let info_board = board.get_moves_of_selected();

        // assert!(matches!(info_board.get(41), PosInfo::Move));
        panic!("not implemented")
    }

    #[test]
    fn moves_that_would_result_in_a_check_are_not_added() {
        // let mut board = Board::new(Color::Black, Color::White);

        // board.set(49, ins(Player::You, Piece::King));
        // board.set(42, ins(Player::You, Piece::Rook));

        // board.set(35, ins(Player::Opponent, Piece::Bishop));
        // board.set(58, ins(Player::Opponent, Piece::Rook));

        // board.set_selected(49);

        // let info_board = board.get_moves_of_selected();

        // assert!(matches!(info_board.get(56), PosInfo::None));
        // assert!(matches!(info_board.get(57), PosInfo::None));

        // board.selected_idx = Some(42);

        // let info_board = board.get_moves_of_selected();

        // assert!(matches!(info_board.get(34), PosInfo::None));
        // assert!(
        //     matches!(info_board.get(58), PosInfo::Piece(_)),
        //     "{:?}",
        //     info_board.get(58)
        // );
        panic!("not implemented");
    }

    /// Create a new piece instance.
    ///
    /// Not that the piece will always be created as **was moved**. Without that, the logic would
    /// fail in certain cases, depending on where the piece is placed.
    fn ins(player: Player, piece: Piece) -> Option<PieceInstance> {
        let mut ins = PieceInstance::new(player, piece);
        ins.was_moved = true;

        Some(ins)
    }
}
