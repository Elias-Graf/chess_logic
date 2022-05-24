use std::mem;

use crate::{
    piece::{DIR_EAST, DIR_NORTH, DIR_OFFSETS, DIR_SOUTH, DIR_WEST, TO_EDGE_OFFSETS},
    Color, Piece, Player,
};

pub type Move = (usize, usize);

// TODO: Consider refactoring to use `i8` everywhere and save a bunch of casting.

#[derive(Clone)]
pub struct Board {
    pub opponent_color: Color,
    pub piece_eligible_for_en_passant: Vec<(usize, usize)>,
    pub poses: [Option<PieceInstance>; Self::SIZE as usize],
    pub promote_idx: Option<usize>,
    #[deprecated]
    pub selected_idx: Option<usize>,
    pub you_color: Color,
}

impl Board {
    pub const HEIGHT: usize = 8;
    pub const WIDTH: usize = 8;
    pub const SIZE: usize = Self::HEIGHT * Self::WIDTH;

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
                if ins.player != src_ins.player {
                    self.piece_eligible_for_en_passant.push((hit_pos, hit_idx));
                }
            }
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
        let rook_ins = self.replace_pos(rook_src_idx, None);
        let rook_des_idx = (hit_idx as i8 - DIR_OFFSETS[king_move_dir]) as usize;

        self.set(rook_des_idx, rook_ins);
    }

    fn check_and_execute_en_passant(&mut self, src_ins: &PieceInstance, hit_idx: usize) {
        if matches!(src_ins.piece, Piece::Pawn) && matches!(self.get(hit_idx), None) {
            let idx_behind_hit =
                ((hit_idx as i8) - DIR_OFFSETS[self.get_attack_dir_of(&src_ins.player)]) as usize;

            self.set(idx_behind_hit, None);
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
    /// Notably this does not check the move's validity, (if you want to do that use
    /// [`Piece::is_valid_move()`]). **However**, this function needs to check if
    /// given move is a special one (en passant or castle). So calling with invalid
    /// moves might result in severe side effects and crash the program.
    /// As an API consumer, it is advisable to only call with moves generated by
    /// [`Piece::get_moves_for_piece_at()`].
    ///
    /// # Panics
    /// If no piece is at the source index.
    pub fn do_move(&mut self, mv: Move) {
        let (src_idx, hit_idx) = mv;
        let src_ins = match self.replace_pos(src_idx, None) {
            Some(i) => i,
            None => panic!(
                "cannot execute move '{:?}' because there is no piece at the source index",
                mv
            ),
        };

        self.remove_old_en_passant_moves();
        // Adds en passant moves for the next turn
        self.check_and_add_en_passant_eligibility(&src_ins, src_idx, hit_idx);
        // Execute en passant moves for this turn
        self.check_and_execute_en_passant(&src_ins, hit_idx);

        self.check_and_execute_castle(&src_ins, src_idx, hit_idx);

        self.set(hit_idx, Some(src_ins));
    }

    pub fn get(&self, idx: usize) -> &Option<PieceInstance> {
        &self.poses[idx]
    }

    /// Get the direction ([`DIR_NORTH`]/[`DIR_SOUTH`]) in which the pieces are
    /// attacking. Basically the opposite side of where the pieces started.
    pub fn get_attack_dir_of(&self, player: &Player) -> usize {
        match player {
            Player::You => DIR_NORTH,
            Player::Opponent => DIR_SOUTH,
        }
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

    pub fn is_pos_attacked_by(&self, pos_idx: usize, attacker: &Player) -> bool {
        for (iter, pos) in self.poses.iter().enumerate() {
            let ins = match pos {
                Some(i) => i,
                None => continue,
            };

            if &ins.player != attacker {
                continue;
            }

            for (_, hit_idx) in Piece::get_moves_of_piece_at(iter, self) {
                if pos_idx == hit_idx {
                    return true;
                }
            }
        }

        false
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

        move_ins.was_moved = true;

        self.poses[from_idx] = None;
        self.poses[to_idx] = Some(move_ins);

        self.selected_idx = None;

        self.check_if_pawn_needs_promoting(to_idx);

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

    /// Replaced a given `idx` on the board with a given `val`.
    ///
    /// The replaced value will be returned.
    fn replace_pos(&mut self, idx: usize, val: Option<PieceInstance>) -> Option<PieceInstance> {
        mem::replace(&mut self.poses[idx], val)
    }

    pub fn set(&mut self, idx: usize, ins: Option<PieceInstance>) {
        self.poses[idx] = ins;
    }

    #[deprecated]
    pub fn set_selected(&mut self, idx: usize) {
        self.selected_idx = Some(idx);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
    fn en_passant_removes_the_other_piece_you() {
        let mut board = board();
        board.set(8, ins_opp(Piece::Pawn));
        board.set(25, ins_you(Piece::Pawn));

        board.do_move((8, 24));
        board.do_move((25, 16));

        assert!(
            matches!(board.get(24), None),
            "piece was not removed {}",
            board
        );
    }

    #[test]
    fn en_passant_removes_the_other_piece_opponent() {
        let mut board = board();
        board.set(33, ins_opp(Piece::Pawn));
        board.set(48, ins_you(Piece::Pawn));

        board.do_move((48, 32));
        board.do_move((33, 40));

        assert!(
            matches!(board.get(32), None),
            "piece was not removed {}",
            board
        );
    }

    #[test]
    fn en_passant_is_not_done_for_other_pieces() {
        let mut board = board();
        board.set(33, ins_opp(Piece::Bishop));
        board.set(32, ins_you(Piece::Pawn));

        board.do_move((33, 40));

        assert!(
            !matches!(board.get(32), None),
            "piece was removed {}",
            board
        );
    }

    #[test]
    fn en_passant_is_not_executed_on_normal_take() {
        let mut board = board();
        board.set(33, ins_you(Piece::Pawn));
        board.set(34, ins_opp(Piece::Pawn));
        board.set(41, ins_you(Piece::Pawn));

        board.do_move((34, 41));

        assert!(
            !matches!(board.get(33), None),
            "piece was removed {}",
            board
        );
    }

    #[test]
    fn castle_moves_the_rook_you_west() {
        let mut board = board_castle_you_west();

        board.do_move((60, 58));

        assert_eq!(
            board.get(56).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(59).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board
        );
    }

    #[test]
    fn castle_moves_the_rook_you_east() {
        let mut board = board_castle_you_east();

        board.do_move((60, 62));

        assert_eq!(
            board.get(63).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(61).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board,
        );
    }

    #[test]
    fn castle_moves_the_rook_opponent_west() {
        let mut board = board();
        board.set(0, ins_opp(Piece::Rook));
        board.set(4, ins_opp(Piece::King));

        board.do_move((4, 2));

        assert_eq!(
            board.get(0).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(3).as_ref(),
            ins_opp(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board,
        );
    }

    #[test]
    fn castle_moves_the_rook_opponent_east() {
        let mut board = board();
        board.set(4, ins_opp(Piece::King));
        board.set(7, ins_opp(Piece::Rook));

        board.do_move((4, 6));

        assert_eq!(
            board.get(7).as_ref(),
            None,
            "rook was not removed {}",
            board,
        );

        assert_eq!(
            board.get(5).as_ref(),
            ins_opp(Piece::Rook).as_ref(),
            "rook was not moved {}",
            board
        );
    }

    #[test]
    fn castle_only_moves_the_correct_rook_west() {
        let mut board = board_castle_you_west();
        board.set(63, ins_you(Piece::Rook));

        board.do_move((60, 58));

        assert_eq!(
            board.get(63).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "east rook was moved {}",
            board,
        );
    }

    #[test]
    fn castle_only_moves_the_correct_rook_east() {
        let mut board = board_castle_you_east();
        board.set(56, ins_you(Piece::Rook));

        board.do_move((60, 62));

        assert_eq!(
            board.get(56).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "west rook was moved {}",
            board,
        );
    }

    #[test]
    fn castle_does_not_happen_on_normal_king_moves() {
        let mut board = board();
        board.set(56, ins_you(Piece::Rook));
        board.set(60, ins_you(Piece::King));

        board.do_move((60, 59));

        assert_eq!(
            board.get(56).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "rook was removed",
        );
    }

    #[test]
    fn castle_move_is_not_done_for_other_pieces() {
        let mut board = board();
        board.set(56, ins_you(Piece::Rook));
        board.set(60, ins_you(Piece::Queen));

        board.do_move((60, 58));

        assert_eq!(
            board.get(56).as_ref(),
            ins_you(Piece::Rook).as_ref(),
            "rook was replaced {}",
            board
        );
    }

    fn board_castle_you_west() -> Board {
        let mut board = board();
        board.set(56, ins_you(Piece::Rook));
        board.set(60, ins_you(Piece::King));
        board
    }

    fn board_castle_you_east() -> Board {
        let mut board = board();
        board.set(60, ins_you(Piece::King));
        board.set(63, ins_you(Piece::Rook));
        board
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins_opp(piece: Piece) -> Option<PieceInstance> {
        ins(Player::Opponent, piece)
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins_you(piece: Piece) -> Option<PieceInstance> {
        ins(Player::You, piece)
    }

    /// Note that the instance is created with `was_moved = true`.
    fn ins(player: Player, piece: Piece) -> Option<PieceInstance> {
        let mut ins = PieceInstance::new(player, piece);
        ins.was_moved = true;

        Some(ins)
    }

    fn board() -> Board {
        Board::new(Color::Black, Color::White)
    }
}
