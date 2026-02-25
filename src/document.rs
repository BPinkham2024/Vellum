use ropey::Rope;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error};
use crate::editor::Position;
use crate::highlighting::Type;
use tree_sitter::{Parser, Tree, Query, QueryCursor};

pub struct Document {
    pub rope: Rope,
    pub filename: Option<String>,
    dirty: bool,
    undo_stack: Vec<Rope>, // Past states
    redo_stack: Vec<Rope>, // Future states
    pub parser: Parser,
    pub tree: Option<Tree>,
    pub query: Query,
    pub source_string: String,
}

impl Default for Document {
    fn default() -> Self {
        let mut parser = Parser::new();
        // Set language to markdown
        parser.set_language(tree_sitter_markdown::language()).expect("Failed to load markdown grammar");
        let tree = parser.parse("", None);

        let query = Query::new(
            tree_sitter_markdown::language(),
            "(atx_heading) @header
            (strong_emphasis) @bold
            (emphasis) @italic
            (list_item) @list
            (fenced_code_block) @string"
        ).unwrap();

        Self {
            rope: Rope::new(),
            filename: None,
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            parser,
            tree: tree,
            query,
            source_string: String::new(),
        }
    }
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, Error> {
        let file = File::open(filename)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        let mut parser = Parser::new();
        parser.set_language(tree_sitter_markdown::language()).expect("Failed to load markdown grammar");
        // Parse initial loaded file
        let text = rope.to_string();
        let tree = parser.parse(&text, None);

        let query = Query::new(
            tree_sitter_markdown::language(),
            "(atx_heading) @header
            (strong_emphasis) @bold
            (emphasis) @italic
            (list_item) @list
            (fenced_code_block) @string"
        ).unwrap();

        Ok(Self {
            rope,
            filename: Some(filename.to_string()),
            dirty: false,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            parser,
            tree: tree,
            query,
            source_string: text,
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

    pub fn update_tree(&mut self) {
        self.source_string = self.rope.to_string();
        self.tree = self.parser.parse(&self.source_string, None);
    }

    pub fn undo(&mut self) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(self.rope.clone());
            self.rope = prev;
            self.dirty = true;
            self.update_tree();
            return true;
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(self.rope.clone());
            self.rope = next;
            self.dirty = true;
            self.update_tree();
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
        self.update_tree();
    }

    pub fn insert_str(&mut self, at: &Position, text: &str) {
        if at.y >= self.len() { return; }
        let char_idx = self.get_char_index(at);
        self.rope.insert(char_idx, text);
        self.dirty = true;
        self.update_tree();
    }

    pub fn delete(&mut self, at: &Position) {
        let char_idx = self.get_char_index(at);
        // Don't delete past end of file
        if char_idx < self.rope.len_chars() {
            self.rope.remove(char_idx..char_idx + 1);
            self.dirty = true;
            self.update_tree();
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
            self.update_tree();
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
        self.update_tree();
    }

    pub fn indent(&mut self, y: usize, count: usize) {
        if y > self.len() { return; }
        let char_idx = self.rope.line_to_char(y);
        let spaces = " ".repeat(count * 4);
        self.rope.insert(char_idx, &spaces);
        self.dirty = true;
        self.update_tree();
    }

    pub fn get_highlights(&self, y: usize) -> Vec<crate::highlighting::Type> {
        let line = self.rope.line(y);
        let mut colors =  vec![crate::highlighting::Type::None; line.len_chars()];

        if let Some(tree) = &self.tree {
            let start_byte = self.rope.line_to_byte(y);
            let end_byte = start_byte + line.len_bytes();

            let mut cursor = QueryCursor::new();
            cursor.set_byte_range(start_byte, end_byte);

            let text_bytes = self.source_string.as_bytes();
            let matches = cursor.matches(
                &self.query, 
                tree.root_node(), 
                |node: tree_sitter::Node| &text_bytes[node.byte_range()]
            );
            
            for m in matches {
                for capture in m.captures {
                    let capture_name = self.query.capture_names()[capture.index as usize].as_str();
                    let hl_type = match capture_name {
                        "header" => Type::Header,
                        "bold" => Type::Bold,
                        "italic" => Type::Italic,
                        "list" => Type::List,
                        "string" => Type::String,
                        _ => Type::None,
                    };

                    let node = capture.node;

                    // Clamping ranges inside the line
                    let n_start_byte = std::cmp::max(node.start_byte(), start_byte);
                    let n_end_byte = std::cmp::min(node.end_byte(), end_byte);
                    if n_start_byte >= n_end_byte { continue; }

                    let start_char = self.rope.byte_to_char(n_start_byte).saturating_sub(self.rope.line_to_char(y));
                    let end_char = self.rope.byte_to_char(n_end_byte).saturating_sub(self.rope.line_to_char(y));
                    for i in start_char..end_char {
                        if i < colors.len() {
                            colors[i] = hl_type;
                        }
                    }
                }
            }
        }
        colors
    }
}