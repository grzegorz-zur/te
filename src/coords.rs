use std::num::TryFromIntError;
use std::ops::{Add, Sub};

pub const EOL: char = '\n';

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub lines: usize,
    pub columns: usize,
}

impl Position {
    pub fn start() -> Self {
        Position { line: 0, column: 0 }
    }

    pub fn next(self, character: char) -> Self {
        match character {
            EOL => Position {
                line: self.line + 1,
                column: 0,
            },
            _ => Position {
                line: self.line,
                column: self.column + 1,
            },
        }
    }
}

impl From<(u16, u16)> for Position {
    fn from(tuple: (u16, u16)) -> Self {
        Position {
            line: (tuple.1 - 1).into(),
            column: (tuple.0 - 1).into(),
        }
    }
}

impl From<(u16, u16)> for Size {
    fn from(tuple: (u16, u16)) -> Self {
        Size {
            lines: tuple.1.into(),
            columns: tuple.0.into(),
        }
    }
}

impl TryInto<(u16, u16)> for Position {
    type Error = TryFromIntError;
    fn try_into(self) -> Result<(u16, u16), Self::Error> {
        Ok(((self.column + 1).try_into()?, (self.line + 1).try_into()?))
    }
}

impl TryInto<(u16, u16)> for Size {
    type Error = TryFromIntError;
    fn try_into(self) -> Result<(u16, u16), Self::Error> {
        Ok(((self.columns).try_into()?, (self.lines).try_into()?))
    }
}

impl Add<Size> for Position {
    type Output = Position;
    fn add(self: Position, size: Size) -> Self::Output {
        Position {
            line: self.line + size.lines,
            column: self.column + size.columns,
        }
    }
}

impl Sub<Position> for Position {
    type Output = Size;
    fn sub(self: Position, position: Position) -> Self::Output {
        Size {
            lines: self.line - position.line,
            columns: self.column - position.column,
        }
    }
}
