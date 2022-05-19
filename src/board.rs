use crate::{
    info_board::{self, PosInfo},
    piece::{DIR_EAST, DIR_NORTH, DIR_OFFSETS, DIR_SOUTH, DIR_WEST, TO_EDGE_OFFSETS},
    Color, InfoBoard, Piece, Player,
};

pub type Move = (usize, usize);

#[derive(Clone)]
pub struct Board {
    pub poses: [Option<PieceInstance>; Self::SIZE as usize],
    pub opponent_color: Color,
    pub promote_idx: Option<usize>,
    pub selected_idx: Option<usize>,
    pub you_color: Color,
}

impl Board {
    pub const HEIGHT: usize = 8;
    pub const WIDTH: usize = 8;
    pub const SIZE: usize = Self::HEIGHT * Self::WIDTH;

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

    pub fn get(&self, idx: usize) -> &Option<PieceInstance> {
        &self.poses[idx]
    }

    pub fn get_color_of_player(&self, player: &Player) -> &Color {
        match player {
            Player::Opponent => &self.opponent_color,
            Player::You => &self.you_color,
        }
    }

    pub fn get_moves_of_selected(&self) -> InfoBoard {
        let mut info_board: InfoBoard = self.into();

        let idx = match self.selected_idx {
            Some(i) => i,
            None => panic!("cannot get moves of selected as nothing was selected"),
        };

        Piece::add_moves_for_piece(idx, &mut info_board);

        if let Some(ins) = &self.poses[idx] {
            if ins.is_eligible_for_en_passant {
                Piece::add_en_passant_moves_to_board(idx, ins, &mut info_board);
            }
        }
        self.remove_moves_that_result_in_check(idx, &mut info_board);

        info_board
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut PieceInstance> {
        self.poses[idx].as_mut()
    }

    pub fn get_selected(&self) -> &Option<usize> {
        &self.selected_idx
    }

    pub fn is_king_in_check(&self, check_player: &Player) -> bool {
        let mut info_board: InfoBoard = self.into();
        let mut king_idx = None;

        for (idx, pos) in self.poses.as_ref().iter().enumerate() {
            let ins = match pos {
                Some(i) => i,
                _ => continue,
            };

            if matches!(ins.piece, Piece::King) && &ins.player == check_player {
                king_idx = Some(idx);
            }

            if &ins.player != check_player {
                Piece::add_moves_for_piece(idx, &mut info_board);
            }
        }

        let king_idx = king_idx.expect(&format!(
            "could not find king of player: '{:?}'",
            check_player
        ));

        matches!(info_board.get(king_idx), PosInfo::PieceHit(_))
    }

    fn make_piece_eligible_for_en_passant_if_appropriate(
        &mut self,
        moved_ins: &PieceInstance,
        moved_distance: usize,
        check_idx: usize,
    ) {
        if moved_distance != 2 {
            return;
        }

        if !matches!(moved_ins.piece, Piece::Pawn) {
            return;
        }

        if let Some(ins) = self.get_mut(check_idx) {
            ins.is_eligible_for_en_passant = true;
        }
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
    pub fn move_selected_to(&mut self, to_idx: usize) -> bool {
        if let Some(promote_idx) = self.promote_idx {
            panic!(
                "cannot move a piece while a promotion (at '{}') is outstanding",
                promote_idx
            );
        }

        let selected_idx = match self.selected_idx {
            None => panic!("cannot move when no piece was selected, use `Self::update_selection()` to select a piece"),
            Some(i) => i,
        };

        let mut move_ins = if let Some(instance) = &self.poses[selected_idx] {
            instance.clone()
        } else {
            panic!("no piece found to move at index '{}'", selected_idx);
        };

        let info_board = self.get_moves_of_selected();

        let is_valid_move = matches!(
            info_board.get(to_idx),
            info_board::PosInfo::Move | info_board::PosInfo::PieceHit(_)
        );

        if !is_valid_move {
            return false;
        }

        // En passant is only valid for the turn immediately after.
        self.remove_old_en_passant();

        let selected_x = selected_idx % Board::WIDTH;
        let selected_y = selected_idx / Board::HEIGHT;
        let to_x = to_idx % Board::WIDTH;
        let to_y = to_idx / Board::HEIGHT;
        let moved_distance = selected_y.abs_diff(to_y);

        self.make_piece_eligible_for_en_passant_if_appropriate(
            &move_ins,
            moved_distance,
            (to_idx as i8 + DIR_OFFSETS[DIR_EAST]) as usize,
        );
        self.make_piece_eligible_for_en_passant_if_appropriate(
            &move_ins,
            moved_distance,
            (to_idx as i8 + DIR_OFFSETS[DIR_WEST]) as usize,
        );

        if self.move_was_successful_en_passant(to_idx, &move_ins) {
            self.remove_piece_passed_by_en_passant(to_idx, &move_ins);
        }

        move_ins.was_moved = true;

        let move_is_castle =
            matches!(move_ins.piece, Piece::King) && selected_x.abs_diff(to_x) == 2;

        self.poses[selected_idx] = None;
        self.poses[to_idx] = Some(move_ins);

        self.selected_idx = None;

        self.check_if_pawn_needs_promoting(to_idx);

        if move_is_castle {
            self.move_rook_for_castle(selected_idx, to_x as i8);
        }

        true
    }

    fn get_wander_dir_of(&self, player: &Player) -> usize {
        match player {
            Player::You => DIR_NORTH,
            Player::Opponent => DIR_SOUTH,
        }
    }

    fn move_was_successful_en_passant(&mut self, to_idx: usize, ins: &PieceInstance) -> bool {
        if !matches!(ins.piece, Piece::Pawn) {
            return false;
        }

        let moved_to_empty_square = matches!(self.poses[to_idx], None);
        let moved_behind_pawn = self.poses
            [(to_idx as i8 - DIR_OFFSETS[self.get_wander_dir_of(&ins.player)]) as usize]
            .as_ref()
            .map_or(false, |behind| matches!(behind.piece, Piece::Pawn));

        moved_to_empty_square && moved_behind_pawn && ins.is_eligible_for_en_passant
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        // https://github.com/rust-lang/rust/issues/44796
        const INIT: Option<PieceInstance> = None;
        Self {
            poses: [INIT; Self::SIZE as usize],
            opponent_color,
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

    fn remove_moves_that_result_in_check(&self, idx: usize, info_board: &mut InfoBoard) {
        let mut piece = self.get(idx).as_ref().unwrap().clone();
        piece.was_moved = true;

        for (_, to_idx) in info_board.moves.clone() {
            let new_pos_info = match info_board.get(to_idx) {
                PosInfo::Move => PosInfo::None,
                PosInfo::PieceHit(ins) => PosInfo::Piece(ins.clone()),
                _ => continue,
            };

            let mut next_move = self.clone();
            next_move.poses[idx] = None;
            next_move.poses[to_idx] = Some(piece.clone());

            if next_move.is_king_in_check(&piece.player) {
                info_board.set(to_idx, new_pos_info);
            }
        }
    }

    fn remove_old_en_passant(&mut self) {
        for pos in &mut self.poses {
            if let Some(ins) = pos {
                ins.is_eligible_for_en_passant = false;
            }
        }
    }

    fn remove_piece_passed_by_en_passant(&mut self, to_idx: usize, ins: &PieceInstance) {
        self.set(
            (to_idx as i8 - DIR_OFFSETS[self.get_wander_dir_of(&ins.player)]) as usize,
            None,
        )
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
    /// This flag determines if a pawn can make the en passant move this round.
    /// It only makes sense when used together with an instance of a pawn, and is
    /// otherwise ignored.
    pub is_eligible_for_en_passant: bool,
    pub piece: Piece,
    pub player: Player,
    pub was_moved: bool,
}

impl PieceInstance {
    pub fn new(player: Player, piece: Piece) -> Self {
        Self {
            is_eligible_for_en_passant: false,
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
        let mut pawn = PieceInstance::new(Player::You, Piece::Pawn);
        pawn.was_moved = false;

        let mut board = Board::new(Color::Black, Color::White);
        board.set(49, Some(pawn.clone()));
        board.set(0, ins(Player::Opponent, Piece::King));
        board.set(13, ins(Player::Opponent, Piece::Bishop));

        board.selected_idx = Some(13);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(41), PosInfo::Move));
    }

    #[test]
    fn moves_that_would_result_in_a_check_are_not_added() {
        let mut board = Board::new(Color::Black, Color::White);

        board.set(49, ins(Player::You, Piece::King));
        board.set(42, ins(Player::You, Piece::Rook));

        board.set(35, ins(Player::Opponent, Piece::Bishop));
        board.set(58, ins(Player::Opponent, Piece::Rook));

        board.set_selected(49);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(56), PosInfo::None));
        assert!(matches!(info_board.get(57), PosInfo::None));

        board.selected_idx = Some(42);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(34), PosInfo::None));
        assert!(
            matches!(info_board.get(58), PosInfo::Piece(_)),
            "{:?}",
            info_board.get(58)
        );
    }

