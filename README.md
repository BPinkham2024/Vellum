# Vellum Text Editor

Vellum is a lightweight, terminal-based text editor written in Rust. It is designed to be a fast, modal editor specifically for writing Markdown notes. It features syntax highlighting, a command system similar to Vim, and zero-flicker rendering.

## Features

* **Zero-Flicker Rendering:** Uses double-buffering for smooth UI updates.
* **Markdown Syntax Highlighting:**
    * Headers (#, ##, ###)
    * Lists (-, +, *, 1.)
    * **Bold** and *Italic* text support
* **Command System:** Vim-like command mode for complex operations.
* **Smart Editing:**
    * Auto-indentation aware
    * Word wrapping helpers
* **File I/O:** Open, Save, and "Save As" functionality.

## Installation & Usage

You must have [Rust and Cargo](https://rustup.rs/) installed.

1.  **Clone the repository:**
    ```bash
    git clone [https://github.com/BPinkham2024/vellum.git](https://github.com/BPinkham2024/vellum.git)
    cd vellum
    ```

2.  **Run the editor:**
    ```bash
    cargo run
    ```

3.  **Open a specific file:**
    ```bash
    cargo run notes.md
    ```

## Keybindings

### Navigation
| Key | Action |
| :--- | :--- |
| `Arrows` | Move Cursor |
| `Enter` | New Line |
| `Backspace` | Delete Character |

### System Shortcuts
| Key | Action |
| :--- | :--- |
| `Ctrl` + `Q` | Quit Vellum |
| `Ctrl` + `S` | Save File |
| `Ctrl` + `:` | **Enter Command Mode** |

---

## Command Mode

Press `Ctrl` + `:` to enter Command Mode. The status bar will change to `COMMAND:`. Type your command and press `Enter`.

### File Operations
* `q` : Quit the editor.
* `w` : Save the current file.
* `!w <filename>` : Save the file as a new name (e.g., `!w homework.md`).

### Formatting & Editing
* `head <1-3>` : Converts the current line into a Header (e.g., `head 1` makes it H1).
* `bold` : Wraps the word under the cursor in `**bold**` tags.
* `italic` : Wraps the word under the cursor in `*italic*` tags.
* `t <n>` : Indents the current line by `<n>` tabs.
* `find <text>` : Jumps the cursor to the next occurrence of `<text>`.

## Project Structure

This project follows a modular MVC (Model-View-Controller) architecture:

* **`src/main.rs`**: Entry point.
* **`src/editor.rs` (Controller)**: Handles input loops, keypress logic, and mode switching.
* **`src/terminal.rs` (View)**: Wraps `crossterm` to handle low-level terminal drawing and buffer flushing.
* **`src/document.rs` (Model)**: Manages the file data, rows, and dirty state.
* **`src/row.rs`**: Represents a single line of text and handles character logic.
* **`src/highlighting.rs`**: The lexer/scanner that assigns colors to Markdown syntax.

## Future Roadmap

* [ ] Search and Replace
* [ ] Line Number Rendering (Gutter)
* [ ] Config file support (`.vellumrc`)
* [ ] Infinite scrolling
* [ ] Mouse support
* [ ] Markdown Rendering
* [ ] Much more