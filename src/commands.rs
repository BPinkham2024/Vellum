use crate::editor::{Editor, StatusMessage, Position};

pub fn execute_command(editor: &mut Editor, command: &str) -> Result<(), std::io::Error> {
    // I want edits from commands to be able to be reversed/redone
    editor.document.snapshot();

    // Search and replace logic (vim syntax so all one word not split by whitespace)
    if command.starts_with("s/") {
        let parts: Vec<&str> = command.split('/').collect();

        // Expectation is ["s", "old_text", "new_text"]
        if parts.len() >= 3 {
            let target = parts[1];
            let replacement = parts[2];

            let count = editor.document.replace(target, replacement);
            editor.status_message = StatusMessage::from(format!("Replaced '{}' in {} lines", target, count));

            // Saftey clamp for cursor (pulls back to end of line)
            let current_len = editor.line_length(editor.cursor_position.y);
            if editor.cursor_position.x > current_len {
                editor.cursor_position.x = current_len;
            }
        } else {
            editor.status_message = StatusMessage::from("Usage: s/old/new".to_string());
        }
        return Ok(());
    }

    // Standard commands
    let parts: Vec<&str> = command.split_whitespace().collect();
    if parts.is_empty() { return Ok(());}

    match parts[0] {
        "q" => editor.should_quit = true,
        "w" => {
            if let Err(e) = editor.document.save() {
                editor.status_message = StatusMessage::from(format!("Error: {}", e));
            } else {
                editor.status_message = StatusMessage::from("File saved.".to_string());
            }
        },
        "!w" => {
            if parts.len() > 1 {
                let new_name = parts[1].to_string();
                editor.document.filename = Some(new_name);
                editor.document.save()?;
                editor.status_message = StatusMessage::from("File saved as new name.".to_string());
            } else {
                editor.status_message = StatusMessage::from("Error: !w requires a filename".to_string());
            }
        },
        "head" => {
            if parts.len() > 1 {
                if let Ok(level) = parts[1].parse::<usize>() {
                    editor.document.set_header(editor.cursor_position.y, level);
                }
            }
        },
        "bold" => wrap_word(editor, "**"),
        "italic" => wrap_word(editor, "*"),
        "t" => {
            if parts.len() > 1 {
                if let Ok(count) = parts[1].parse::<usize>() {
                    editor.document.indent(editor.cursor_position.y, count);
                    editor.cursor_position.x += count * 4;
                }
            }
        },
        "find" => {
            if parts.len() > 1 {
                let query = parts[1];
                find_next(editor, query);
            }
        },
        "ln" => {
            editor.show_line_numbers = !editor.show_line_numbers;
            editor.status_message = StatusMessage::from(format!("Line numbers: {}", editor.show_line_numbers));
        },
        "dd" => {
            editor.document.delete_line(editor.cursor_position.y);

            // Fix cursor if deleted bottom line
            if editor.cursor_position.y >= editor.document.len() {
                editor.cursor_position.y = editor.document.len().saturating_sub(1);
            }
            let current_len = editor.line_length(editor.cursor_position.y);
            if editor.cursor_position.x > current_len {
                editor.cursor_position.x = current_len;
            }
        },
        "d" => {
            let count = if parts.len() > 1 { parts[1].parse::<usize>().unwrap_or(1) } else { 1 };
            delete_words(editor, count, true);
        }
        "db" => {
            let count = if parts.len() > 1 { parts[1].parse::<usize>().unwrap_or(1) } else { 1 };
            delete_words(editor, count, false);
        }
        _ => editor.status_message = StatusMessage::from(format!("Unknown command: {}", command)),
    }
    Ok(())
}

// Helper functions

// Wrap word for bold and italics
fn wrap_word(editor: &mut Editor, wrapper: &str) {
    let y = editor.cursor_position.y;
    let x = editor.cursor_position.x;

    if y < editor.document.len() {
        let line = editor.document.rope.line(y).to_string();
        let chars: Vec<char> = line.chars().collect();

        // Find start of word
        let mut start = 0;
        for i in (0..x).rev() {
            if i < chars.len() && chars[i] == ' ' {
                start = i + 1;
                break;
            }
        }
        
        // Find end of word
        let mut end = chars.len();
        for i in x..chars.len() {
            if chars[i] == ' ' {
                end = i;
                break;
            }
        }

        // Since we are mutating the line, document needs to be called
        editor.document.insert_str(&Position { x: end, y }, wrapper); // Suffex first so we don't mess with indices for prefix insertion
        editor.document.insert_str(&Position { x: start, y }, wrapper);

        // Move cursor to end of word
        editor.cursor_position.x = end + (wrapper.len() * 2);
    }
}

fn find_next(editor: &mut Editor, query: &str) {
    let start_y = editor.cursor_position.y;
    let mut y = start_y;

    loop {
        if y < editor.document.len() {
            let line = editor.document.rope.line(y).to_string();
            if let Some(byte_idx) = line.find(query) {

                // Convert byte index to char index
                let char_idx = line[..byte_idx].chars().count();

                // Check if found after current cursor position only if cursor on y = stary_y
                if y != start_y || char_idx > editor.cursor_position.x {
                    editor.cursor_position.x = char_idx;
                    editor.cursor_position.y = y;
                    editor.status_message = StatusMessage::from(format!("Found: {}", query));
                    return;
                }
            }
        }

        y += 1;
        // Wrap to top if hitting bottom
        if y >= editor.document.len() {
            y = 0;
        }
        // If word not found in full wrapping of doc, stop
        if y == start_y {
            editor.status_message = StatusMessage::from(format!("Not found: {}", query));
            return;
        }
    }
}

fn delete_words(editor: &mut Editor, count: usize, forward: bool) {
    if count == 0 { return; }
    let y = editor.cursor_position.y;
    if y >= editor.document.len() { return; }

    // Convert 2d cursor to 1d index
    let cursor_idx = editor.document.rope.line_to_char(y) + editor.cursor_position.x;
    let rope = &editor.document.rope;
    let max_chars = rope.len_chars();
    if cursor_idx >= max_chars { return; } 

    let mut start_idx = cursor_idx;
    let mut end_idx = cursor_idx;

    // Find start of current word
    while start_idx > 0 {
        if rope.char(start_idx - 1).is_whitespace() { break; }
        start_idx -= 1;
    }

    // Find end of current word
    while end_idx < max_chars {
        if rope.char(end_idx).is_whitespace() { break; }
        end_idx += 1;
    }

    let mut words_left = count.saturating_add(1);

    // Scan forward or backwards for additional words
    if forward {
        while words_left > 0 && end_idx < max_chars {
            while end_idx < max_chars && rope.char(end_idx).is_whitespace() { end_idx += 1; }
            while end_idx < max_chars && !rope.char(end_idx).is_whitespace() { end_idx += 1; }
            words_left -= 1;
        }
    } else {
        while words_left > 0 && start_idx > 0 {
            while start_idx > 0 && rope.char(start_idx - 1).is_whitespace() { start_idx -= 1; }
            while start_idx > 0 && rope.char(start_idx - 1).is_whitespace() { start_idx -= 1; }
            words_left -= 1;
        }
    }

    // Nuke range and move cursor back
    editor.document.delete_char_range(start_idx, end_idx);
    editor.cursor_position.y = editor.document.rope.char_to_line(start_idx);
    editor.cursor_position.x = start_idx - editor.document.rope.line_to_char(editor.cursor_position.y);
}