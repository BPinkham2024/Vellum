use crate::editor::{Editor, Mode};
use crossterm::style::Color;
use std::time::{Duration, Instant};

// Renders the TUI
pub fn refresh_screen(editor: &mut Editor) -> Result<(), std::io::Error> {
    editor.scroll();

    // 1. Hide the cursor so it doesn't jump around while being drawn
    editor.terminal.cursor_hide();
    
    // 2. Move to top-left to start drawing
    editor.terminal.cursor_position(0, 0);

    // 3. Queue up the drawing commands
    if editor.should_quit {
        editor.terminal.clear_screen();
        editor.terminal.print("Goodbye.\r\n");
    } else {
        draw_rows(editor);
        draw_status_bar(editor);
        draw_message_bar(editor);
        
        // 4. Put the cursor back where it belongs and with offset
        let offset_x = gutter_width(editor) as u16;
        let screen_y = (editor.cursor_position.y - editor.row_offset) as u16;

        editor.terminal.cursor_position(
            editor.cursor_position.x as u16 + offset_x, 
            screen_y
        );
    }

    // 5. Show the cursor again
    editor.terminal.cursor_show();
    
    // 6. THE BIG FLUSH
    editor.terminal.flush()
}

// Helper to calculate gutter width
fn gutter_width(editor: &mut Editor) -> usize {
    if !editor.show_line_numbers {
        return 0;
    }

    // Adds 2 for padding and pipe
    editor.document.len().to_string().len() + 2
}

// Draws each row
fn draw_rows(editor: &mut Editor) {
    let height = editor.terminal.size().height;
    let width = editor.terminal.size().width as usize;
    let gutter = gutter_width(editor);

    for terminal_row in 0..height  - 2 { // subtracting 2 allows for the status and message bar
        // Clear the line so old text doesn't linger
        editor.terminal.clear_current_line();

        let doc_row = terminal_row as usize + editor.row_offset;

        // Draw line numbers
        if editor.show_line_numbers {
            editor.terminal.set_fg_color(Color::DarkGrey);
            if doc_row < editor.document.len() {
                let num_str = format!("{:>w$} |", doc_row + 1, w = gutter - 2);
                editor.terminal.print(&num_str);
            } else {
                let empty_str = format!("{:>w$} |", "~", w = gutter - 2);
                editor.terminal.print(&empty_str);
            }
            editor.terminal.reset_colors();
        }

        // If the row exists in the document, render it
        if let Some(row) = editor.document.row(doc_row) {

            let start = 0;
            let end = width.saturating_sub(gutter);
            let render_string = row.render(start, end);

            for (i, c) in render_string.chars().enumerate() {
                if let Some(hl_type) = row.highlighting.get(i) {
                    let color = hl_type.to_color();
                    editor.terminal.set_fg_color(color);
                } else {
                    editor.terminal.set_fg_color(Color::Reset);
                }

                editor.terminal.print(&c.to_string());
            }

            editor.terminal.reset_colors();
        } else if !editor.show_line_numbers {
            // ~ for empty lines, thank you vim
            editor.terminal.print("~");
        }
        
        if terminal_row < height - 1 {
            editor.terminal.print("\r\n");
        }
    }
}

fn draw_status_bar(editor: &mut Editor) {
    let mut status;
    let width = editor.terminal.size().width as usize;
    let modified_indicator = if editor.document.is_dirty() { "(modified)" } else { "" };
    
    if let Mode::Command(cmd) = &editor.mode {
        status = format!("COMMAND: {}_", cmd);
    } else {
        let filename = editor.document.filename.clone().unwrap_or_else(|| "[No Name]".to_string());
        status = format!("{} - {} lines {}", filename, editor.document.len(), modified_indicator);
    }
    
    let line_indicator = format!("{}/{}", editor.cursor_position.y + 1, editor.document.len());
    let len = status.len() + line_indicator.len();
    
    if width > len {
        status.push_str(&" ".repeat(width - len));
    }
    // Truncation if line is too long
    status = format!("{}{}", status, line_indicator);
    status.truncate(width);

    // Styling for status
    editor.terminal.set_bg_color(Color::White);
    editor.terminal.set_fg_color(Color::Black);
    editor.terminal.print(&status);

    // Reset colors
    editor.terminal.reset_colors();
    editor.terminal.print("\r\n");


}

fn draw_message_bar(editor: &mut Editor) {
    editor.terminal.clear_current_line();
    let msg = &editor.status_message;
    if Instant::now() - msg.time < Duration::from_secs(5) {
        let mut text = msg.text.clone();
        text.truncate(editor.terminal.size().width as usize);
        editor.terminal.print(&text);
    }
}