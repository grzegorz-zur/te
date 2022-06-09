pub const EOL: char = '\n';

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Position {
    pub fn start() -> Self {
        Position { line: 0, column: 0 }
    }
    pub fn next(&self, character: char) -> Self {
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
