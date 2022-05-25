use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

enum Mode {
    Command,
}

enum Action {
    None,
    Exit,
}

pub struct Editor {
    mode: Mode,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            mode: Mode::Command,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let stdin = stdin();
        let mut stdout = stdout().into_raw_mode()?;
        self.clear(&mut stdout)?;
        for res in stdin.keys() {
            let key = res?;
            self.render(&mut stdout)?;
            stdout.flush()?;
            let action = self.handle(key)?;
            match action {
                Action::Exit => break,
                Action::None => {}
            }
        }
        Ok(())
    }

    fn clear(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(term, "{}", clear::All)?;
        Ok(())
    }

    fn render(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(term, "render\r\n")?;
        Ok(())
    }

    fn handle(&self, key: Key) -> Result<Action, Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.command(key),
        }
    }

    fn command(&self, key: Key) -> Result<Action, Box<dyn Error>> {
        match key {
            Key::Char('B') => Ok(Action::Exit),
            _ => Ok(Action::None),
        }
    }
}
