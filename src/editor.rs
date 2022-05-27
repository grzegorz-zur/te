use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use termion::clear;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};

enum Mode {
    Command,
    Switch,
}

pub struct Editor {
    mode: Mode,
    exit: bool,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            mode: Mode::Command,
            exit: false,
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
            self.handle(key)?;
            if self.exit {
                break;
            };
        }
        Ok(())
    }

    fn clear(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(term, "{}", clear::All)?;
        Ok(())
    }

    fn render(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.render_command(term),
            Mode::Switch => self.render_switch(term),
        }
    }

    fn render_command(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(term, "{}command", clear::All)?;
        Ok(())
    }

    fn render_switch(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(term, "{}switch", clear::All)?;
        Ok(())
    }

    fn handle(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.handle_command(key),
            Mode::Switch => self.handle_switch(key),
        }
    }

    fn handle_command(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match key {
            Key::Char('\t') => self.mode = Mode::Switch,
            Key::Char('B') => self.exit = true,
            _ => {}
        }
        Ok(())
    }

    fn handle_switch(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match key {
            Key::Char('\t') => self.mode = Mode::Command,
            _ => {}
        }
        Ok(())
    }
}
