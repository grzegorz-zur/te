use std::env::current_dir;
use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{clear, color, cursor, terminal_size};
use walkdir::WalkDir;

enum Mode {
    Command,
    Switch,
}
pub struct Editor {
    files: Files,
    mode: Mode,
    run: bool,
}

struct Files {
    path: String,
    hide: bool,
    query: String,
    list: Vec<String>,
    filter: Vec<String>,
    select: usize,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            files: Files {
                path: String::new(),
                hide: true,
                query: String::new(),
                list: vec![],
                filter: vec![],
                select: 0,
            },
            mode: Mode::Command,
            run: true,
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

    fn render_command(&self, _term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn render_switch(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
        let (_columns, rows) = terminal_size()?;
        let mut row: u16 = 1;
        for file in &self.files.filter {
            if self.files.select + 1 == row.into() {
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
            self.files.path,
            self.files.query,
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
                self.files_list()?;
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
                self.files.hide = !self.files.hide;
                self.files_list()?
            }
            Key::Down => {
                if self.files.select + 1 < self.files.filter.len() {
                    self.files.select += 1;
                }
            }
            Key::Up => {
                if self.files.select > 0 {
                    self.files.select -= 1;
                }
            }
            Key::Backspace => {
                self.files.query.pop();
                self.files_filter()?;
            }
            Key::Char('\n') => {}
            Key::Char(c) => {
                self.files.query += &c.to_string();
                self.files_filter()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn files_list(&mut self) -> Result<(), Box<dyn Error>> {
        self.files.path = current_dir()?.to_string_lossy().to_string();
        self.files.query = String::new();
        self.files.list.clear();
        for file in WalkDir::new(&self.files.path)
            .follow_links(true)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !self.files.hide || !name.starts_with("."))
                    .unwrap_or(true)
            })
            .filter_map(|file| file.ok())
        {
            if file.metadata()?.is_file() {
                let relative = file
                    .path()
                    .strip_prefix(&self.files.path)?
                    .to_string_lossy()
                    .to_string();
                self.files.list.push(relative);
            }
        }
        self.files.filter = self.files.list.clone();
        self.files.select = 0;
        Ok(())
    }

    fn files_filter(&mut self) -> Result<(), Box<dyn Error>> {
        self.files.filter = self
            .files
            .list
            .iter()
            .filter(|file| file.contains(&self.files.query))
            .cloned()
            .collect();
        self.files.select = 0;
        Ok(())
    }
}
