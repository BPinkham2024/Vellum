use crossterm::style::Color;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Comment,
    // MD Specific
    Header,
    Bold,
    Italic,
    List,
}

impl Type {
    pub fn to_color(self) -> Color {
        match self {
            Type::Number => Color::Cyan,
            Type::Match => Color::Green,
            Type::String => Color::Magenta,
            Type::Comment => Color::DarkGrey,
            Type::Header => Color::Blue,
            Type::Bold => Color::White,
            Type::Italic => Color::Yellow, 
            Type::List => Color::Cyan,
            _ => Color::White,
        }
    }
}