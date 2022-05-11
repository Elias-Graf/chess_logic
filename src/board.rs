use std::fmt::{self, Display};

use crate::{
    info_board::{self, PosInfo},
    Color, InfoBoard, Piece, Player,
};

#[derive(Clone)]
pub struct Board {
    board: Vec<Vec<Option<PieceInstance>>>,
    height: i8,
    opponent_color: Color,
    promote_pos: Option<(i8, i8)>,
    selected_pos: Option<(i8, i8)>,
    width: i8,
    you_color: Color,
}

impl Board {
    /// Check if a given piece is a pawn, and if it reached the end of the board.
    /// If the end of the board was reached, the position will be saved in promote
    /// pos.
    fn check_if_pawn_needs_promoting(&mut self, x: i8, y: i8) {
        let instance = match self.get(x, y) {
            Some(instance) => instance,
            None => return,
        };

        let reached_end_of_board = y == 0 || y == self.height - 1;

        if matches!(instance.piece, Piece::Pawn) && reached_end_of_board {
            self.promote_pos = Some((x, y));
        }
    }

    pub fn get(&self, x: i8, y: i8) -> &Option<PieceInstance> {
        assert!(
            self.is_in_bounds(x, y),
            "cannot get piece outside of board ({}/{})",
            x,
            y
        );

        &self.board[x as usize][y as usize]
    }

    pub fn get_color_of_player(&self, player: &Player) -> &Color {
        match player {
            Player::Opponent => &self.opponent_color,
            Player::You => &self.you_color,
        }
    }

    fn get_display_fg_color_for_piece_instance(&self, instance: Option<&PieceInstance>) -> &str {
        const FG_BLACK: &str = "\u{001b}[38;5;0m";
        const FG_WHITE: &str = "\u{001b}[38;5;15m";

        if let Some(instance) = instance {
            return if matches!(self.get_color_of_player(&instance.player), Color::Black) {
                FG_BLACK
            } else {
                FG_WHITE
            };
        }

        ""
    }

    fn get_display_square_bg_color(&self, x: i8, y: i8) -> &str {
        const BG_BLACK: &str = "\u{001b}[48;5;126m";
        const BG_WHITE: &str = "\u{001b}[48;5;145m";

        let is_even_row = y % 2 == 0;
        let is_even_column = x % 2 == 0;

        if is_even_row && is_even_column || !is_even_row && !is_even_column {
            return BG_WHITE;
        }

        BG_BLACK
    }

    fn get_display_symbol_for_piece_instance(&self, instance: Option<&PieceInstance>) -> String {
        if let Some(instance) = instance {
            return Piece::get_symbol(&instance.piece);
        }

        "  ".to_owned()
    }

    pub fn get_moves_of_selected(&self) -> InfoBoard {
        let mut info_board: InfoBoard = self.into();

        let (x, y) = match self.selected_pos {
            Some(pos) => pos,
            None => panic!("cannot get moves of selected as nothing was selected"),
        };

        Piece::add_moves_for_piece_at_to_board(x, y, &mut info_board);

        if let Some(ins) = self.get(x, y) {
            // TODO: refactor
            if ins.is_eligible_for_en_passant {
                let dir = Piece::get_pawn_direction(ins);

                let east_x = x + 1;
                if self.is_in_bounds(east_x, y)
                    && matches!(info_board.get(east_x, y), PosInfo::Piece(_))
                {
                    info_board.set(east_x, y + dir, PosInfo::Move);
                }

                let west_x = x - 1;
                if self.is_in_bounds(west_x, y)
                    && matches!(info_board.get(west_x, y), PosInfo::Piece(_))
                {
                    info_board.set(west_x, y + dir, PosInfo::Move);
                }
            }
        }

        self.remove_moves_that_result_in_check(x, y, &mut info_board);

        info_board
    }

    pub fn get_mut(&mut self, x: i8, y: i8) -> Option<&mut PieceInstance> {
        assert!(
            self.is_in_bounds(x, y),
            "cannot get piece outside of board ({}/{})",
            x,
            y
        );

        self.board[x as usize][y as usize].as_mut()
    }

    pub fn get_promote_pos(&self) -> Option<(i8, i8)> {
        self.promote_pos
    }

    pub fn get_selected(&self) -> &Option<(i8, i8)> {
        &self.selected_pos
    }

    pub fn height(&self) -> i8 {
        return self.height;
    }

