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

        // Find start of word
        let start = line[..x].rfind(' ').map(|i| i + 1).unwrap_or(0);
        
        // Find end of word
        let end = line[x..].find(' ').map(|i| x + i).unwrap_or_else(|| {
            editor.line_length(y)
        });

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
            if let Some(x) = line.find(query) {
                // Check if found after current cursor position only if cursor on y = stary_y
                if y != start_y || x > editor.cursor_position.x {
                    editor.cursor_position.x = x;
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