use crate::terminal::Terminal;
use crate::document::Document;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{
    env,
    time::Instant
};

// Cursor coordinates, non-negative
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command(String), //Holds the command being typed
}

// Main state of the editor
// keeps track of terminal size and where user is looking
pub struct Editor {
    pub(crate) should_quit: bool,
    pub(crate) terminal: Terminal,
    pub(crate) cursor_position: Position,
    pub(crate) document: Document,
    pub(crate) status_message: StatusMessage,
    pub(crate) mode: Mode,
    pub(crate) show_line_numbers: bool,
    pub(crate) row_offset: usize,
}

pub(crate) struct StatusMessage {
    pub(crate) text: String,
    pub(crate) time: Instant,
}

impl StatusMessage {
    pub(crate) fn from(message: String) -> Self {
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

        let mut editor = Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            cursor_position: Position { x: 0, y: 0 },
            document,
            status_message: StatusMessage::from(initial_status.to_string()),
            mode: Mode::Normal,
            show_line_numbers: true,
            row_offset: 0,
        };

        editor.load_config();

        // Reset startup message so it doesn't just show the last command from the config
        editor.status_message = StatusMessage::from(initial_status);

        editor
    }

    // The main loop
    // 1. Draw the UI
    // 2. Wait for a keypress
    // 3. Process the keypress
    pub fn run(&mut self) {
        loop {
            if let Err(e) = crate::ui::refresh_screen(self) {
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
                crate::commands::execute_command(self, &command)?;

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

    pub fn scroll(&mut self) {
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



    // "Save As" implementation (roughly)
    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            crate::ui::refresh_screen(self)?;

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

    fn load_config(&mut self) {
        if let Ok(home) = std::env::var("HOME") {
            let config_path = format!("{}/.vellumrc", home);

            if let Ok(contents) = std::fs::read_to_string(config_path) {
                for line in contents.lines() {
                    let cmd = line.trim();
                    // Skip empty lines and comments
                    if !cmd.is_empty() && !cmd.starts_with("#") {
                        // Ignoring errors so bad configs don't crash the program
                        let _ = crate::commands::execute_command(self, cmd);
                    }
                }
            }
        }
    }
    
    // Updated to match terminal struct
    fn die(&mut self, e: &std::io::Error) {
        self.terminal.clear_screen();
        panic!("{}", e);
    }
}