    /// Checks if a position is within the bounds of the board.
    ///
    /// The variable might safely be cased to [`usize`] after `true` was returned
    /// from this function.
    pub fn is_in_bounds(&self, x: i8, y: i8) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }

    pub fn is_king_in_check(&self, check_player: &Player) -> bool {
        let mut info_board: InfoBoard = self.into();
        let mut king_pos = None;

        for (x, y) in self.iter_over_positions() {
            let ins = match self.get(x, y) {
                Some(ins) => ins,
                _ => continue,
            };

            if matches!(ins.piece, Piece::King) {
                if &ins.player == check_player {
                    king_pos = Some((x, y));
                }
            }

            Piece::add_moves_for_piece_at_to_board(x, y, &mut info_board);
        }

        let (king_x, king_y) = king_pos.expect(&format!(
            "could not find king of player: '{:?}'",
            check_player
        ));

        matches!(info_board.get(king_x, king_y), PosInfo::PieceHit(_))
    }

    pub fn iter_over_positions(&self) -> PosInter {
        PosInter {
            height: self.height,
            width: self.width,
            x: 0,
            y: 0,
        }
    }

    fn make_piece_eligible_for_en_passant_if_appropriate(
        &mut self,
        moved_ins: &PieceInstance,
        moved_distance: u8,
        check_x: i8,
        check_y: i8,
    ) {
        if !self.is_in_bounds(check_x, check_y) {
            return;
        }

        if moved_distance != 2 {
            return;
        }

        if !matches!(moved_ins.piece, Piece::Pawn) {
            return;
        }

        if let Some(ins) = self.get_mut(check_x, check_y) {
            ins.is_eligible_for_en_passant = true;
        }
    }

    fn move_rook_for_castle(&mut self, king_x_from: i8, king_x_to: i8, king_y: i8) {
        let (rook_x_from, rook_x_to) = if king_x_from - king_x_to < 0 {
            (self.width - 1, king_x_from + 1)
        } else {
            (0, king_x_from - 1)
        };

        let rook_instance = match self.get(rook_x_from, king_y) {
            Some(rook_instance) => rook_instance.clone(),
            _ => panic!("this function should not be called if no rook is at the castle location"),
        };

        self.set(rook_x_from as usize, king_y as usize, None);
        self.set(rook_x_to as usize, king_y as usize, Some(rook_instance));
    }

    // TODO: rework.
    /// Moves the currently selected piece to the specified position (if possible).
    /// One may select a piece using [`Self::update_selected()`].
    ///
    /// Returns `true` if the piece was moved.
    pub fn move_selected_to(&mut self, to_x: i8, to_y: i8) -> bool {
        assert!(
            self.is_in_bounds(to_x, to_y),
            "cannot move piece outside the board (move to {}/{})",
            to_x,
            to_y,
        );

        if let Some((promote_x, promote_y)) = self.promote_pos {
            panic!(
                "cannot move a piece while a promotion (at {}/{}) is outstanding",
                promote_x, promote_y
            );
        }

        let (piece_x, piece_y) = match self.selected_pos {
            None => panic!("cannot move when no piece was selected, use `Self::update_selection()` to select a piece"),
            Some(pos) => pos,
        };

        let mut move_ins = if let Some(instance) = self.get(piece_x, piece_y) {
            instance.clone()
        } else {
            panic!("no piece found to move at position {}/{}", piece_x, piece_y);
        };

        let info_board = self.get_moves_of_selected();

        let is_valid_move = matches!(
            info_board.get(to_x, to_y),
            info_board::PosInfo::Move | info_board::PosInfo::PieceHit(_)
        );

        if !is_valid_move {
            return false;
        }

        // En passant is only valid for the turn immediately after.
        self.remove_old_en_passant();

        self.make_piece_eligible_for_en_passant_if_appropriate(
            &move_ins,
            piece_y.abs_diff(to_y),
            to_x + 1,
            to_y,
        );
        self.make_piece_eligible_for_en_passant_if_appropriate(
            &move_ins,
            piece_y.abs_diff(to_y),
            to_x - 1,
            to_y,
        );

        if self.move_was_successful_en_passant(to_x, to_y, &move_ins) {
            self.remove_piece_passed_by_en_passant(to_x, to_y, &move_ins);
        }

        move_ins.was_moved = true;

        let move_is_castle = matches!(move_ins.piece, Piece::King) && (to_x - piece_x).abs() == 2;

        self.set(piece_x as usize, piece_y as usize, None);
        self.set(to_x as usize, to_y as usize, Some(move_ins));

        self.selected_pos = None;

        self.check_if_pawn_needs_promoting(to_x, to_y);

        if move_is_castle {
            self.move_rook_for_castle(piece_x, to_x, piece_y);
        }

        true
    }

    fn move_was_successful_en_passant(&mut self, to_x: i8, to_y: i8, ins: &PieceInstance) -> bool {
        let moved_to_empty_square = matches!(self.get(to_x, to_y), None);
        let moved_behind_pawn = self
            .get(to_x, to_y - Piece::get_pawn_direction(ins))
            .as_ref()
            .map_or(false, |behind| matches!(behind.piece, Piece::Pawn));

        moved_to_empty_square && moved_behind_pawn && ins.is_eligible_for_en_passant
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        let width = 8;
        let height = 8;

        Self {
            board: vec![vec![None; width]; height],
            height: height as i8,
            opponent_color,
            promote_pos: None,
            selected_pos: None,
            width: width as i8,
            you_color,
        }
    }

    pub fn new_with_standard_formation(you_color: Color, opponent_color: Color) -> Self {
        let mut board = Self::new(you_color, opponent_color);

        // Standard chess formation:
        board.set_piece(0, 0, Player::Opponent, Piece::Rook);
        board.set_piece(1, 0, Player::Opponent, Piece::Knight);
        board.set_piece(2, 0, Player::Opponent, Piece::Bishop);
        board.set_piece(3, 0, Player::Opponent, Piece::Queen);
        board.set_piece(4, 0, Player::Opponent, Piece::King);
        board.set_piece(5, 0, Player::Opponent, Piece::Bishop);
        board.set_piece(6, 0, Player::Opponent, Piece::Knight);
        board.set_piece(7, 0, Player::Opponent, Piece::Rook);

        board.set_piece(0, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(1, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(2, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(3, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(4, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(5, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(6, 1, Player::Opponent, Piece::Pawn);
        board.set_piece(7, 1, Player::Opponent, Piece::Pawn);

        board.set_piece(0, 7, Player::You, Piece::Rook);
        board.set_piece(1, 7, Player::You, Piece::Knight);
        board.set_piece(2, 7, Player::You, Piece::Bishop);
        board.set_piece(3, 7, Player::You, Piece::Queen);
        board.set_piece(4, 7, Player::You, Piece::King);
        board.set_piece(5, 7, Player::You, Piece::Bishop);
        board.set_piece(6, 7, Player::You, Piece::Knight);
        board.set_piece(7, 7, Player::You, Piece::Rook);

        board.set_piece(0, 6, Player::You, Piece::Pawn);
        board.set_piece(1, 6, Player::You, Piece::Pawn);
        board.set_piece(2, 6, Player::You, Piece::Pawn);
        board.set_piece(3, 6, Player::You, Piece::Pawn);
        board.set_piece(4, 6, Player::You, Piece::Pawn);
        board.set_piece(5, 6, Player::You, Piece::Pawn);
        board.set_piece(6, 6, Player::You, Piece::Pawn);
        board.set_piece(7, 6, Player::You, Piece::Pawn);

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

        let (x, y) = self
            .promote_pos
            .expect("cannot promote as no outstanding promotion was detected");

        let instance = match self.get(x, y) {
            Some(i) => i.clone(),
            None => panic!("no piece to promote at {}/{}", x, y),
        };

        assert!(
            matches!(instance.piece, Piece::Pawn),
            "can only promote pawns but piece ({}/{}) was '{:?}'",
            x,
            y,
            instance.piece
        );

        self.set(
            x as usize,
            y as usize,
            Some(PieceInstance::new(instance.player, promote_to)),
        );

        self.promote_pos = None
    }

    fn remove_moves_that_result_in_check(&self, x: i8, y: i8, info_board: &mut InfoBoard) {
        let piece = self.get(x, y).as_ref().unwrap();

        for (move_x, move_y) in self.iter_over_positions() {
            let new_pos_info = match info_board.get(move_x, move_y) {
                PosInfo::Move => PosInfo::None,
                PosInfo::PieceHit(ins) => PosInfo::Piece(ins.clone()),
                _ => continue,
            };

            let mut next_move = self.clone();
            next_move.set(x as usize, y as usize, None);
            next_move.set(move_x as usize, move_y as usize, Some(piece.clone()));

            if next_move.is_king_in_check(&piece.player) {
                info_board.set(move_x, move_y, new_pos_info);
            }
        }
    }

    fn remove_old_en_passant(&mut self) {
        for (x, y) in self.iter_over_positions() {
            if let Some(ins) = self.get_mut(x, y) {
                ins.is_eligible_for_en_passant = false;
            }
        }
    }

    fn remove_piece_passed_by_en_passant(&mut self, to_x: i8, to_y: i8, ins: &PieceInstance) {
        self.set(
            to_x as usize,
            (to_y - Piece::get_pawn_direction(ins)) as usize,
            None,
        );
    }

    /// "Low-level" set function, that simply overrides a position with the given
    /// value.
    ///
    /// The passed values are **not** checked for validity, e.g. if they are in
    /// the boards bounds. That burden is on the caller of this function.
    // TODO: change type of x and y to i8
    pub fn set(&mut self, x: usize, y: usize, instance: Option<PieceInstance>) {
        self.board[x][y] = instance;
    }

    /// Adds a new piece to the board.
    ///
    /// Creates and adds a completely new piece to the board. New pieces can only
    /// be added to empty positions.
    // TODO: remove method, just move `set` instead. Actually setting a piece should
    // not be common anyway.
    // The todo of `set` should be done first though.
    pub fn set_piece(&mut self, x: i8, y: i8, player: Player, piece: Piece) {
        assert!(
            self.is_in_bounds(x, y),
            "cannot set piece at out of bounds position ({}/{})",
            x,
            y
        );

        let pos = self.get(x, y);

        if pos.is_some() {
            panic!(
                "pieces can only be set on empty positions, but position {}/{} was {:?}",
                x, y, pos
            );
        }

        let instance = PieceInstance::new(player, piece);

        self.set(x as usize, y as usize, Some(instance));
    }

    /// Updates the piece selection.
    ///
    /// After a piece has been selected, certain things might be performed like moving
    /// it to another spot on the board.
    ///
    /// Rules:
    /// - Selecting any piece will result in the selection to be updated.
    /// - Selecting an empty square will remove the selection.
    /// - Selecting the current selected piece will remove the selection.
    // TODO: replace with simple `get_selected` and `set_selected`.
    pub fn update_selected(&mut self, x: i8, y: i8) {
        assert!(
            self.is_in_bounds(x, y),
            "cannot select piece outside of board ({}/{})",
            x,
            y
        );

        if self.get(x, y).is_none() {
            self.selected_pos = None;
            return;
        }

        if let Some(previous_selection) = self.selected_pos {
            if previous_selection == (x, y) {
                self.selected_pos = None;
            } else {
                self.selected_pos = Some((x, y));
            }
        } else {
            self.selected_pos = Some((x, y));
        }
    }

    pub fn width(&self) -> i8 {
        return self.width;
    }
}

pub struct PosInter {
    height: i8,
    width: i8,
    x: i8,
    y: i8,
}

impl Iterator for PosInter {
    type Item = (i8, i8);

    fn next(&mut self) -> Option<Self::Item> {
        if self.x < self.width {
            let next = (self.x, self.y);

            self.x += 1;

            return Some(next);
        }

        if self.y < self.height - 1 {
            self.x = 0;
            self.y += 1;

            let next = (0, self.y);

            return Some(next);
        }

        None
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const RESET: &str = "\u{001b}[0m";

        let mut val = "\n".to_owned();

        for y in 0..self.height {
            for x in 0..self.height {
                let piece_ins = self.get(x, y).as_ref();
                let bg_color = self.get_display_square_bg_color(x, y);
                let fg_color = self.get_display_fg_color_for_piece_instance(piece_ins);
                let piece_symbol = self.get_display_symbol_for_piece_instance(piece_ins);

                val.push_str(&format!(
                    "{}{} {} {}",
                    fg_color, bg_color, piece_symbol, RESET
                ));
            }

            val.push('\n');
        }

        write!(f, "{}", val)
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
    fn your_king_is_not_in_check() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));

        assert!(!board.is_king_in_check(&Player::You));
    }

    #[test]
    fn your_king_is_in_check() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(1, 6, ins(Player::Opponent, Piece::Pawn));

        assert!(board.is_king_in_check(&Player::You));
    }

    #[test]
    fn your_king_is_not_in_check_but_opponent_is() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 0, ins(Player::You, Piece::King));
        board.set(1, 1, ins(Player::You, Piece::Pawn));
        board.set(2, 0, ins(Player::Opponent, Piece::King));

        assert!(!board.is_king_in_check(&Player::You));
        assert!(board.is_king_in_check(&Player::Opponent));
    }

    #[test]
    fn moves_that_result_in_a_piece_bing_taken_are_added() {
        let mut pawn = PieceInstance::new(Player::You, Piece::Pawn);
        pawn.was_moved = false;

        let mut board = Board::new(Color::Black, Color::White);
        board.set(1, 6, Some(pawn.clone()));
        board.set(0, 0, ins(Player::Opponent, Piece::King));
        board.set(5, 1, ins(Player::Opponent, Piece::Bishop));

        board.update_selected(5, 1);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(1, 5), PosInfo::Move));
    }

    #[test]
    fn moves_that_would_result_in_a_check_are_not_added() {
        let mut board = Board::new(Color::Black, Color::White);

        board.set(1, 6, ins(Player::You, Piece::King));
        board.set(2, 5, ins(Player::You, Piece::Rook));

        board.set(3, 4, ins(Player::Opponent, Piece::Bishop));
        board.set(2, 7, ins(Player::Opponent, Piece::Rook));

        board.update_selected(1, 6);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(0, 7), PosInfo::None));
        assert!(matches!(info_board.get(1, 7), PosInfo::None));

        board.update_selected(2, 5);

        let info_board = board.get_moves_of_selected();

        assert!(matches!(info_board.get(2, 4), PosInfo::None));
        assert!(
            matches!(info_board.get(2, 7), PosInfo::Piece(_)),
            "{:?}",
            info_board.get(2, 7)
        );
    }

    #[test]
    fn en_passant_check_is_not_done_if_it_is_outside_the_board() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(0, 6, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(1, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(0, 6);
        assert!(board.move_selected_to(0, 4));
    }

    #[test]
    fn en_passant_your_east_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(2, 3, ins(Player::You, Piece::Pawn));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(
            3,
            1,
            Some(PieceInstance::new(Player::Opponent, Piece::Pawn)),
        );

        board.update_selected(3, 1);
        board.move_selected_to(3, 3);

        board.update_selected(2, 3);
        assert!(
            !board.move_selected_to(1, 2),
            "en passant was added to the wrong side"
        );
        assert!(
            board.move_selected_to(3, 2),
            "east en passant move was not accepted"
        );
        assert!(
            matches!(board.get(3, 3), None),
            "east en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_your_west_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(4, 3, ins(Player::You, Piece::Pawn));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(
            3,
            1,
            Some(PieceInstance::new(Player::Opponent, Piece::Pawn)),
        );

        board.update_selected(3, 1);
        board.move_selected_to(3, 3);

        board.update_selected(4, 3);
        assert!(
            !board.move_selected_to(5, 2),
            "west en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(3, 2),
            "west en passant move was not accepted"
        );
        assert!(
            matches!(board.get(3, 3), None),
            "west en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_opponent_east_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(3, 6, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(2, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(3, 6);
        board.move_selected_to(3, 4);

        board.update_selected(2, 4);
        assert!(
            !board.move_selected_to(1, 5),
            "east en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(3, 5),
            "east en passant move was not accepted"
        );
        assert!(
            matches!(board.get(3, 4), None),
            "east en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_opponent_west_pawn() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(3, 6, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(4, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(3, 6);
        board.move_selected_to(3, 4);

        board.update_selected(4, 4);
        assert!(
            !board.move_selected_to(5, 5),
            "west en passant was added to the wrong side",
        );
        assert!(
            board.move_selected_to(3, 5),
            "west en passant move was not accepted"
        );
        assert!(
            matches!(board.get(3, 4), None),
            "west en passant pawn was not removed"
        );
    }

    #[test]
    fn en_passant_is_only_added_when_piece_is_eligible() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(3, 5, ins(Player::You, Piece::Pawn));
        board.set(5, 6, ins(Player::You, Piece::Rook));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(4, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(3, 5);
        board.move_selected_to(3, 4);

        board.update_selected(4, 4);
        assert!(
            !board.move_selected_to(3, 5),
            "en passant was added even though the enemy only moved one square"
        );

        board.update_selected(5, 6);
        board.move_selected_to(5, 4);

        board.update_selected(4, 4);
        assert!(
            !board.move_selected_to(5, 5),
            "en passant was added event though moved piece wasn't a pawn"
        );
    }

    #[test]
    fn en_passant_can_only_be_done_immediately_after_the_opponent_piece_was_moved() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(3, 6, Some(PieceInstance::new(Player::You, Piece::Pawn)));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(4, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(3, 6);
        board.move_selected_to(3, 4);

        board.update_selected(7, 0);
        board.move_selected_to(7, 1);

        board.update_selected(4, 4);
        assert!(!board.move_selected_to(3, 5));
    }

    #[test]
    fn en_passant_pieces_are_only_taken_if_the_move_actually_was_one() {
        let mut board = Board::new(Color::Black, Color::White);
        board.set(0, 7, ins(Player::You, Piece::King));
        board.set(1, 6, ins(Player::You, Piece::Bishop));
        board.set(7, 0, ins(Player::Opponent, Piece::King));
        board.set(4, 4, ins(Player::Opponent, Piece::Pawn));

        board.update_selected(1, 6);
        board.move_selected_to(4, 3);

        assert!(matches!(board.get(4, 4), Some(_)));
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
