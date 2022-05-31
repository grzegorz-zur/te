use std::env::current_dir;
use std::error::Error;
use std::io::{stdin, stdout, Stdout, Write};
use std::path::PathBuf;
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
    path: PathBuf,
    list: Vec<PathBuf>,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            files: Files {
                path: PathBuf::new(),
                list: vec![],
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
        let mut row = 1;
        for file in &self.files.list {
            write!(term, "{}{}", cursor::Goto(1, row), file.display())?;
            row += 1;
            if row == rows {
                break;
            }
        }
        write!(
            term,
            "{}{}{}{}",
            cursor::Goto(1, rows),
            color::Bg(color::Blue),
            self.files.path.display(),
            clear::AfterCursor
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
                self.list_files()?;
            }
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

    fn list_files(&mut self) -> Result<(), Box<dyn Error>> {
        self.files.path = current_dir()?;
        self.files.list.clear();
        for file in WalkDir::new(self.files.path.as_path())
            .follow_links(true)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|file| file.ok())
        {
            if file.metadata()?.is_file() {
                let relative = file.path().strip_prefix(self.files.path.as_path())?;
                self.files.list.push(relative.to_path_buf());
            }
        }
        Ok(())
    }
}
