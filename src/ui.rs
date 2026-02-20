use crate::editor::{Editor, Mode};
use crossterm::style::Color;
use std::time::{Duration, Instant};

const WRAP_PREFIX: &str = " >"; // Visual indicator for wrapped text (will not show in saved files)

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
        
        // 4. Put the cursor back where it belongs and with offset (updated)
        let gutter = gutter_width(editor);
        let text_width = (editor.terminal.size().width as usize).saturating_sub(gutter);

        let (visual_x, visual_y) = get_visual_cursor(editor, text_width);

        editor.terminal.cursor_position(
            visual_x + gutter as u16,
            visual_y
        );
    }

    // 5. Show the cursor again
    editor.terminal.cursor_show();
    
    // 6. THE BIG FLUSH
    editor.terminal.flush()
}

fn get_visual_cursor(editor: &mut Editor, text_width: usize) -> (u16, u16) {
    if text_width == 0 { return (0, 0); }
    let mut visual_y = 0;

    // Calc how many visual lines are taken up by rows above the cursor
    for doc_y in editor.row_offset..editor.cursor_position.y {
        if let Some(row) = editor.document.row(doc_y) {
            let len = row.len();
            if len <= text_width {
                visual_y += 1;
            } else {
                let remaining = len.saturating_sub(text_width);
                let wrap_width = text_width.saturating_sub(WRAP_PREFIX.len());
                let extra_lines = (remaining as f32 / std::cmp::max(1, wrap_width) as f32).ceil() as usize;
                visual_y += 1 + extra_lines;
            }
        } else {
            visual_y += 1;
        }
    }

    // Calc x offset and remaining y offset for current row
    let mut visual_x = editor.cursor_position.x;
    if visual_x >= text_width {
        let remaining_x = visual_x.saturating_sub(text_width);
        let wrap_width = text_width.saturating_sub(WRAP_PREFIX.len());
        let safe_wrap_width = std::cmp::max(1, wrap_width);

        let extra_lines = remaining_x / safe_wrap_width;

        visual_y += extra_lines + 1; // Drop down for each wrap
        visual_x = WRAP_PREFIX.len() + (remaining_x % safe_wrap_width); // Shift past the indicator
    }

    (visual_x as u16, visual_y as u16)
}

// Helper to calculate gutter width
fn gutter_width(editor: &mut Editor) -> usize {
    if !editor.show_line_numbers {
        return 0;
    }

    // Adds 2 for padding and pipe
    editor.document.len().to_string().len() + 2
}

fn draw_gutter(terminal: &mut crate::terminal::Terminal, show_line_numbers: bool, gutter: usize, doc_row: usize, is_wrapped: bool) {
    if !show_line_numbers { return; }
    terminal.set_fg_color(Color::DarkGrey);
    
    if !is_wrapped {
        let num_str = format!("{:>w$} |", doc_row + 1, w = gutter.saturating_sub(2));
        terminal.print(&num_str);
    } else {
        let empty_str = format!("{:>w$} |", "", w = gutter.saturating_sub(2));
        terminal.print(&empty_str);
    }
    terminal.reset_colors();
}

// Draws each row
fn draw_rows(editor: &mut Editor) {
    let height = editor.terminal.size().height as usize;
    let width = editor.terminal.size().width as usize;
    let gutter = gutter_width(editor);
    let text_width = width.saturating_sub(gutter);

    let mut terminal_row = 0;
    let mut doc_row = editor.row_offset;

    while terminal_row < height - 2 && doc_row < editor.document.len() { // subtracting 2 allows for the status and message bar
        if let Some(row) = editor.document.row(doc_row) {
            let row_len = row.len();
            let mut char_index = 0;
            let mut is_wrapped = false;

            if row_len == 0 {
                editor.terminal.clear_current_line();
                draw_gutter(&mut editor.terminal, editor.show_line_numbers, gutter, doc_row, is_wrapped);
                editor.terminal.print("\r\n");
                terminal_row += 1;
                doc_row += 1;
                continue;
            }

            // Chunk text to fit screen
            while char_index < row_len && terminal_row < height - 2 {
                let current_width = if is_wrapped {
                    text_width.saturating_sub(WRAP_PREFIX.len())
                } else {
                    text_width
                };

                let end_index = std::cmp::min(char_index + current_width, row_len);
                let chunk = row.render(char_index, end_index);

                editor.terminal.clear_current_line();
                draw_gutter(&mut editor.terminal, editor.show_line_numbers, gutter, doc_row, is_wrapped);


                if is_wrapped {
                    editor.terminal.set_fg_color(Color::DarkGrey);
                    editor.terminal.print(WRAP_PREFIX);
                    editor.terminal.reset_colors();
                }

                // Render colored chars
                for (i, c) in chunk.chars().enumerate() {
                    if let Some(hl_type) = row.highlighting.get(char_index + i) {
                        editor.terminal.set_fg_color(hl_type.to_color());
                    } else {
                        editor.terminal.set_fg_color(Color::Reset);
                    }
                    editor.terminal.print(&c.to_string());
                }
                
                editor.terminal.reset_colors();
                editor.terminal.print("\r\n");

                char_index = end_index;
                is_wrapped = true;
                terminal_row += 1;
            }
        }
        doc_row += 1;
    }

    // Fill empty screen with ~, thank you vim
    while terminal_row < height - 2 {
        editor.terminal.clear_current_line();
        if editor.show_line_numbers {
            let empty_str = format!("{:>w$} |", "~", w = gutter.saturating_sub(2));
            editor.terminal.set_fg_color(Color::DarkGrey);
            editor.terminal.print(&empty_str);
            editor.terminal.reset_colors();
        } else {
            editor.terminal.print("~");
        }
        editor.terminal.print("\r\n");
        terminal_row += 1;
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