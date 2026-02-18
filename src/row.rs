use std::cmp;

pub struct Row {
    string: String,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
        }
    }
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        self.string.get(start..end).unwrap_or("").to_string()
    }

    pub fn len(&self) -> usize {
        self.string.len()
    }

    pub fn is_empty(&self) -> bool {
        self.string.is_empty()
    }

    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len() {
            self.string.push(c);
        } else {
            self.string.insert(at, c);
        }
    }

    pub fn delete(&mut self, at: usize) {
        if at < self.len() {
            self.string.remove(at);
        }
    }

    pub fn append(&mut self, new: &Row) {
        self.string = format!("{}{}", self.string, new.string);
    }

    pub fn split(&mut self, at: usize) -> Row {
        let length = self.string.len();
        
        // Safety check to make sure there is no splitting past the end of the string
        let split_index = std::cmp::min(at, length);
        
        // split_off keeps the first part in self.string and returns the second part
        let remainder = self.string.split_off(split_index);
        
        Row::from(remainder.as_str())
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}