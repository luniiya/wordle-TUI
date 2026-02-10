# Wordle TUI

Wordle TUI is a terminal user interface game inspired by the original Wordle. It lets you pick a word length (3–8 letters), play with color-coded feedback, and runs entirely in the terminal using Crossterm and Ratatui.

## Prerequisites
- Rust 1.76+ with `cargo` (install via <https://rustup.rs/>).

## Install
1. Clone or download this repository.
   ```bash
   git clone <your-fork-or-repo-url>
   cd wordle-tui
   ```
2. Build and install the binary locally (puts it in `~/.cargo/bin`):
   ```bash
   cargo install --path .
   ```
   Alternatively, build in place without installing:
   ```bash
   cargo build --release
   ```
3. Ensure the dictionary asset is available next to the binary at runtime. If you use `cargo install`, create an `assets` folder beside the installed binary and copy the dictionary there:
   ```bash
   mkdir -p ~/.cargo/bin/assets
   cp assets/dictionary.txt ~/.cargo/bin/assets/
   ```
   This makes the file available as `~/.cargo/bin/assets/dictionary.txt`, which the game will detect automatically.

## Run
- From the project directory:
  ```bash
  cargo run --release
  ```
- From a globally installed binary:
  ```bash
  wordle-tui
  ```

At startup you’ll be asked for the desired word length. Press Enter to accept the default (5) or enter any supported length shown in the prompt.

## Dictionary placement
The game looks for `dictionary.txt` in these locations, in order:
- `assets/dictionary.txt` relative to the current directory
- `../assets/dictionary.txt` or `../../assets/dictionary.txt` (useful for running from a nested path)
- `assets/dictionary.txt` next to the executable

If none of those paths exist, the program exits with an error. Keep `dictionary.txt` with the binary or adjust your working directory accordingly.

## Development
- Format/check with:
  ```bash
  cargo fmt
  cargo clippy --all-targets --all-features
  ```
- Run tests (if any are added):
  ```bash
  cargo test
  ```
