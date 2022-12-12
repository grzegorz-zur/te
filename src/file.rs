use crate::coords::*;
use crate::utils::*;
use crossterm::cursor::*;
use crossterm::queue;
use crossterm::style::*;
use crossterm::terminal::*;
use std::error::Error;
use std::fs;
use std::io::stdout;
use std::time::SystemTime;

pub struct File {
    pub path: String,
    pub timestamp: SystemTime,
    pub content: String,
    pub position: Position,
    pub offset: Position,
}

impl File {
    pub fn open(path: &str) -> Result<File, Box<dyn Error>> {
        let mut file = File {
            path: path.to_string(),
            timestamp: SystemTime::now(),
            content: String::new(),
            position: Position::start(),
            offset: Position::start(),
        };
        file.read()?;
        Ok(file)
    }

    pub fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = fs::read_to_string(&self.path)?;
        self.timestamp = fs::metadata(&self.path)?.modified()?;
        self.goto(self.position);
        Ok(())
    }

    pub fn refresh(&mut self) -> Result<bool, Box<dyn Error>> {
        let timestamp = fs::metadata(&self.path)?.modified()?;
        if self.timestamp == timestamp {
            return Ok(false);
        }
        self.read()?;
        Ok(true)
    }

    pub fn display(&mut self, size: Size) -> Result<(Position, Position), Box<dyn Error>> {
        self.offset = self.offset.shift(self.position, size);
        queue!(stdout(), ResetColor, MoveTo(0, 0), Clear(ClearType::All),)?;
        self.content
            .lines()
            .skip(self.offset.line)
            .take(size.lines)
            .map(|line| sub(line, self.offset.column..self.offset.column + size.columns))
            .try_for_each(|line| queue!(stdout(), Print(line), MoveToNextLine(1)))?;
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
