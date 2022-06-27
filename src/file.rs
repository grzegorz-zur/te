use std::error::Error;
use std::fs::read_to_string;
use std::io::Stdout;
use std::io::Write;
use termion::raw::RawTerminal;
use termion::{clear, color, cursor};

use crate::coords::*;
use crate::utils::*;

pub struct File {
    pub path: String,
    pub content: String,
    index: usize,
    offset: Position,
    render: bool,
}

impl File {
    pub fn open(path: &str) -> Result<File, Box<dyn Error>> {
        let mut file = File {
            path: path.to_string(),
            content: String::new(),
            index: 0,
            offset: Position::start(),
            render: true,
        };
        file.read()?;
        Ok(file)
    }

    pub fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = read_to_string(&self.path)?;
        Ok(())
    }

    pub fn render(
        &mut self,
        term: &mut RawTerminal<Stdout>,
        size: Size,
    ) -> Result<(Position, Position), Box<dyn Error>> {
        let position = self.position();
        (self.offset, self.render) = self.offset.shift(position, size);
        if self.render {
            write!(
                term,
                "{}{}{}",
                color::Bg(color::Reset),
                cursor::Goto(1, 1),
                clear::All
            )?;
            self.content
                .lines()
                .skip(self.offset.line)
                .take(size.lines)
                .map(|line| sub(line, self.offset.column..self.offset.column + size.columns))
                .try_for_each(|line| write!(term, "{}\r\n", line))?;
        }
        self.render = false;
        let relative = Position::start() + (position - self.offset);
        Ok((position, relative))
    }

    pub fn position(&self) -> Position {
        self.content
            .chars()
            .take(self.index)
            .fold(Position::start(), Position::next)
    }

    pub fn backward(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    pub fn forward(&mut self) {
        if self.index + 1 < self.content.len() {
            self.index += 1;
        }
    }
}
