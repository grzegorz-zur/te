mod editor;

use editor::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Editor::create().run()
}