    #[test]
    fn en_passant_check_is_not_done_if_it_is_outside_the_board() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(48, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(33, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(48);
        assert!(board.move_selected_to(32));
    }

    #[test]
    fn en_passant_your_east_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(26, ins(Player::You, Piece::Pawn));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(11, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));

        board.set_selected(11);
        board.move_selected_to(27);

        board.set_selected(26);
        assert!(
            !board.move_selected_to(17),
            "en passant was added to the wrong side"
        );

        assert!(
            board.move_selected_to(19),
            "east en passant move was not accepted"
        );
        assert!(
            matches!(board.get(27), None),
            "east en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_your_west_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(28, ins(Player::You, Piece::Pawn));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(11, Some(PieceInstance::new(Player::Opponent, Piece::Pawn)));

        board.set_selected(11);
        board.move_selected_to(27);

        board.set_selected(28);
        assert!(
            !board.move_selected_to(21),
            "west en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(19),
            "west en passant move was not accepted"
        );
        assert!(
            matches!(board.get(27), None),
            "west en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_opponent_east_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(51, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(34, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(51);
        board.move_selected_to(35);

        board.set_selected(34);
        assert!(
            !board.move_selected_to(41),
            "east en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(43),
            "east en passant move was not accepted"
        );
        assert!(
            matches!(board.get(35), None),
            "east en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_opponent_west_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(51, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(36, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(51);
        board.move_selected_to(35);

        board.set_selected(36);
        assert!(
            !board.move_selected_to(45),
            "west en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(43),
            "west en passant move was not accepted"
        );
        assert!(
            matches!(board.get(35), None),
            "west en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_is_only_added_when_piece_is_eligible() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(43, ins(Player::You, Piece::Pawn));
        board.set(53, ins(Player::You, Piece::Rook));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(36, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(43);
        board.move_selected_to(35);

        board.set_selected(36);
        assert!(
            !board.move_selected_to(43),
            "en passant was added even though the enemy only moved one square"
        );

        board.set_selected(53);
        board.move_selected_to(37);

        board.set_selected(36);
        assert!(
            !board.move_selected_to(45),
            "en passant was added event though moved piece wasn't a pawn"
        );
    }

    #[test]
    fn en_passant_can_only_be_done_immediately_after_the_opponent_piece_was_moved() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(51, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(36, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(51);
        board.move_selected_to(35);

        board.set_selected(7);
        board.move_selected_to(15);

        board.set_selected(36);
        assert!(!board.move_selected_to(43));
    }

    #[test]
    fn en_passant_pieces_are_only_taken_if_the_move_actually_was_one() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(49, ins(Player::You, Piece::Bishop));
        board.set(7, ins(Player::Opponent, Piece::King));
        board.set(36, ins(Player::Opponent, Piece::Pawn));

        board.set_selected(49);
        board.move_selected_to(28);

        assert!(matches!(board.get(36), Some(_)));
    }

    #[test]
    fn en_passant_move_check_is_only_done_for_pawn() {
        // If the checks are done for all pieces (which in itself would be fine),
        // it is possible that they are done for pieces on the end of the board,
        // which results in an out of bound access.

        let mut board = Board::new(Color::Black, Color::White);
        board.set(56, ins(Player::You, Piece::King));
        board.set(0, ins(Player::Opponent, Piece::King));

        board.set_selected(0);
        board.move_selected_to(1);
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
