use std::fmt::{self, Display};

use crate::{info_board, Color, InfoBoard, Piece, Player};

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

    pub fn display(&self) -> String {
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

        val
    }

    pub fn generate_info_board(&self) -> InfoBoard {
        let mut info_board = InfoBoard::new();

        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(instance) = self.get(x, y) {
                    info_board.set(x, y, info_board::PosInfo::Piece(instance.clone()));
                }
            }
        }

        if let Some((x, y)) = self.selected_pos {
            Piece::add_moves_for_piece_at_to_board(x, y, &mut info_board);
        }

        info_board
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
    pub fn move_selected_to(&mut self, x: i8, y: i8) -> bool {
        assert!(
            self.is_in_bounds(x, y),
            "cannot move piece outside the board (move to {}/{})",
            x,
            y,
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

        let mut instance_of_piece_to_move = if let Some(instance) = self.get(piece_x, piece_y) {
            instance.clone()
        } else {
            panic!("no piece found to move at position {}/{}", piece_x, piece_y);
        };

        let info_board = self.generate_info_board();

        let is_valid_move = matches!(
            info_board.get(x, y),
            info_board::PosInfo::Move | info_board::PosInfo::PieceHit(_)
        );

        if !is_valid_move {
            return false;
        }

        instance_of_piece_to_move.was_moved = true;

        let move_is_castle =
            matches!(instance_of_piece_to_move.piece, Piece::King) && (x - piece_x).abs() == 2;

        self.set(piece_x as usize, piece_y as usize, None);
        self.set(x as usize, y as usize, Some(instance_of_piece_to_move));

        self.selected_pos = None;

        self.check_if_pawn_needs_promoting(x, y);

        if move_is_castle {
            self.move_rook_for_castle(piece_x, x, piece_y);
        }

        true
    }

    pub fn new(you_color: Color, opponent_color: Color) -> Self {
        let width = 8;
        let height = 8;

        let mut board = Self {
            board: vec![vec![None; width]; height],
            height: height as i8,
            opponent_color,
            promote_pos: None,
            selected_pos: None,
            width: width as i8,
            you_color,
        };

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

    /// "Low-level" set function, that simply overrides a position with the given
    /// value.
    ///
    /// The passed values are **not** checked for validity, e.g. if they are in
    /// the boards bounds. That burden is on the caller of this function.
    fn set(&mut self, x: usize, y: usize, instance: Option<PieceInstance>) {
        self.board[x][y] = instance;
    }

    /// Adds a new piece to the board.
    ///
    /// Creates and adds a completely new piece to the board. New pieces can only
    /// be added to empty positions.
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

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display())
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
