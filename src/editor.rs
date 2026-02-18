use crate::terminal::Terminal;
use crate::document::Document;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// Cursor coordinates, non-negative
pub struct Position {
    pub x: usize,
    pub y: usize,
}

// Main state of the editor
// keeps track of terminal size and where user is looking
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
}

impl Editor {
    // Initialize the editor with default values
    pub fn default() -> Self {
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position { x: 0, y: 0 },
            document: Document::default(),
        }
    }

    // The main loop
    // 1. Draw the UI
    // 2. Wait for a keypress
    // 3. Process the keypress
    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.refresh_screen() {
                self.die(&e);
            }
            if self.should_quit {
                break;
            }
            if let Err(e) = self.process_keypress() {
                self.die(&e);
            }
        }
    }

    // Reads a single key event and updates state
    fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            // Quit on Ctrl+Q 
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => self.should_quit = true,

            // TYPING: Handle Enter
            KeyEvent { code: KeyCode::Enter, .. } => {
                self.document.insert(&self.cursor_position, '\n');
                self.cursor_position.y += 1;
                self.cursor_position.x = 0;
            }

            // TYPING: Handle Character insertion
            KeyEvent { code: KeyCode::Char(c), .. } => {
                self.document.insert(&self.cursor_position, c);
                self.cursor_position.x += 1;
            }

            // TYPING: Handle Backspace
            KeyEvent { code: KeyCode::Backspace, .. } => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    if self.cursor_position.x > 0 {
                        self.cursor_position.x -= 1;
                        self.document.delete(&self.cursor_position);
                    } else {
                        // Moving back a line (complex logic simplified for now)
                        let previous_row_len = self.document.row(self.cursor_position.y - 1).unwrap().len();
                        self.cursor_position.y -= 1;
                        self.cursor_position.x = previous_row_len;
                        self.document.delete(&self.cursor_position);
                    }
                }
            }
            
            // Delegate movement logic
            KeyEvent {
                code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                ..
            } => self.move_cursor(pressed_key.code),
            
            _ => (),
        }
        Ok(())
    }

    // Simplifying cursor movement, takes in key code and translates to movement
    fn move_cursor(&mut self, key: KeyCode) {
        let Position { mut x, mut y } = self.cursor_position;
        let size = self.terminal.size();
        let height = size.height as usize;
        let width = size.width as usize;

        match key {
            KeyCode::Up | KeyCode::Char('w') => y = y.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('s') => {
                if y < height.saturating_sub(1) { y += 1; }
            }
            KeyCode::Left | KeyCode::Char('a') => x = x.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('d') => {
                if x < width.saturating_sub(1) { x += 1; }
            }
            _ => (),
        }
        self.cursor_position = Position { x, y };
    }

    // Renders the TUI
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        // 1. Hide the cursor so it doesn't jump around while being drawn
        self.terminal.cursor_hide();
        
        // 2. Move to top-left to start drawing
        self.terminal.cursor_position(0, 0);

        // 3. Queue up the drawing commands
        if self.should_quit {
            self.terminal.clear_screen();
            self.terminal.print("Goodbye.\r\n");
        } else {
            self.draw_rows();
            
            // 4. Put the cursor back where it belongs
            self.terminal.cursor_position(
                self.cursor_position.x as u16, 
                self.cursor_position.y as u16
            );
        }

        // 5. Show the cursor again
        self.terminal.cursor_show();
        
        // 6. THE BIG FLUSH
        self.terminal.flush()
    }

    // Draws each row
    fn draw_rows(&mut self) {
        let height = self.terminal.size().height;
        
        for terminal_row in 0..height {
            // Clear the line so old text doesn't linger
            self.terminal.clear_current_line();
            
            // If the row exists in the document, render it
            if let Some(row) = self.document.row(terminal_row as usize) {
                self.terminal.print(&row.render(0, self.terminal.size().width as usize));
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                // ~ for empty lines, thank you vim
                self.terminal.print("~");
            }
            
            if terminal_row < height - 1 {
                self.terminal.print("\r\n");
            }
        }
    }

    fn draw_welcome_message(&mut self) {
        let mut welcome = format!("Vellum Editor -- Version 0.0.1");
        let width = self.terminal.size().width as usize;
        let len = welcome.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding);
        
        welcome = format!("{}{}", spaces, welcome);
        welcome.truncate(width);
        
        self.terminal.print(&welcome);
    }
    
    // Updated to match terminal struct
    fn die(&mut self, e: &std::io::Error) {
        self.terminal.clear_screen();
        panic!("{}", e);
    }
}