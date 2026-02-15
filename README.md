# Wordle TUI

Wordle TUI is a terminal user interface game inspired by the original Wordle. It lets you pick a word length (3–8 letters), play with color-coded feedback, and runs entirely in the terminal using Crossterm and Ratatui.
## Overview
<img width="2560" height="1440" alt="yes" src="https://github.com/user-attachments/assets/2dcbec2f-a715-46eb-bac0-65c6b3813eaf" />

Wordle-TUI run from the terminal without an internet connection, it use a TUI made in rust and works well with pretty pywal coloring if any rice enjoyer are reading this.
You can also replace the dictionary by anything making it compatible with any language.

It also support multiple word length, when starting the game you are prompted with this :
<img width="1378" height="503" alt="image" src="https://github.com/user-attachments/assets/cb9296a8-e672-4fdc-b6aa-0d2cd87ff6dc" />

## Insallation
### Prerequisites
- Rust 1.76+ with `cargo` (install via <https://rustup.rs/>).

### Install
1. Clone or download this repository.
   ```bash
   git clone https://github.com/luniiya/wordle-TUI.git
   cd wordle-TUI
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

### Add `cargo` bin to your `PATH`
To run the installed binary directly, ensure `~/.cargo/bin` is on your `PATH`.
- Bash/Zsh (adds it for new shells):
  ```bash
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc  # or ~/.zshrc
  source ~/.bashrc  # reloads your shell config
  ```
- Fish shell:
  ```bash
  set -U fish_user_paths $HOME/.cargo/bin $fish_user_paths
  ```
For a single session without editing config files, run:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Run
- From the project directory:
  ```bash
  cargo run --release
  ```
- From a globally installed binary:
  ```bash
  wordle-tui
  ```

At startup you’ll be asked for the desired word length. Press Enter to accept the default (5) or enter any supported length shown in the prompt.

### Dictionary placement
The game looks for `dictionary.txt` in these locations, in order:
- `assets/dictionary.txt` relative to the current directory
- `../assets/dictionary.txt` or `../../assets/dictionary.txt` (useful for running from a nested path)
- `assets/dictionary.txt` next to the executable

If none of those paths exist, the program exits with an error. Keep `dictionary.txt` with the binary or adjust your working directory accordingly.

## Contributing
feel free to open a pull request if you want to add something / fix a bug, since this was just a quick codex experiment, there's definitely room for polish. 
If the changes look good and everything works, i'll definitely merge them in
