mod editor;
mod file;
mod position;

use editor::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Editor::create().run()
}
