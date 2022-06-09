use std::error::Error;
use std::fs::read_to_string;
use std::io::Stdout;
use std::io::Write;
use termion::raw::RawTerminal;
use termion::{clear, cursor, terminal_size};

use crate::position::Position;

pub struct File {
    path: String,
    content: String,
    index: usize,
}

impl File {
    pub fn open(path: &str) -> Result<File, Box<dyn Error>> {
        let mut file = File {
            path: path.to_string(),
            content: String::new(),
            index: 0,
        };
        file.read()?;
        Ok(file)
    }

    pub fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = read_to_string(&self.path)?;
        Ok(())
    }

    pub fn render(&self, term: &mut RawTerminal<Stdout>) -> Result<(u16, u16), Box<dyn Error>> {
        let (_columns, rows) = terminal_size()?;
        let mut row: u16 = 1;
        for line in self.content.lines() {
            write!(
                term,
                "{}{}{}",
                cursor::Goto(1, row),
                line,
                clear::UntilNewline,
            )?;
            row += 1;
            if row == rows {
                break;
            }
        }
        let position = self.position();
        let column: u16 = position.column.try_into()?;
        let row: u16 = position.line.try_into()?;
        Ok((column + 1, row + 1))
    }

    pub fn position(&self) -> Position {
        self.content
            .chars()
            .take(self.index)
            .fold(Position::start(), |p, c| p.next(c))
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
