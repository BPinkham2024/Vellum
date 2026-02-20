use crate::highlighting::Type;

#[derive(Clone)]
pub struct Row {
    pub(crate) string: String,
    pub highlighting: Vec<Type>,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        let mut row = Self {
            string: String::from(slice),
            highlighting: Vec::new(),
        };
        row.highlight();
        row
    }
}

impl Row {
    pub fn highlight(&mut self) {
        let mut highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut index = 0;

        while index < chars.len() {
            // Default to no highlight
            highlighting.push(Type::None);
            index += 1;
        }

        // Headers
        if self.string.starts_with("#") {
            for i in 0..self.string.len() {
                highlighting[i] = Type::Header;
            }
        }

        // Lists
        if self.string.starts_with("- ") || self.string.starts_with("* ") || self.string.starts_with("+ ") {
            highlighting[0] = Type::List; // Bullet list
        }

        if chars.len() > 2 && chars[0].is_numeric() && chars[1] == '.' && chars[2] == ' ' {
            highlighting[0] = Type::List;
            highlighting[1] = Type::List;
        }

        // Inline formatting
        let mut i = 0;
        while i < chars.len() {
            // Bold (**)
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                let start = i;
                i += 2;
                while i + 1 < chars.len() {
                    if chars[i] == '*' && chars[i + 1] == '*' {
                        // Highlight everything inbetween the pair
                        for j in start..=i+1 {
                            highlighting[j] = Type::Bold;
                        }
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            // Italics 
            else if chars[i] == '*' {
                let start = i;
                i += 1;
                while i < chars.len() {
                    if chars[i] == '*' {
                        for j in start..=i {
                            // Don't overwrite bold (might have messed up logic just a safeguard)
                            if highlighting[j] == Type::None {
                                highlighting[j] = Type::Italic;
                            }
                        }
                        i += 1;
                        break;
                    }
                    i += 1;
                }
            }
            else {
                i += 1;
            }
        }

        self.highlighting = highlighting;

    }
}