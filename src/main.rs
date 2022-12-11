mod coords;
mod editor;
mod file;
mod utils;

use crossterm::tty::IsTty;
use editor::*;
use std::env;
use std::error;
use std::io::{stdout, Error, ErrorKind};

fn main() -> Result<(), Box<dyn error::Error>> {
    if !stdout().is_tty() {
        return Err(Box::new(Error::new(ErrorKind::Other, "No TTY")));
    }
    let mut editor = Editor::create();
    let files: Vec<String> = env::args().skip(1).collect();
    for file in files {
        editor.open(&file)?;
    }
    editor.run()
}
