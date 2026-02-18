// TODO: implement command system

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{self, stdout, Write};

// Struct to hold cursor coordinates
// Using usize because screen coordinates can't be negative
struct Position {
    x: usize,
    y: usize,
}

// Main state of the editor
// Keeps track of the terminal size and where user is looking
struct Editor {
    should_quit: bool,
    terminal_size: (u16, u16),
    cursor_position: Position,
}

impl Editor {
    // Initialize the editor with default values
    fn default() -> Self {
        let size = terminal::size().unwrap_or((0, 0));
        Self {
            should_quit: false,
            terminal_size: size,
            cursor_position: Position { x: 0, y: 0 },
        }
    }

    // The main loop
    // 1. Draw the UI
    // 2. Wait for a keypress
    // 3. Process the keypress
    fn run(&mut self) -> io::Result<()> {
        // Raw allows for typing multiple lines
        enable_raw_mode()?;

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
        
        disable_raw_mode()?;
        Ok(())
    }

    // Reads a single key event and updates state
    fn process_keypress(&mut self) -> io::Result<()> {
        // event::read() blocks until an event is received
        match event::read()? {
            Event::Key(key) => self.process_cursor_movement(key),
            _ => (), // Ignore resize events for now (basic prototype)
        }
        Ok(())
    }

    // Logic for moving the cursor or quitting
    fn process_cursor_movement(&mut self, key: KeyEvent) {
        match key {
            // Quit on Ctrl+Q 
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => self.should_quit = true,

            // Movement keys (WASD + arrow keys)
            KeyEvent { code: KeyCode::Up, .. } 
            | KeyEvent { code: KeyCode::Char('w'), .. } => {
                if self.cursor_position.y > 0 {
                    self.cursor_position.y -= 1;
                }
            }
            KeyEvent { code: KeyCode::Down, .. } 
            | KeyEvent { code: KeyCode::Char('s'), .. } => {
                if self.cursor_position.y < (self.terminal_size.1 as usize).saturating_sub(1) {
                    self.cursor_position.y += 1;
                }
            }
            KeyEvent { code: KeyCode::Left, .. } 
            | KeyEvent { code: KeyCode::Char('a'), .. } => {
                if self.cursor_position.x > 0 {
                    self.cursor_position.x -= 1;
                }
            }
            KeyEvent { code: KeyCode::Right, .. } 
            | KeyEvent { code: KeyCode::Char('d'), .. } => {
                if self.cursor_position.x < (self.terminal_size.0 as usize).saturating_sub(1) {
                    self.cursor_position.x += 1;
                }
            }
            _ => (),
        }
    }

    // Renders the TUI
    // Clears the screen, draws rows, and positions the cursor
    fn refresh_screen(&self) -> io::Result<()> {
        // Queue commands to the buffer to avoid flickering
        // \x1b[?25l hides cursor, \x1b[?25h shows it
        execute!(stdout(), cursor::Hide, cursor::MoveTo(0, 0))?;

        self.draw_rows();

        // Move cursor to the tracked position
        execute!(
            stdout(),
            cursor::MoveTo(self.cursor_position.x as u16, self.cursor_position.y as u16),
            cursor::Show
        )?;
        
        stdout().flush()
    }

    // Draws '~' for empty lines, thank you Vim
    fn draw_rows(&self) {
        let height = self.terminal_size.1;
        
        for i in 0..height {
            print!("~");
            
            // Clear the rest of the line to clear artifacts
            execute!(stdout(), Clear(ClearType::UntilNewLine)).unwrap();
            
            // Move to next line (except for the last one to avoid scrolling)
            if i < height - 1 {
                print!("\r\n");
            }
        }
    }

    // Panic handler: cleans up terminal before crashing
    fn die(&self, e: &std::io::Error) {
        let _ = disable_raw_mode();
        execute!(stdout(), Clear(ClearType::All)).unwrap();
        panic!("{}", e);
    }
}

fn main() {
    let mut editor = Editor::default();
    if let Err(e) = editor.run() {
        eprintln!("Error: {}", e);
    }
}