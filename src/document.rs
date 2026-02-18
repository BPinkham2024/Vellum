use crate::row::Row;
use crate::editor::Position;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub fn default() -> Self {
        Self {
            rows: Vec::new(),
        }
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

    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        if at.y >= len {
            return;
        }

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
}