use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{clear, cursor, terminal_size};

enum Mode {
    Command,
    Switch,
}

pub struct Editor {
    mode: Mode,
    run: bool,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            mode: Mode::Command,
            run: true,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut term = AlternateScreen::from(stdout().into_raw_mode()?);
        let mut keys = stdin().keys();
        while self.run {
            write!(term, "{}", clear::All)?;
            let (columns, rows) = terminal_size()?;
            self.render(&mut term, columns, rows)?;
            term.flush()?;
            if let Some(Ok(key)) = keys.next() {
                self.handle(key)?;
            }
        }
        Ok(())
    }

    fn render(
        &self,
        term: &mut RawTerminal<Stdout>,
        columns: u16,
        rows: u16,
    ) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.render_command(term, columns, rows),
            Mode::Switch => self.render_switch(term, columns, rows),
        }
    }

    fn render_command(
        &self,
        term: &mut RawTerminal<Stdout>,
        _columns: u16,
        _rows: u16,
    ) -> Result<(), Box<dyn Error>> {
        write!(term, "{}command", cursor::Goto(1, 1))?;
        Ok(())
    }

    fn render_switch(
        &self,
        term: &mut RawTerminal<Stdout>,
        _columns: u16,
        _rows: u16,
    ) -> Result<(), Box<dyn Error>> {
        write!(term, "{}switch", cursor::Goto(1, 1))?;
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
            Key::Char('B') => self.run = false,
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
