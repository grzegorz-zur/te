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
    pub position: Position,
    pub offset: Position,
}

impl File {
    pub fn open(path: &str) -> Result<File, Box<dyn Error>> {
        let mut file = File {
            path: path.to_string(),
            content: String::new(),
            position: Position::start(),
            offset: Position::start(),
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
        self.offset = self.offset.shift(self.position, size);
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
        let relative = Position::start() + (self.position - self.offset);
        Ok((self.position, relative))
    }

    pub fn goto(&mut self, target: Position) {
        let mut position = Position::start();
        self.position = position;
        for character in self.content.chars() {
            position = position.next(character);
            if position <= target {
                self.position = position;
            } else {
                break;
            }
        }
    }
}
