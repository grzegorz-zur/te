use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use std::env::current_dir;
use std::error::Error;
use std::io::{stderr, stdin, stdout, Stdout, Write};
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::IntoAlternateScreen;
use termion::{clear, color, cursor, terminal_size};
use walkdir::WalkDir;

use crate::coords::*;
use crate::file::*;

type Signal = i32;

enum Mode {
    Command,
    Switch,
}

enum Message {
    Signal(Signal),
    Input(Key),
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
        let term = stdout().into_raw_mode()?;
        let mut screen = term.into_alternate_screen()?;
        let (sender, receiver) = mpsc::channel::<Message>();
        let sender_signal = sender.clone();
        let sender_input = sender.clone();
        thread::spawn(|| Self::signals(sender_signal));
        thread::spawn(|| Self::input(sender_input));
        while self.run {
            let size = terminal_size()?.into();
            self.display(&mut screen, size)?;
            screen.flush()?;
            let message = receiver.recv()?;
            match message {
                Message::Signal(signal) => self.handle_signal(signal)?,
                Message::Input(key) => self.handle_input(key)?,
            }
        }
        Ok(())
    }

    fn signals(sender: Sender<Message>) {
        let mut signals = Signals::new(&[SIGINT, SIGTERM, SIGQUIT]).unwrap();
        let handle = signals.handle();
        for signal in &mut signals {
            let message = Message::Signal(signal);
            println!("{}", signal);
            sender.send(message).unwrap();
        }
        handle.close();
    }

    fn input(sender: Sender<Message>) {
        loop {
            let mut keys = stdin().keys();
            if let Some(Ok(key)) = keys.next() {
                let message = Message::Input(key);
                sender.send(message).unwrap();
            }
        }
    }

    fn display(
        &mut self,
        term: &mut RawTerminal<Stdout>,
        size: Size,
    ) -> Result<(), Box<dyn Error>> {
        write!(
            term,
            "{}{}{}",
            color::Bg(color::Reset),
            cursor::Goto(1, 1),
            clear::All
        )?;
        match self.mode {
            Mode::Command => self.display_command(term, size),
            Mode::Switch => self.display_switch(term, size),
        }
    }

    fn display_command(
        &mut self,
        term: &mut RawTerminal<Stdout>,
        size: Size,
    ) -> Result<(), Box<dyn Error>> {
        let (_columns, rows) = size.try_into()?;
        match self.files.get_mut(self.current) {
            Some(file) => {
                let (position, relative) = file.display(
                    term,
                    Size {
                        lines: size.lines - 1,
                        columns: size.columns,
                    },
                )?;
                write!(
                    term,
                    "{}{}{}{} {}:{}",
                    cursor::Goto(1, rows),
                    color::Bg(color::Green),
                    clear::CurrentLine,
                    file.path,
                    position.line,
                    position.column,
                )?;
                if let Ok((column, row)) = relative.try_into() {
                    write!(term, "{}", cursor::Goto(column, row))?;
                }
            }
            None => {
                write!(
                    term,
                    "{}{}{}",
                    cursor::Goto(1, rows),
                    color::Bg(color::Green),
                    clear::CurrentLine,
                )?;
            }
        }
        Ok(())
    }

    fn display_switch(
        &mut self,
        term: &mut RawTerminal<Stdout>,
        size: Size,
    ) -> Result<(), Box<dyn Error>> {
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
            .try_for_each(|file| write!(term, "{}\r\n", file))?;
        if let Some(path) = self.view.get(self.position.line) {
            write!(
                term,
                "{}{}{}{}",
                cursor::Goto(1, (self.position.line - self.offset.line + 1).try_into()?),
                color::Bg(color::LightBlack),
                clear::CurrentLine,
                path
            )?;
        }
        write!(
            term,
            "{}{}{}{} {}",
            cursor::Goto(1, size.lines.try_into()?),
            color::Bg(color::Blue),
            clear::CurrentLine,
            self.path,
            self.query,
        )?;
        Ok(())
    }

    fn handle_signal(&mut self, signal: Signal) -> Result<(), Box<dyn Error>> {
        match signal {
            SIGINT | SIGTERM | SIGQUIT => self.exit(),
            _ => {}
        }
        Ok(())
    }

    fn handle_input(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match self.mode {
            Mode::Command => self.handle_command(key),
            Mode::Switch => self.handle_switch(key),
        }
    }

    fn handle_command(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match key {
            Key::Char('\t') => self.switch()?,
            Key::Char('B') => self.exit(),
            _ => {}
        }
        if let Some(file) = self.files.get_mut(self.current) {
            match key {
                Key::Char('a') => file.goto(file.position.up()),
                Key::Char('A') => file.goto(Position::start()),
                Key::Char('s') => file.goto(file.position.down()),
                Key::Char('S') => file.goto(Position::end()),
                Key::Char('d') => file.goto(file.position.left()),
                Key::Char('D') => file.goto(file.position.line_start()),
                Key::Char('f') => file.goto(file.position.right()),
                Key::Char('F') => file.goto(file.position.line_end()),
                Key::Up => file.goto(file.position.up()),
                Key::Down => file.goto(file.position.down()),
                Key::Left => file.goto(file.position.left()),
                Key::Right => file.goto(file.position.right()),
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_switch(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        match key {
            Key::Char('\t') => self.command(),
            Key::BackTab => {
                self.hide = !self.hide;
                self.list()?;
            }
            Key::Down => {
                if self.position.line + 1 < self.view.len() {
                    self.position.line += 1;
                }
            }
            Key::Up => {
                if self.position.line > 0 {
                    self.position.line -= 1;
                }
            }
            Key::Backspace => {
                self.query.pop();
                self.filter();
            }
            Key::Char('\n') => self.open()?,
            Key::Char(c) => {
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

    fn exit(&mut self) {
        self.run = false;
    }

    fn open(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(path) = self.view.get(self.position.line) {
            let file = File::open(path)?;
            self.files.push(file);
            self.current = self.files.len() - 1;
            self.mode = Mode::Command;
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

impl Drop for Editor {
    fn drop(&mut self) {
        stdout().flush().unwrap();
        stderr().flush().unwrap();
    }
}
