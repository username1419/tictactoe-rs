use crate::logutil::LogStatus;
use crossterm::style::{StyledContent, Stylize};
use crossterm::{ExecutableCommand, cursor, terminal};
use std::io::Stdout;

use crate::logutil::log;
use crate::{ApplicationState, ResultMenu};
// (x, y) = (ROW, COL)

#[derive(Clone)]
pub struct Game {
    board: Box<[Player]>,
    size: u8,
    turn: Player,
    cursor: (u8, u8),
    confirm: bool,
    is_ai: bool,
    can_player_select: bool,
}

impl Game {
    pub fn new(size: u8, is_ai: bool) -> Game {
        let mut board = vec![Player::None; size.pow(2) as usize].into_boxed_slice();
        board[0] = Player::None;
        Game {
            board,
            size,
            turn: Player::X,
            cursor: (0, 0),
            confirm: false,
            is_ai,
            can_player_select: true,
        }
    }

    pub fn draw(&self, stdout: &mut Stdout) {
        let (w, h) = terminal::size().expect("Failed to get terminal size");
        let mut row_index: u8 = 0;
        for row in self.board.chunks(self.size as usize) {
            let content = row
                .iter()
                .enumerate()
                .map(|(col, player)| {
                    let s = match player {
                        Player::X => "X".stylize(),
                        Player::O => "O".stylize(),
                        Player::None => " ".stylize(),
                    };
                    if self.cursor == (col as u8, (row_index) as u8) {
                        s.on_white().black()
                    } else {
                        s
                    }
                })
                .collect::<Vec<StyledContent<&str>>>();
            stdout
                .execute(cursor::MoveTo(
                    w / 2 - content.len() as u16,
                    h / 2 + row_index as u16 - (self.size / 2) as u16,
                ))
                .ok();

            for cell in content {
                print!("[{}]", cell);
            }
            row_index += 1;
        }
    }

    pub fn update(mut self) -> ApplicationState {
        // FUCK
        if !self.confirm {
            return ApplicationState::Game(self);
        }
        self.confirm = false;

        let winner = self.try_get_winner();

        if winner.is_some() {
            ApplicationState::Result(ResultMenu::new(winner.unwrap()))
        } else if self.board.iter().all(|p| p.ne(&Player::None)) {
            ApplicationState::Result(ResultMenu::new(Player::None))
        } else {
            ApplicationState::Game(self)
        }
    }

    fn select_cell(&mut self, pos: (u8, u8)) {
        let (x2, y2) = pos;

        if (y2 * self.size + x2) >= self.size.pow(2) {
            return;
        }

        self.cursor = pos;
    }

    #[allow(non_snake_case)]
    pub fn handleButtonLeft(&mut self) {
        log(LogStatus::DEBUG, "Left button handled in game screen");
        if self.cursor.0 == 0 {
            return;
        }
        self.select_cell((self.cursor.0 - 1, self.cursor.1));
    }

    #[allow(non_snake_case)]
    pub fn handleButtonRight(&mut self) {
        log(LogStatus::DEBUG, "Right button handled in game screen");
        if self.cursor.0 == self.size - 1 {
            return;
        }
        self.select_cell((self.cursor.0 + 1, self.cursor.1));
    }

    #[allow(non_snake_case)]
    pub fn handleButtonDown(&mut self) {
        log(LogStatus::DEBUG, "Down button handled in game screen");
        if self.cursor.1 == self.size - 1 {
            return;
        }
        self.select_cell((self.cursor.0, self.cursor.1 + 1));
    }

    #[allow(non_snake_case)]
    pub fn handleButtonUp(&mut self) {
        log(LogStatus::DEBUG, "Up button handled in game screen");
        if self.cursor.1 == 0 {
            return;
        }
        self.select_cell((self.cursor.0, self.cursor.1 - 1));
    }

    #[allow(non_snake_case)]
    pub fn handleConfirm(&mut self) {
        log(LogStatus::DEBUG, "Confirmation handled in game screen");
        let (x, y) = self.cursor;
        match self.board[(y * self.size + x) as usize] {
            Player::None => {
                self.board[(y * self.size + x) as usize] = self.turn.clone();
                self.turn = match self.turn {
                    Player::X => Player::O,
                    Player::O => Player::X,
                    _ => unreachable!(),
                };
                self.confirm = true;
                log(
                    LogStatus::DEBUG,
                    format!("Current board: {:?}", self.board).as_str(),
                );
            }
            _ => return,
        }
    }

    fn count_matching_diagonals(&self) -> (u8, u8, Player, Player) {
        if self.size & 1 == 0 {
            return (0, 0, Player::None, Player::None);
        }

        let target_lr_diag = self.board[0].clone();
        let target_rl_diag = self.board[(self.size - 1) as usize].clone();

        let mut count_lr_diag = 0;
        let mut count_rl_diag = 0;
        for y in 0..self.size {
            let lr_diag_cell = (y * (self.size + 1)) as usize;
            let rl_diag_cell = ((y + 1) * (self.size - 1)) as usize;

            if self.board[lr_diag_cell] == target_lr_diag && target_lr_diag != Player::None {
                count_lr_diag += 1;
            }
            if self.board[rl_diag_cell] == target_rl_diag && target_rl_diag != Player::None {
                count_rl_diag += 1;
            }
        }

        (count_lr_diag, count_rl_diag, target_lr_diag, target_rl_diag)
    }

