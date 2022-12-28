use crate::coords::*;
use crate::file::*;
use crossterm::cursor::*;
use crossterm::event::*;
use crossterm::style::*;
use crossterm::terminal::*;
use crossterm::{execute, queue};
use libc;
use signal_hook::consts::*;
use signal_hook::flag::register;
use signal_hook::low_level::raise;
use std::env::current_dir;
use std::error::Error;
use std::io::{stdout, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use walkdir::WalkDir;

const TIMEOUT: Duration = Duration::from_millis(200);

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
    view: Vec<String>,
    offset: Position,
    position: Position,
    files: Vec<File>,
    current: usize,
}

impl Editor {
    pub fn create() -> Editor {
        Editor {
            run: true,
            mode: Mode::Command,
            path: String::new(),
            hide: true,
            list: vec![],
            query: String::new(),
            view: vec![],
            offset: Position::start(),
            position: Position::start(),
            files: vec![],
            current: 0,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let terminate = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let unpause = Arc::new(AtomicBool::new(false));

        register(libc::SIGINT, terminate.clone())?;
        register(libc::SIGTERM, terminate.clone())?;
        register(libc::SIGQUIT, terminate.clone())?;
        register(libc::SIGTSTP, pause.clone())?;
        register(libc::SIGCONT, unpause.clone())?;

        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;

        let mut result: Result<(), Box<dyn Error>> = Ok(());
        let mut display = true;

        while self.run && result.is_ok() {
            result = self.step(
                &mut display,
                terminate.clone(),
                pause.clone(),
                unpause.clone(),
            );
        }

        execute!(stdout(), LeaveAlternateScreen)?;
        disable_raw_mode()?;

        result
    }

    fn step(
        &mut self,
        display: &mut bool,
        terminate: Arc<AtomicBool>,
        pause: Arc<AtomicBool>,
        unpause: Arc<AtomicBool>,
    ) -> Result<(), Box<dyn Error>> {
        if terminate.load(Ordering::Relaxed) {
            self.stop()?;
        }
        if pause.load(Ordering::Relaxed) {
            self.pause()?;
        }
        if unpause.load(Ordering::Relaxed) {
            self.unpause()?;
            *display = true;
        };
        if let Some(file) = self.files.get_mut(self.current) {
            *display |= file.refresh()?;
        }
        if *display {
            self.display()?;
            *display = false
        }
        if poll(TIMEOUT)? {
            if let Event::Key(key) = read()? {
                self.handle_input(key)?;
                *display = true;
            }
        }
        Ok(())
    }

    fn display(&mut self) -> Result<(), Box<dyn Error>> {
        let size = size()?.into();
        queue!(stdout(), ResetColor, MoveTo(0, 0), Clear(ClearType::All))?;
        match self.mode {
            Mode::Command => self.display_command(size)?,
            Mode::Switch => self.display_switch(size)?,
        };
        stdout().flush()?;
        Ok(())
    }

    fn display_command(&mut self, size: Size) -> Result<(), Box<dyn Error>> {
        let (_columns, rows) = size.try_into()?;
        match self.files.get_mut(self.current) {
            Some(file) => {
                let (position, relative) = file.display(Size {
                    lines: size.lines - 1,
                    columns: size.columns,
                })?;
                queue!(
                    stdout(),
                    MoveTo(0, rows - 1),
                    SetBackgroundColor(Color::Green),
                    Clear(ClearType::CurrentLine),
                    Print(format!(
                        "{} {}:{}",
                        file.path,
                        position.line + 1,
                        position.column + 1
                    )),
                )?;
                if let Ok((column, row)) = relative.try_into() {
                    queue!(stdout(), MoveTo(column - 1, row - 1))?;
                }
            }
            None => {
                queue!(
                    stdout(),
                    MoveTo(0, rows - 1),
                    SetBackgroundColor(Color::Green),
                    Clear(ClearType::CurrentLine),
                )?;
            }
        }
        Ok(())
    }

    fn display_switch(&mut self, size: Size) -> Result<(), Box<dyn Error>> {
        self.offset = self.offset.shift(
            self.position,
            Size {
                lines: size.lines - 1,
                columns: size.columns,
            },
        );
        self.view
            .iter()
            .skip(self.offset.line)
            .take(size.lines - 1)
            .try_for_each(|file| queue!(stdout(), Print(file), MoveToNextLine(1)))?;
        if let Some(path) = self.view.get(self.position.line) {
            queue!(
                stdout(),
                MoveTo(
                    0,
                    (self.position.line - self.offset.line).try_into().unwrap()
                ),
                SetBackgroundColor(Color::DarkGrey),
                Clear(ClearType::CurrentLine),
                Print(path),
            )?;
        }
        queue!(
            stdout(),
            MoveTo(0, (size.lines - 1).try_into().unwrap()),
            SetBackgroundColor(Color::Blue),
            Clear(ClearType::CurrentLine),
            Print(format!("{} {}", self.path, self.query)),
        )?;
        Ok(())
    }

    fn handle_input(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.handle_command(key),
            Mode::Switch => self.handle_switch(key),
        }
    }

