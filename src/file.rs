use std::error::Error;
use std::fs::read_to_string;
use std::io::Stdout;
use std::io::Write;
use termion::raw::RawTerminal;
use termion::{clear, cursor, terminal_size};

pub struct File {
    path: String,
    content: String,
}

impl File {
    pub fn open(path: &str) -> Result<File, Box<dyn Error>> {
        let mut file = File {
            path: path.to_string(),
            content: String::new(),
        };
        file.read()?;
        Ok(file)
    }

    pub fn read(&mut self) -> Result<(), Box<dyn Error>> {
        self.content = read_to_string(&self.path)?;
        Ok(())
    }

    pub fn render(&self, term: &mut RawTerminal<Stdout>) -> Result<(), Box<dyn Error>> {
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
        Ok(())
    }
}
