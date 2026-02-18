use crossterm::{
    cursor,
    event::{read, Event, KeyEvent, KeyEventKind},
    queue,
    style::{Print, Color, SetBackgroundColor, SetForegroundColor},
    terminal::{enable_raw_mode, size, Clear, ClearType},
};
use std::io::{self, stdout, Write};

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Terminal {
    size: Size,
    stdout: io::Stdout,
}

impl Terminal {
    pub fn default() -> Result<Self, std::io::Error> {
        enable_raw_mode()?;
        Ok(Self {
            size: Size {
                width: size()?.0,
                height: size()?.1,
            },
            stdout: stdout(),
        })
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn read_key() -> Result<KeyEvent, std::io::Error> {
        loop {
            if let Event::Key(event) = read()? {
                // FILTER: Only return the event if it is a keypress, not release
                if event.kind == KeyEventKind::Press {
                    return Ok(event);
                }
            }
        }
    }

    // --- BUFFERED COMMANDS (These don't happen until flush is called) ---

    pub fn clear_screen(&mut self) {
        queue!(self.stdout, Clear(ClearType::All)).unwrap();
    }

    pub fn cursor_position(&mut self, x: u16, y: u16) {
        queue!(self.stdout, cursor::MoveTo(x, y)).unwrap();
    }

    pub fn cursor_hide(&mut self) {
        queue!(self.stdout, cursor::Hide).unwrap();
    }

    pub fn cursor_show(&mut self) {
        queue!(self.stdout, cursor::Show).unwrap();
    }

    pub fn clear_current_line(&mut self) {
        queue!(self.stdout, Clear(ClearType::CurrentLine)).unwrap();
    }

    // using queue! + Print instead of println!
    pub fn print(&mut self, string: &str) {
        queue!(self.stdout, Print(string)).unwrap();
    }

    pub fn set_bg_color(&mut self, color: Color) {
        queue!(self.stdout, SetBackgroundColor(color)).unwrap();
    }

    pub fn set_fg_color(&mut self, color: Color) {
        queue!(self.stdout, SetForegroundColor(color)).unwrap();
    }

    pub fn reset_colors(&mut self) {
        queue!(self.stdout, SetForegroundColor(Color::Reset)).unwrap();
        queue!(self.stdout, SetBackgroundColor(Color::Reset)).unwrap();
    }

    // Send all queued changes to the screen at once
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stdout.flush()
    }
}