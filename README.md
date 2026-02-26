# Vellum

Vellum is a fast, minimalist terminal text editor written in Rust. It's built to be lightweight and get out of your way, while still packing modern features under the hood.

## Features

* **Rope Data Structure:** Powered by the `ropey` crate to handle massive files without breaking a sweat.
* **Tree-Sitter Highlighting:** Real-time, structurally aware Markdown syntax highlighting.
* **Smart Word Wrapping:** Visual word wrapping that correctly maps cursor movements so you don't skip over text.
* **Modal Editing:** Built with Normal, Insert, and Command modes.
* **Safe Undo/Redo:** Snapshot-based undo stack capped at 100 states so it doesn't eat your RAM.

## Keybindings

**Normal Mode**
* `i` - Enter Insert Mode
* `w` / `a` / `s` / `d` or Arrow Keys - Move cursor
* `y` - Copy the current line to clipboard
* `:` - Enter Command Mode
* `Esc` - Return to Normal Mode

**Insert Mode**
* Type to insert text.
* `Esc` - Return to Normal Mode

## Commands

Type `:` in Normal Mode to open the command bar.

* `w` - Save the file
* `!w <filename>` - Save as a new file
* `q` - Quit Vellum
* `s/old/new` - Search and replace
* `find <query>` - Jump to the next match
* `ln` - Toggle line numbers
* `head <level>` - Turn the current line into a Markdown header (e.g. `head 2` for `##`)
* `bold` / `italic` - Wrap the current word in Markdown formatting
* `t <count>` - Indent the current line by `<count>` spaces
* `dd` - Delete the entire current line
* `d <#>` - Delete `<#>` words forward (e.g. `d 3`)
* `db <#>` - Delete `<#>` words backward

## Installation

Clone the repository and build with Cargo:

```bash
git clone https://github.com/bpinkham2024/vellum.git
cd vellum
cargo build --release
