mod coords;
mod editor;
mod file;
mod utils;

use editor::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Editor::create()?.run()
}
