use std::env::current_dir;
use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{clear, color, cursor, terminal_size};
use walkdir::WalkDir;

use crate::file::File;

enum Mode {
    Command,
    Switch,
}

pub struct Editor {
    mode: Mode,
    run: bool,
    path: String,
    hide: bool,
    list: Vec<String>,
    query: String,
    filter: Vec<String>,
    select: usize,
    files: Vec<File>,
    current: usize,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            mode: Mode::Command,
            run: true,
            path: String::new(),
            hide: true,
            list: vec![],
            query: String::new(),
            filter: vec![],
            select: 0,
            files: vec![],
            current: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let mut term = AlternateScreen::from(stdout().into_raw_mode()?);
        let mut keys = stdin().keys();
        while self.run {
            self.clear(&mut term)?;
            self.render(&mut term)?;
            term.flush()?;
            if let Some(Ok(key)) = keys.next() {
                self.handle(key)?;
            }
        }
        Ok(())
    }

    fn render(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.render_command(term),
            Mode::Switch => self.render_switch(term),
        }
    }

    fn render_command(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        if let Some(file) = self.files.get(self.current) {
            file.render(term)?;
        }
        let (_columns, rows) = terminal_size()?;
        write!(
            term,
            "{}{}{}{}",
            cursor::Goto(1, rows),
            color::Bg(color::Green),
            clear::UntilNewline,
            color::Bg(color::Reset),
        )?;
        Ok(())
    }

    fn render_switch(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        let (_columns, rows) = terminal_size()?;
        let mut row: u16 = 1;
        for file in &self.filter {
            if self.select + 1 == row.into() {
                write!(
                    term,
                    "{}{}{}{}{}",
                    cursor::Goto(1, row),
                    color::Bg(color::LightBlack),
                    file,
                    clear::UntilNewline,
                    color::Bg(color::Reset),
                )?;
            } else {
                write!(term, "{}{}", cursor::Goto(1, row), file)?;
            }
            row += 1;
            if row == rows {
                break;
            }
        }
        write!(
            term,
            "{}{}{} {}{}{}",
            cursor::Goto(1, rows),
            color::Bg(color::Blue),
            self.path,
            self.query,
            clear::UntilNewline,
            color::Bg(color::Reset),
        )?;
        Ok(())
    }

    fn clear(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        write!(
            term,
            "{}{}{}{}",
            color::Fg(color::Reset),
            color::Bg(color::Reset),
            clear::All,
            cursor::Goto(1, 1)
        )?;
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
            Key::Char('\t') => {
                self.mode = Mode::Switch;
                self.list()?;
            }
            Key::Char('B') => self.run = false,
            _ => {}
        }
        Ok(())
    }

    fn handle_switch(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match key {
            Key::Char('\t') => self.mode = Mode::Command,
            Key::BackTab => {
                self.hide = !self.hide;
                self.list()?
            }
            Key::Down => {
                if self.select + 1 < self.filter.len() {
                    self.select += 1;
                }
            }
            Key::Up => {
                if self.select > 0 {
                    self.select -= 1;
                }
            }
            Key::Backspace => {
                self.query.pop();
                self.filter();
            }
            Key::Char('\n') => {
                if let Some(path) = self.filter.get(self.select) {
                    let file = File::open(path)?;
                    self.files.push(file);
                    self.current = self.files.len() - 1;
                    self.mode = Mode::Command;
                }
            }
            Key::Char(c) => {
                self.query += &c.to_string();
                self.filter();
            }
            _ => {}
        }
        Ok(())
    }

    fn list(&mut self) -> Result<(), Box<dyn Error>> {
        self.path = current_dir()?.to_string_lossy().to_string();
        self.query = String::new();
        self.list.clear();
        for file in WalkDir::new(&self.path)
            .follow_links(true)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !self.hide || !name.starts_with("."))
                    .unwrap_or(true)
            })
            .filter_map(|file| file.ok())
        {
            if file.metadata()?.is_file() {
                let relative = file
                    .path()
                    .strip_prefix(&self.path)?
                    .to_string_lossy()
                    .to_string();
                self.list.push(relative);
            }
        }
        self.filter = self.list.clone();
        self.select = 0;
        Ok(())
    }

    fn filter(&mut self) {
        self.filter = self
            .list
            .iter()
            .filter(|file| file.contains(&self.query))
            .cloned()
            .collect();
        self.select = 0;
    }
}