    fn handle_command(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Tab => self.switch()?,
            KeyCode::Char('c') => self.previous(),
            KeyCode::Char('v') => self.next(),
            KeyCode::Char('b') => self.pause()?,
            KeyCode::Char('B') => self.stop()?,
            KeyCode::Char('n') => self.close()?,
            KeyCode::Char('m') => self.write()?,
            _ => {}
        }
        if let Some(file) = self.files.get_mut(self.current) {
            match key.code {
                KeyCode::Char('a') => file.goto(file.position.up()),
                KeyCode::Char('A') => file.goto(Position::start()),
                KeyCode::Char('s') => file.goto(file.position.down()),
                KeyCode::Char('S') => file.goto(Position::end()),
                KeyCode::Char('d') => file.goto(file.position.left()),
                KeyCode::Char('D') => file.goto(file.position.line_start()),
                KeyCode::Char('f') => file.goto(file.position.right()),
                KeyCode::Char('F') => file.goto(file.position.line_end()),
                KeyCode::Char('N') => file.read()?,
                KeyCode::Char('M') => file.write()?,
                KeyCode::Up => file.goto(file.position.up()),
                KeyCode::Down => file.goto(file.position.down()),
                KeyCode::Left => file.goto(file.position.left()),
                KeyCode::Right => file.goto(file.position.right()),
                _ => {}
            }
        }
        Ok(())
    }

    fn previous(&mut self) {
        let length = self.files.len();
        if length == 0 {
            self.current = 0;
            return;
        }
        if self.current == 0 {
            self.current = length - 1;
        } else {
            self.current -= 1;
        }
    }

    fn next(&mut self) {
        let length = self.files.len();
        if length == 0 {
            self.current = 0;
            return;
        }
        if self.current + 1 == length {
            self.current = 0;
        } else {
            self.current += 1;
        }
    }

    fn handle_switch(&mut self, key: KeyEvent) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Tab => self.command(),
            KeyCode::BackTab => {
                self.hide = !self.hide;
                self.list()?;
            }
            KeyCode::Down => {
                if self.position.line + 1 < self.view.len() {
                    self.position.line += 1;
                }
            }
            KeyCode::Up => {
                if self.position.line > 0 {
                    self.position.line -= 1;
                }
            }
            KeyCode::Backspace => {
                self.query.pop();
                self.filter();
            }
            KeyCode::Enter => self.select()?,
            KeyCode::Char(c) => {
                self.query.push(c);
                self.filter()
            }
            _ => {}
        }
        Ok(())
    }

    fn command(&mut self) {
        self.mode = Mode::Command;
    }

    fn switch(&mut self) -> Result<(), Box<dyn Error>> {
        self.mode = Mode::Switch;
        self.list()?;
        Ok(())
    }

    fn pause(&mut self) -> Result<(), Box<dyn Error>> {
        execute!(stdout(), ResetColor, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        raise(SIGSTOP)?;
        Ok(())
    }

    fn unpause(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        execute!(stdout(), ResetColor, EnterAlternateScreen)?;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        self.write()?;
        self.run = false;
        Ok(())
    }

    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(path) = self.view.get(self.position.line).map(String::clone) {
            self.open(&path)?;
        } else {
            let path = self.query.to_string();
            self.open(&path)?;
        }
        Ok(())
    }

    pub fn open(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = File::open(path)?;
        self.files.push(file);
        self.current = self.files.len() - 1;
        self.mode = Mode::Command;
        Ok(())
    }

    pub fn write(&mut self) -> Result<(), Box<dyn Error>> {
        for file in self.files.iter_mut() {
            file.write()?;
        }
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(file) = self.files.get_mut(self.current) {
            file.write()?;
            self.files.remove(self.current);
            self.previous();
        }
        Ok(())
    }

    fn list(&mut self) -> Result<(), Box<dyn Error>> {
        self.path = current_dir()?.to_string_lossy().to_string();
        self.list = WalkDir::new(&self.path)
            .follow_links(true)
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !self.hide || !name.starts_with('.'))
                    .unwrap_or(true)
            })
            .filter_map(|file| file.ok())
            .filter_map(|file| {
                if file.metadata().ok()?.is_file() {
                    Some(file)
                } else {
                    None
                }
            })
            .filter_map(|file| {
                Some(
                    file.path()
                        .strip_prefix(&self.path)
                        .ok()?
                        .to_string_lossy()
                        .to_string(),
                )
            })
            .collect();
        self.query = String::new();
        self.view = self.list.clone();
        self.position = Position::start();
        Ok(())
    }

    fn filter(&mut self) {
        self.view = self
            .list
            .iter()
            .filter(|file| file.contains(&self.query))
            .cloned()
            .collect();
        self.position = Position::start();
    }
}
