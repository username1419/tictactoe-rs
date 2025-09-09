mod game;
mod logutil;

use crossterm::event::{self, *};
use crossterm::style::{ContentStyle, PrintStyledContent, StyledContent, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::{ExecutableCommand, cursor};
use game::*;
use std::io::{Stdout, Write, stdout};
use std::panic::set_hook;
use std::process::exit;
use std::str::FromStr;

use self::logutil::log;

#[derive(Clone)]
enum ApplicationState {
    Menu(ApplicationMenu),
    Game(Game),
    Result(ResultMenu),
}

impl ApplicationState {
    fn draw(&self, stdout: &mut Stdout) {
        match self {
            ApplicationState::Menu(menu) => menu.draw(stdout),
            ApplicationState::Result(result) => result.draw(stdout),
            ApplicationState::Game(game) => game.draw(stdout),
        }
    }

    #[allow(non_snake_case)]
    fn handleButtonLeft(&mut self) {
        match self {
            ApplicationState::Game(game) => game.handleButtonLeft(),
            _ => {}
        };
    }

    #[allow(non_snake_case)]
    fn handleButtonRight(&mut self) {
        match self {
            ApplicationState::Game(game) => game.handleButtonRight(),
            _ => {}
        };
    }

    #[allow(non_snake_case)]
    fn handleButtonDown(&mut self) {
        match self {
            ApplicationState::Game(game) => game.handleButtonDown(),
            ApplicationState::Menu(menu) => menu.try_increment(),
            ApplicationState::Result(result) => result.try_increment(),
        };
    }

    #[allow(non_snake_case)]
    fn handleButtonUp(&mut self) {
        match self {
            ApplicationState::Game(game) => game.handleButtonUp(),
            ApplicationState::Menu(menu) => menu.try_decrement(),
            ApplicationState::Result(result) => result.try_decrement(),
        };
    }

    #[allow(non_snake_case)]
    fn handleConfirm(&mut self) {
        match self {
            ApplicationState::Menu(menu) => menu.confirmed = true,
            ApplicationState::Game(game) => game.handleConfirm(),
            ApplicationState::Result(result) => result.handleConfirm(),
        }
    }

    fn update(self) -> ApplicationState {
        match self {
            ApplicationState::Menu(menu) => menu.update(),
            ApplicationState::Game(game) => game.update(),
            ApplicationState::Result(result) => result.update(),
        }
    }
}

#[derive(Clone)]
struct ApplicationMenu {
    selected: usize,
    selection: Vec<(String, fn() -> ApplicationState)>,
    confirmed: bool,
}

impl ApplicationMenu {
    fn new() -> ApplicationMenu {
        ApplicationMenu {
            selected: 0,
            selection: vec![
                ("Play with AI".to_string(), || {
                    ApplicationState::Game(Game::new(3, true))
                }),
                ("Play locally".to_string(), || {
                    ApplicationState::Game(Game::new(3, false))
                }),
                ("Exit".to_string(), || {
                    stdout().execute(cursor::Show).ok();
                    disable_raw_mode().ok();
                    exit(0);
                }),
            ],
            confirmed: false,
        }
    }

    fn try_increment(&mut self) {
        let max_i = self.selection.len() - 1;
        if self.selected == max_i {
            return;
        };
        self.selected += 1;
    }

    fn try_decrement(&mut self) {
        if self.selected == 0 {
            return;
        };
        self.selected -= 1;
    }

    fn draw(&self, stdout: &mut Stdout) {
        let (w, h) = terminal::size().expect("Failed to retrieve size");
        for (index, (option, _)) in self.selection.iter().enumerate() {
            let content_length = option.len() as u16;
            let mut content = StyledContent::new(ContentStyle::new(), option);
            if index == self.selected {
                content = content.black().on_white();
            }
            stdout
                .execute(cursor::MoveTo(
                    w / 2 - content_length / 2,
                    h / 2 + index as u16,
                ))
                .expect("err")
                .execute(PrintStyledContent(content))
                .ok();
        }

        stdout.flush().ok();
    }

    fn update(self) -> ApplicationState {
        if !self.confirmed {
            return ApplicationState::Menu(self);
        }

        self.selection.get(self.selected).unwrap().1()
    }
}

#[derive(Clone)]
struct ResultMenu {
    win: Player,
    selection: Vec<String>,
    selected: usize,
    confirm: bool,
}

impl ResultMenu {
    fn new(win: Player) -> ResultMenu {
        ResultMenu {
            win,
            selection: vec![
                String::from_str("Return to Menu").unwrap(),
                String::from_str("Restart").unwrap(),
            ],
            selected: 0,
            confirm: false,
        }
    }

    fn draw(&self, stdout: &mut Stdout) {
        let (w, h) = terminal::size().expect("Failed to retrieve terminal size");

        let winner_text = match self.win {
            Player::X => "Winner: Player X",
            Player::O => "Winner: Player O",
            Player::None => "Draw",
        }
        .on_green();

        stdout
            .execute(cursor::MoveTo(
                w / 2 - winner_text.content().len() as u16 / 2,
                h / 2,
            ))
            .unwrap()
            .execute(PrintStyledContent(winner_text))
            .ok();
        for (index, content) in self.selection.iter().enumerate() {
            let content_length = content.len() as u16;
            stdout
                .execute(cursor::MoveTo(
                    w / 2 - content_length / 2,
                    h / 2 + index as u16 + 2,
                ))
                .expect("err")
                .execute(PrintStyledContent(if index == self.selected {
                    content.clone().negative()
                } else {
                    content.clone().stylize()
                }))
                .ok();
        }
    }

    fn update(&self) -> ApplicationState {
        if !self.confirm {
            return ApplicationState::Result(self.clone());
        }
        match self.selection[self.selected].as_str() {
            "Return to Menu" => ApplicationState::Menu(ApplicationMenu::new()),
            "Restart" => ApplicationState::Game(Game::new(3, false)),
            _ => unimplemented!(),
        }
    }

    fn try_increment(&mut self) {
        let max_i = self.selection.len() - 1;
        if self.selected == max_i {
            return;
        };
        self.selected += 1;
    }

    fn try_decrement(&mut self) {
        if self.selected == 0 {
            return;
        };
        self.selected -= 1;
    }

    #[allow(non_snake_case)]
    fn handleConfirm(&mut self) {
        self.confirm = true;
    }
}

#[allow(non_snake_case)]
fn main() {
    set_hook(Box::new(|p| {
        stdout().execute(cursor::Show).ok();
        disable_raw_mode().ok();

        log(
            logutil::LogStatus::FATAL,
            format!(
                "Paniced at line {} of {}, {}",
                p.location().unwrap().line(),
                p.location().unwrap().file(),
                p.payload().downcast_ref::<&str>().unwrap()
            )
            .as_str(),
        );
    }));

    let mut applicationState = ApplicationState::Menu(ApplicationMenu::new());
    let mut stdout = stdout();
    enable_raw_mode().ok();
    stdout
        .execute(cursor::Hide)
        .expect("err")
        .execute(terminal::Clear(terminal::ClearType::All))
        .ok();
    applicationState.draw(&mut stdout);

    loop {
        let KeyEvent {
            code,
            modifiers,
            kind: _,
            state: _,
        } = event::read()
            .expect("")
            .as_key_press_event()
            .unwrap_or(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE));

        match (code, modifiers) {
            (KeyCode::Up, KeyModifiers::NONE) | (KeyCode::Char('w'), KeyModifiers::NONE) => {
                applicationState.handleButtonUp()
            }
            (KeyCode::Down, KeyModifiers::NONE) | (KeyCode::Char('s'), KeyModifiers::NONE) => {
                applicationState.handleButtonDown()
            }
            (KeyCode::Right, KeyModifiers::NONE) | (KeyCode::Char('d'), KeyModifiers::NONE) => {
                applicationState.handleButtonRight()
            }
            (KeyCode::Left, KeyModifiers::NONE) | (KeyCode::Char('a'), KeyModifiers::NONE) => {
                applicationState.handleButtonLeft()
            }
            (KeyCode::Enter, KeyModifiers::NONE) => applicationState.handleConfirm(),
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                disable_raw_mode().ok();
                stdout.execute(cursor::Hide).ok();
                exit(0);
            }

            (_, _) => continue,
        }

        applicationState = applicationState.update();
        stdout
            .execute(terminal::Clear(terminal::ClearType::All))
            .ok(); // flush the screen
        applicationState.draw(&mut stdout);
        stdout.flush().ok(); // flush the buffer
    }
}