    fn count_matching_rows_cols(&self) -> (u8, u8, Player) {
        let (sel_row, sel_col) = self.cursor;
        let target: Player = self.board[(sel_col * self.size + sel_row) as usize].clone();

        let mut count_col: u8 = 0;
        let mut count_row: u8 = 0;
        log(
            LogStatus::DEBUG,
            format!("Counting rows and col cells; cursor cell {:?}", self.cursor).as_str(),
        );
        log(
            LogStatus::DEBUG,
            format!("Selected count target {:?} for row", target).as_str(),
        );
        for cell in 0..self.size {
            let current_row = (sel_col * self.size + cell) as usize;
            let current_col = (cell * self.size + sel_row) as usize;
            log(
                LogStatus::DEBUG,
                format!(
                    "Current same-column cell {current_col} value {:?}, Current same-row cell {current_row} value {:?}",
                    self.board[current_col],
                    self.board[current_row],
                )
                .as_str(),
            );

            if self.board[current_row] == target && target != Player::None {
                count_row += 1;
            }
            if self.board[current_col] == target && target != Player::None {
                count_col += 1;
            }
        }

        (count_row, count_col, target)
    }

    fn try_get_winner(&self) -> Option<Player> {
        let (count_row, count_col, target) = self.count_matching_rows_cols();

        let (count_lr_diag, count_rl_diag, target_lr_diag, target_rl_diag) =
            self.count_matching_diagonals();

        let is_match_all = count_row == self.size
            || count_col == self.size
            || count_lr_diag == self.size
            || count_rl_diag == self.size;

        if is_match_all {
            match vec![count_row, count_col, count_rl_diag, count_lr_diag]
                .iter()
                .max()
                .unwrap()
            {
                cr if cr == &count_row => target,
                cc if cc == &count_col => target,
                clr if clr == &count_lr_diag => target_lr_diag,
                crl if crl == &count_rl_diag => target_rl_diag,
                _ => unreachable!(),
            };
        }
        None
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Player {
    X,
    O,
    None,
}

#[cfg(test)]
mod test {
    use crate::game::Player;

    use super::Game;
    use super::Player::*;

    #[test]
    fn diagonals() {
        let r1 = Game {
            cursor: (0, 0),
            board: vec![X, O, None, O, X, None, None, O, X].into_boxed_slice(),
            turn: O,
            size: 3,
            confirm: true,
            is_ai: false,
            can_player_select: true,
        }
        .count_matching_diagonals();

        assert_eq!(r1, (3, 0, Player::X, Player::None));

        let r2 = Game {
            cursor: (0, 0),
            board: vec![O, O, None, O, X, None, None, O, X].into_boxed_slice(),
            turn: X,
            size: 3,
            confirm: true,
            is_ai: false,
            can_player_select: true,
        }
        .count_matching_diagonals();

        assert_eq!(r2, (1, 0, Player::O, Player::None));

        let r3 = Game {
            cursor: (0, 0),
            board: vec![X, O, None, O, X, None, None, O, O].into_boxed_slice(),
            turn: O,
            size: 3,
            confirm: true,
            is_ai: false,
            can_player_select: true,
        }
        .count_matching_diagonals();

        assert_eq!(r3, (2, 0, Player::X, Player::None));
    }

    fn create_game(board: Vec<Player>, size: u8, cursor: (u8, u8)) -> Game {
        Game {
            board: board.into_boxed_slice(),
            size,
            turn: Player::X,
            cursor,
            confirm: false,
            is_ai: false,
            can_player_select: true,
        }
    }

    #[test]
    fn test_empty_board() {
        let size = 3;
        let board = vec![Player::None; (size * size) as usize];
        let game = create_game(board, size, (0, 0));

        let (count_row, count_col, target) = game.count_matching_rows_cols();
        assert_eq!(count_row, 0);
        assert_eq!(count_col, 0);
        assert_eq!(target, Player::None);
    }

    #[test]
    fn test_full_row_match() {
        let size = 3;
        let board = vec![
            X, X, X, // row 0
            O, None, O, None, O, None,
        ];
        let game = create_game(board, size, (1, 0)); // cursor on first row, middle column

        let (count_row, count_col, target) = game.count_matching_rows_cols();
        assert_eq!(count_row, 3); // entire row is X
        assert_eq!(count_col, 1); // only one X in the column
        assert_eq!(target, Player::X);
    }

    #[test]
    fn test_full_col_match() {
        let size = 3;
        let board = vec![O, X, None, O, None, X, O, X, None];
        let game = create_game(board, size, (0, 2)); // cursor bottom-left

        let (count_row, count_col, target) = game.count_matching_rows_cols();
        assert_eq!(count_row, 1); // only one O in row 2
        assert_eq!(count_col, 3); // full column of O
        assert_eq!(target, Player::O);
    }

    #[test]
    fn test_mixed_board() {
        let size = 3;
        let board = vec![X, O, X, O, X, O, X, None, O];
        let game = create_game(board, size, (1, 1)); // cursor at middle

        let (count_row, count_col, target) = game.count_matching_rows_cols();
        assert_eq!(count_row, 1); // only middle cell matches Player::X
        assert_eq!(count_col, 1); // top and middle cells are X
        assert_eq!(target, Player::X);
    }
}
