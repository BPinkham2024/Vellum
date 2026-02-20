use ropey::Rope;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error};
use crate::editor::Position;

pub struct Document {
    pub rope: Rope,
    pub filename: Option<String>,
    dirty: bool,
    undo_stack: Vec<Rope>, // Past states
    redo_stack: Vec<Rope>, // Future states
}

impl Default for Document {
    fn default() -> Self {
        Self {
            rope: Rope::new(),
            filename: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new()
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        Ok(Self {
            rope,
            filename: Some(filename.to_string()),
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        })
    }
    
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let file = File::create(filename)?;
            self.rope.write_to(BufWriter::new(file))?;
            self.dirty = false;
        }
        Ok(())
    }

    // Snapshotting
    pub fn snapshot(&mut self) {
        self.undo_stack.push(self.rope.clone());
        self.redo_stack.clear(); // Can't redo if you edit the past
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.rope.clone());
            self.rope = prev;
            self.dirty = true;
            return true;
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.rope.clone());
            self.rope = next;
            self.dirty = true;
            return true;
        }
        false
    }

    // Info getters
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    pub fn len(&self) -> usize {
        self.rope.len_lines()
    }

    // Helper to translate 2d cursor into 1d rope index
    fn get_char_index(&self, at: &Position) -> usize {
        let line_start = self.rope.line_to_char(at.y);
        line_start + at.x
    }

    // Editing
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() { return; }
        let char_idx = self.get_char_index(at);
        self.rope.insert_char(char_idx, c);
        self.dirty = true;
    }

    pub fn insert_str(&mut self, at: &Position, text: &str) {
        if at.y >= self.len() { return; }
        let char_idx = self.get_char_index(at);
        self.rope.insert(char_idx, text);
        self.dirty = true;
    }

    pub fn delete(&mut self, at: &Position) {
        let char_idx = self.get_char_index(at);
        // Don't delete past end of file
        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
            self.dirty = true;
        }
    }

    // Comamnd helpers
    pub fn replace(&mut self, target: &str, replacement: &str) -> usize {
        // Convert to string, replace, and rebuild the rope
        // Works for now but may get slow with larget files
        let text = self.rope.to_string();
        let count = text.matches(target).count();

        if count > 0 {
            let new_text = text.replace(target, replacement);
            self.rope = ropey::Rope::from_str(&new_text);
            self.dirty = true;
        }
        count
    }

    pub fn set_header(&mut self, y: usize, level: usize) {
        if y >= self.len() { return; }

        let line = self.rope.line(y).to_string();
        let content = line.trim_start_matches("#").trim_start();
        let hashes = "#".repeat(level);
        let new_content = format!("{} {}", hashes, content);

        let char_idx = self.rope.line_to_char(y);
        let line_len = line.chars().count();

        // Remove old line and insert the formatted one
        self.rope.remove(char_idx..(char_idx + line_len));
        self.rope.insert(char_idx, &new_content);
        self.dirty = true;
    }

    pub fn indent(&mut self, y: usize, count: usize) {
        if y > self.len() { return; }
        let char_idx = self.rope.line_to_char(y);
        let spaces = " ".repeat(count * 4);
        self.rope.insert(char_idx, &spaces);
        self.dirty = true;
    }
}