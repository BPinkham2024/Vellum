use crate::terminal::Terminal;
use crate::document::Document;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::Color;
use std::{
    env,
    time::{Duration, Instant}
};

// Cursor coordinates, non-negative
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq)]
enum Mode {
    Normal,
    Insert,
    Command(String), //Holds the command being typed
}

// Main state of the editor
// keeps track of terminal size and where user is looking
pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    document: Document,
    status_message: StatusMessage,
    mode: Mode,
    show_line_numbers: bool,
    row_offset: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text:message,
        }
    }
}

impl Editor {
    // Initialize the editor with default values
    pub fn default() -> Self {

        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("Normal Mode - Press 'i' to insert".to_string());

        let document = if args.len() > 1 {
            let filename = &args[1];
            let doc = Document::open(filename);
            if let Ok(doc) = doc {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", filename);
                Document::default()
            }
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position { x: 0, y: 0 },
            document,
            status_message: StatusMessage::from(initial_status),
            mode: Mode::Normal,
            show_line_numbers: true,
            row_offset: 0,
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
        
        match &self.mode {
            Mode::Normal => self.process_normal_mode(pressed_key),
            Mode::Insert => self.process_insert_mode(pressed_key),
            Mode::Command(_) => self.process_command_mode(pressed_key),
        }
    }

    fn process_normal_mode(&mut self, key: KeyEvent) -> Result<(), std::io::Error> {
        match key {

            // Enter insert mode
            KeyEvent { code: KeyCode::Char('i'), .. } => {
                self.document.snapshot();
                self.mode = Mode::Insert;
                self.status_message = StatusMessage::from("Insert Mode".to_string());
            }

            // Enter command mode
            KeyEvent { code: KeyCode::Char(':'), .. } => {
                self.mode = Mode::Command(String::new());
                self.status_message = StatusMessage::from("Command: ".to_string());
            }

            // Quick escape on ctrl + q
            KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL, .. } => self.should_quit = true,

            // Save with Ctrl+S (keeping for now, not 100% sure w and !w work as I want yet)
            KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                if self.document.filename.is_none() {
                    let new_name = self.prompt("Save as: ")?;
                    if let Some(name) = new_name {
                        self.document.filename = Some(name);
                    } else {
                        self.status_message = StatusMessage::from("Save aborted.".to_string());
                        return Ok(());
                    }
                }
                
                if self.document.save().is_ok() {
                    self.status_message = StatusMessage::from("File saved successfully.".to_string());
                } else {
                    self.status_message = StatusMessage::from("Error writing file!".to_string());
                }
            }

            KeyEvent { code: KeyCode::Char('u'), .. } => {
                if self.document.undo() {
                    self.status_message = StatusMessage::from("Undo".to_string());
                }
            }

            KeyEvent { code: KeyCode::Char('r'), .. } => {
                if self.document.redo() {
                    self.status_message = StatusMessage::from("Redo".to_string());
                }
            }
            
            // Delegate movement logic
            KeyEvent {
                code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right,
                ..
            } => self.move_cursor(key.code),
            
            _ => (),
        }
        Ok(())
    }

    fn process_insert_mode(&mut self, key: KeyEvent) -> Result<(), std::io::Error> {
        match key {

            // Exit into normal mode
            KeyEvent { code: KeyCode::Esc, .. } => {
                self.mode = Mode::Normal;
                self.status_message = StatusMessage::from("Normal Mode".to_string());
            }

            // Typing logic (moved from process_normal_mode)
            // Handle Enter
            KeyEvent { code: KeyCode::Enter, .. } => {
                self.document.snapshot();
                self.document.insert(&self.cursor_position, '\n');
                self.cursor_position.y += 1;
                self.cursor_position.x = 0;
            }

            // Save state every space
            KeyEvent { code: KeyCode::Char(' '), .. } => {
                self.document.snapshot();
                self.document.insert(&self.cursor_position, ' ');
                self.cursor_position.x += 1;
            }

            // Handle Character insertion
            KeyEvent { code: KeyCode::Char(c), .. } => {
                self.document.insert(&self.cursor_position, c);
                self.cursor_position.x += 1;
            }

            // Handle Backspace
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
            
            // Movement logic
            KeyEvent { code: KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right, .. } => {
                self.move_cursor(key.code);
            }

            _ => (),
        }
        Ok(())
    }

    fn process_command_mode(&mut self, key: KeyEvent) -> Result<(), std::io::Error> {
        let mut command = if let Mode::Command(s) = &self.mode { s.clone() } else { String::new() };

        match key {
            // Execute command
            KeyEvent { code: KeyCode::Enter, .. } => {
                let _ = self.execute_command(&command);
                self.mode = Mode::Normal;
                self.status_message = StatusMessage::from(String::new());
            }
            // Cancel command
            KeyEvent { code: KeyCode::Esc, .. } => {
                self.mode = Mode::Normal;
                self.status_message = StatusMessage::from(String::new());
            }
            // Edit command string
            KeyEvent { code: KeyCode::Backspace, .. } => {
                command.pop();
                self.mode = Mode::Command(command);
            }
            KeyEvent { code: KeyCode::Char(c), .. } => {
                command.push(c);
                self.mode = Mode::Command(command);
            }
            _ => (),
        }
        Ok(())
    }

    fn execute_command(&mut self, command: &str) -> Result<(), std::io::Error> {

        // I want edits from commands to be able to be reversed/redone
        self.document.snapshot();

        // Search and replace logic (vim syntax so all one word not split by whitespace)
        if command.starts_with("s/") {
            let parts: Vec<&str> = command.split('/').collect();

            // Expectation is ["s", "old_text", "new_text"]
            if parts.len() >= 3 {
                let target = parts[1];
                let replacement = parts[2];

                let count = self.document.replace(target, replacement);
                self.status_message = StatusMessage::from(format!("Replaced '{}' in {} lines", target, count));

                // Saftey clamp for cursor (pulls back to end of line)
                let current_len = self.document.row(self.cursor_position.y).map_or(0, |r| r.len());
                if self.cursor_position.x > current_len {
                    self.cursor_position.x = current_len;
                }
            } else {
                self.status_message = StatusMessage::from("Usage: s/old/new".to_string());
            }
            return Ok(());
        }


        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() { return Ok(());}

        match parts[0] {
            "q" => self.should_quit = true,
            "w" => {
                if let Err(e) = self.document.save() {
                    self.status_message = StatusMessage::from(format!("Error: {}", e));
                } else {
                    self.status_message = StatusMessage::from("File saved.".to_string());
                }
            },
            "!w" => {
                if parts.len() > 1 {
                    let new_name = parts[1].to_string();
                    self.document.filename = Some(new_name);
                    self.document.save()?;
                    self.status_message = StatusMessage::from("File saved as new name.".to_string());
                } else {
                    self.status_message = StatusMessage::from("Error: !w requires a filename".to_string());
                }
            },
            "head" => {
                if parts.len() > 1 {
                    if let Ok(level) = parts[1].parse::<usize>() {
                        self.document.set_header(self.cursor_position.y, level);
                    }
                }
            },
            "bold" => self.wrap_word("**"),
            "italic" => self.wrap_word("*"),
            "t" => {
                if parts.len() > 1 {
                    if let Ok(count) = parts[1].parse::<usize>() {
                        self.indent_line(count);
                    }
                }
            },
            "find" => {
                if parts.len() > 1 {
                    let query = parts[1];
                    self.find_next(query);
                }
            },
            "ln" => {
                self.show_line_numbers = !self.show_line_numbers;
                self.status_message = StatusMessage::from(format!("Line numbers: {}", self.show_line_numbers));
            }
            _ => self.status_message = StatusMessage::from(format!("Unknown command: {}", command)),
        }
        Ok(())
    }

    // Wrap word for bold and italics
    fn wrap_word(&mut self, wrapper: &str) {
        let y = self.cursor_position.y;
        let x = self.cursor_position.x;

        if let Some(row) = self.document.row(y) {
            let line = &row.string;

            // Find start of word
            let start = line[..x].rfind(' ').map(|i| i + 1).unwrap_or(0);
            
            // Find end of word
            let end = line[x..].find(' ').map(|i| x + i).unwrap_or(line.len());

            // Since we are mutating the line, document needs to be called
            self.document.insert_at(y, end, wrapper); // Suffex first so we don't mess with indices for prefix insertion
            self.document.insert_at(y, start, wrapper);

            // Move cursor to end of word
            self.cursor_position.x = end + (wrapper.len() * 2);
        }
    }

    fn find_next(&mut self, query: &str) {
        let start_y = self.cursor_position.y;
        let mut y = start_y;

        loop {
            if let Some(row) = self.document.row(y) {
                if let Some(x) = row.string.find(query) {
                    // Check if found after current cursor position only if cursor on y = stary_y
                    if y != start_y || x > self.cursor_position.x {
                        self.cursor_position.x = x;
                        self.cursor_position.y = y;
                        self.status_message = StatusMessage::from(format!("Found: {}", query));
                        return;
                    }
                }
            }

            y += 1;
            // Wrap to top if hitting bottom
            if y >= self.document.len() {
                y = 0;
            }
            // If word not found in full wrapping of doc, stop
            if y == start_y {
                self.status_message = StatusMessage::from(format!("Not found: {}", query));
                return;
            }
        }
    }

    fn indent_line(&mut self, count: usize) {
        self.document.indent(self.cursor_position.y, count);
        self.cursor_position.x += count * 4;
    }

    fn scroll(&mut self) {
        let terminal_height = self.terminal.size().height as usize - 2; // -2 for the status bar

        // Move offset up if cursor goes above visible screen
        if self.cursor_position.y < self.row_offset {
            self.row_offset = self.cursor_position.y;
        }
        // Move offset down if cursor goes below visible screen
        else if self.cursor_position.y >= self.row_offset + terminal_height {
            self.row_offset = self.cursor_position.y - terminal_height + 1;
        }
    }


    // Simplifying cursor movement, takes in key code and translates to movement
    fn move_cursor(&mut self, key: KeyCode) {
        let Position { mut x, mut y } = self.cursor_position;
        // The limit is the number of rows in the document
        let height = self.document.len();

        // Allow the cursor to go to the very end of the line to continue typing
        // Handles cases where row is empty
        let width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            KeyCode::Up | KeyCode::Char('w') => {
                y = y.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if y < height { 
                    y = y.saturating_add(1); 
                }
            }
            KeyCode::Left | KeyCode::Char('a') => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    // Wrap to end of previous line
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            },
            KeyCode::Right | KeyCode::Char('d') => {
                if x < width {
                    x += 1;
                } else if y < height {
                    // Wrap to start of next line
                    y += 1;
                    x = 0;
                }
            }
            _ => (),
        }


        // Clamping
        // 1. Vertical constraint: cannot go past last valid line
        // Limiting y to the number of rows
        if y > height {
            y = height;
        }

        // 2. Horizontal constraint: cannot go past the last character in the line
        // If moving vertically, x will be set to last index of line.
        let new_row_len = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > new_row_len {
            x = new_row_len;
        }


        self.cursor_position = Position { x, y };
    }

    // Helper to calculate gutter width
    fn gutter_width(&self) -> usize {
        if !self.show_line_numbers {
            return 0;
        }

        // Adds 2 for padding and pipe
        self.document.len().to_string().len() + 2
    }

    // Renders the TUI
    fn refresh_screen(&mut self) -> Result<(), std::io::Error> {
        self.scroll();

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
            self.draw_status_bar();
            self.draw_message_bar();
            
            // 4. Put the cursor back where it belongs and with offset
            let offset_x = self.gutter_width() as u16;
            let screen_y = (self.cursor_position.y - self.row_offset) as u16;

            self.terminal.cursor_position(
                self.cursor_position.x as u16 + offset_x, 
                screen_y
            );
        }

        // 5. Show the cursor again
        self.terminal.cursor_show();
        
        // 6. THE BIG FLUSH
        self.terminal.flush()
    }


    // "Save As" implementation (roughly)
    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            match Terminal::read_key()? {
                KeyEvent {code: KeyCode::Backspace, .. } => {
                    if !result.is_empty() {
                        result.pop();
                    }
                }
                KeyEvent { code: KeyCode::Enter, .. } => {
                    if result.is_empty() {
                        return Ok(None);
                    }
                    self.status_message = StatusMessage::from(String::new());
                    return Ok(Some(result));
                }
                KeyEvent { code: KeyCode::Char(c), .. } => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                KeyEvent { code: KeyCode::Esc, .. } => {
                    self.status_message = StatusMessage::from(String::new());
                    return Ok(None);
                }
                _ => (),
            }
        }
    }

    // Draws each row
    fn draw_rows(&mut self) {
        let height = self.terminal.size().height;
        let width = self.terminal.size().width as usize;
        let gutter = self.gutter_width();
        
        for terminal_row in 0..height  - 2 { // subtracting 2 allows for the status and message bar
            // Clear the line so old text doesn't linger
            self.terminal.clear_current_line();

            let doc_row = terminal_row as usize + self.row_offset;

            // Draw line numbers
            if self.show_line_numbers {
                self.terminal.set_fg_color(Color::DarkGrey);
                if doc_row < self.document.len() {
                    let num_str = format!("{:>w$} |", doc_row + 1, w = gutter - 2);
                    self.terminal.print(&num_str);
                } else {
                    let empty_str = format!("{:>w$} |", "~", w = gutter - 2);
                    self.terminal.print(&empty_str);
                }
                self.terminal.reset_colors();
            }

            // If the row exists in the document, render it
            if let Some(row) = self.document.row(doc_row) {

                let start = 0;
                let end = width.saturating_sub(gutter);
                let render_string = row.render(start, end);

                for (i, c) in render_string.chars().enumerate() {
                    if let Some(hl_type) = row.highlighting.get(i) {
                        let color = hl_type.to_color();
                        self.terminal.set_fg_color(color);
                    } else {
                        self.terminal.set_fg_color(Color::Reset);
                    }

                    self.terminal.print(&c.to_string());
                }

                self.terminal.reset_colors();
            } else if self.document.is_empty() && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else if !self.show_line_numbers {
                // ~ for empty lines, thank you vim
                self.terminal.print("~");
            }
            
            if terminal_row < height - 1 {
                self.terminal.print("\r\n");
            }
        }
    }

    fn draw_welcome_message(&mut self) {
        let mut welcome = if let Some(name) = &self.document.filename {
            format!("Editing: {}", name)
        } else {
            format!("Vellum Editor -- Version 0.0.1")
        };
        let width = self.terminal.size().width as usize;
        let len = welcome.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding);
        
        welcome = format!("{}{}", spaces, welcome);
        welcome.truncate(width);
        
        self.terminal.print(&welcome);
    }

    fn draw_status_bar(&mut self) {
        let mut status;
        let width = self.terminal.size().width as usize;

        if let Mode::Command(cmd) = &self.mode {
            status = format!("Command: {}", cmd);
        } else {
            let modified_indicator = if self.document.is_dirty() { "(modified)" } else { "" };
            
            let mut filename = "[No Name]".to_string();
            if let Some(name) = &self.document.filename {
                filename = name.clone();
                // truncation if name too long
                if filename.len() > 20 {
                    filename.truncate(20);
                    filename.push_str("...");
                }
            }

            status = format!("{} - {} lines {}", filename, self.document.len(), modified_indicator);
        }

        

        let line_indicator = format!("{}/{}", self.cursor_position.y + 1, self.document.len());

        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }
        status = format!("{}{}", status, line_indicator);
        status.truncate(width);

        // Styling for status
        self.terminal.set_bg_color(Color::White);
        self.terminal.set_fg_color(Color::Black);
        self.terminal.print(&status);

        // Reset colors
        self.terminal.reset_colors();
        self.terminal.print("\r\n");


    }

    fn draw_message_bar(&mut self) {
        self.terminal.clear_current_line();
        let msg = &self.status_message;
        if Instant::now() - msg.time < Duration::from_secs(5) {
            let mut text = msg.text.clone();
            text.truncate(self.terminal.size().width as usize);
            self.terminal.print(&text);
        }
    }
    
    // Updated to match terminal struct
    fn die(&mut self, e: &std::io::Error) {
        self.terminal.clear_screen();
        panic!("{}", e);
    }
}