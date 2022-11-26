mod coords;
mod editor;
mod file;
mod utils;

use crossterm::tty::IsTty;
use editor::*;
use std::error;
use std::io::{stdout, Error, ErrorKind};

fn main() -> Result<(), Box<dyn error::Error>> {
    if !stdout().is_tty() {
        return Err(Box::new(Error::new(ErrorKind::Other, "No TTY")));
    }
    Editor::create().run()
}
