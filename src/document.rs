use crate::row::Row;
use crate::editor::Position;
use std::fs;
use std::io::{Error, Write};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
}

impl Document {
    pub fn default() -> Self {
        Self {
            rows: Vec::new(),
            filename: None,
            dirty: false,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn open(filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();

        for value in contents.lines() {
            rows.push(Row::from(value));
        }

        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false
        })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.dirty = false;
        }
        Ok(())
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;
        
        if c == '\n' {
            self.insert_newline(at);
            return;
        }

        if at.y == self.len() {
            let mut row = Row::from("");
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.insert(at.x, c);
        }
    }

    pub fn insert_at(&mut self, y: usize, x: usize, text: &str) {
        if let Some(row) = self.rows.get_mut(y) {
            row.insert_str(x, text);
            self.dirty = true;
        }
    }

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }
        self.dirty = true;

        if at.x == self.rows.get(at.y).unwrap().len() && at.y < len - 1 {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            row.append(&next_row);
        } else {
            let row = self.rows.get_mut(at.y).unwrap();
            row.delete(at.x);
        }
    }

    fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }
        
        if at.y == self.len() {
            self.rows.push(Row::from(""));
        } else {
            let current_row = self.rows.get_mut(at.y).unwrap();
            let new_row = current_row.split(at.x);
            self.rows.insert(at.y + 1, new_row);
        }
    }

    // Header helper
    pub fn set_header(&mut self, row_idx: usize, level: usize) {
        if let Some(row) = self.rows.get_mut(row_idx) {
            // Remove existing headings
            let content = row.string.trim_start_matches('#').trim_start();
            let hashes = "#".repeat(level);
            row.string = format!("{} {}", hashes, content);
            row.highlight();
        }
        self.dirty = true;
    }

    // Intending helper
    pub fn indent(&mut self, row_idx: usize, count: usize) {
        if let Some(row) = self.rows.get_mut(row_idx) {
            let spaces = "\t".repeat(count);
            row.string.insert_str(0, &spaces);
        }
        self.dirty = true;
    }

}