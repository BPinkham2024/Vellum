mod editor;
mod terminal;
mod row;
mod document;
mod highlighting;

use editor::Editor;

fn main() {
    let mut editor = Editor::default();
    editor.run();
}