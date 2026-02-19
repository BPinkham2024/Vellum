mod editor;
mod terminal;
mod row;
mod document;
mod highlighting;
mod ui;
mod commands;

use editor::Editor;

fn main() {
    let mut editor = Editor::default();
    editor.run();
}