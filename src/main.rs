use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{self, Write},
    path::PathBuf,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
};

const MAX_GUESSES: usize = 6;
const DEFAULT_WORD_LEN: usize = 5;
const MIN_WORD_LEN: usize = 3;
const MAX_WORD_LEN: usize = 8;

type Dictionary = BTreeMap<usize, Vec<String>>;

fn main() -> Result<()> {
    let dictionary = load_dictionary()?;
    let word_length = prompt_word_length(&dictionary)?;
    let word_pool = dictionary
        .get(&word_length)
        .cloned()
        .context("No words available for the chosen length.")?;

    let mut app = App::new(word_length, word_pool);
    let mut terminal = setup_terminal()?;

    let run_result = run_app(&mut terminal, &mut app);
    cleanup_terminal(&mut terminal)?;
    run_result
}

fn load_dictionary() -> Result<Dictionary> {
    let candidates = [
        PathBuf::from("dictionary.txt"),
        PathBuf::from("../dictionary.txt"),
        PathBuf::from("../../dictionary.txt"),
    ];

    let contents = candidates
        .into_iter()
        .find_map(|path| fs::read_to_string(&path).ok())
        .context("Unable to locate dictionary.txt. Place it alongside the binary or in the project root.")?;

    Ok(parse_dictionary(&contents))
}

fn parse_dictionary(contents: &str) -> Dictionary {
    let mut map: Dictionary = BTreeMap::new();

    for raw in contents.lines() {
        let word = raw.trim();
        if word.is_empty() {
            continue;
        }
        if !word.chars().all(|c| c.is_ascii_alphabetic()) {
            continue;
        }

        let upper = word.to_ascii_uppercase();
        let len = upper.len();

        if len < MIN_WORD_LEN || len > MAX_WORD_LEN {
            continue;
        }

        let entry = map.entry(len).or_default();
        if !entry.contains(&upper) {
            entry.push(upper);
        }
    }

    map
}

fn prompt_word_length(dictionary: &Dictionary) -> Result<usize> {
    let mut available_lengths: Vec<usize> = dictionary
        .iter()
        .filter(|(_, words)| !words.is_empty())
        .map(|(&len, _)| len)
        .collect();

    if available_lengths.is_empty() {
        bail!("Dictionary does not contain any usable words.");
    }

    available_lengths.sort_unstable();
    let default = if available_lengths.contains(&DEFAULT_WORD_LEN) {
        DEFAULT_WORD_LEN
    } else {
        *available_lengths.first().unwrap()
    };

    println!(
        "Select word length (default {default}). Available lengths: {}",
        available_lengths
            .iter()
            .map(|len| len.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    print!("Enter word length and press Enter: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let trimmed = input.trim();

    if trimmed.is_empty() {
        Ok(default)
    } else {
        let parsed = trimmed
            .parse::<usize>()
            .context("Please enter a number for word length.")?;

        if available_lengths.contains(&parsed) {
            Ok(parsed)
        } else {
            bail!("Length {parsed} is not available in the dictionary selection.");
        }
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn cleanup_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(50);

    while !app.should_quit {
        terminal.draw(|frame| draw_ui(frame, app))?;

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key) => app.on_key(key),
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
    }

    Ok(())
}

fn draw_ui(frame: &mut Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.size());

    render_instructions(frame, chunks[0], app.word_length);
    render_board(frame, chunks[1], app);
    render_status(frame, chunks[2], app);
}

fn render_instructions(frame: &mut Frame<'_>, area: Rect, word_length: usize) {
    let instructions = Paragraph::new(vec![
        Line::from(Span::styled(
            "Rust TUI Wordle",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(format!(
            "Type letters to build a {word_length}-letter guess."
        )),
        Line::from("Enter submits, Backspace deletes."),
        Line::from("Esc or Ctrl+C exits the game."),
        Line::from("Colors: green=correct spot, yellow=present, gray=absent."),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: true })
    .block(Block::default().title("How To Play").borders(Borders::ALL));

    frame.render_widget(instructions, area);
}

fn render_board(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let mut rows = Vec::with_capacity(MAX_GUESSES);

    for row_idx in 0..MAX_GUESSES {
        let (letters, evals, is_active_row) = if row_idx < app.guesses.len() {
            (
                app.guesses[row_idx].chars().collect::<Vec<_>>(),
                Some(app.evaluations[row_idx].clone()),
                false,
            )
        } else if row_idx == app.guesses.len() && app.is_accepting_input() {
            (app.current_guess.chars().collect::<Vec<_>>(), None, true)
        } else {
            (Vec::new(), None, false)
        };

        let mut cells = Vec::with_capacity(app.word_length);
        for col in 0..app.word_length {
            let ch = letters.get(col).copied().unwrap_or(' ');
            let display = format!(" {} ", ch);
            let mut cell = Cell::from(display);

            if let Some(ref states) = evals {
                if let Some(state) = states.get(col) {
                    cell = cell.style(style_for_state(*state));
                }
            } else if is_active_row && col < letters.len() {
                cell = cell.style(Style::default().fg(Color::Cyan));
            } else {
                cell = cell.style(Style::default().fg(Color::DarkGray));
            }

            cells.push(cell);
        }

        rows.push(Row::new(cells));
    }

    let constraints = vec![Constraint::Length(4); app.word_length];
    let table = Table::new(rows, constraints)
        .block(Block::default().title("Board").borders(Borders::ALL))
        .column_spacing(1)
        .style(Style::default().fg(Color::White));

    frame.render_widget(table, area);
}

fn render_status(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let style = match app.state {
        GameState::Playing => Style::default().fg(Color::White),
        GameState::Won => Style::default().fg(Color::Green),
        GameState::Lost => Style::default().fg(Color::Red),
    };

    let status = Paragraph::new(app.status.as_str())
        .style(style)
        .alignment(Alignment::Center)
        .block(Block::default().title("Status").borders(Borders::ALL));

    frame.render_widget(status, area);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LetterState {
    Correct,
    Present,
    Absent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    Playing,
    Won,
    Lost,
}

struct App {
    word_length: usize,
    target_word: String,
    valid_words: HashSet<String>,
    guesses: Vec<String>,
    evaluations: Vec<Vec<LetterState>>,
    current_guess: String,
    status: String,
    state: GameState,
    should_quit: bool,
}

impl App {
    fn new(word_length: usize, word_pool: Vec<String>) -> Self {
        let mut rng = thread_rng();
        let target_word = word_pool
            .choose(&mut rng)
            .cloned()
            .unwrap_or_else(|| "RUSTY".to_string());
        let valid_words = word_pool.into_iter().collect::<HashSet<_>>();

        Self {
            word_length,
            target_word,
            valid_words,
            guesses: Vec::new(),
            evaluations: Vec::new(),
            current_guess: String::new(),
            status: format!("Type a {word_length}-letter word and press Enter."),
            state: GameState::Playing,
            should_quit: false,
        }
    }

    fn is_accepting_input(&self) -> bool {
        self.state == GameState::Playing
    }

    fn on_key(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc
            || (matches!(key.code, KeyCode::Char('c'))
                && key.modifiers.contains(KeyModifiers::CONTROL))
        {
            self.should_quit = true;
            return;
        }

        if !self.is_accepting_input() {
            return;
        }

        match key.code {
            KeyCode::Char(c)
                if !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if c.is_ascii_alphabetic() && self.current_guess.len() < self.word_length {
                    self.current_guess.push(c.to_ascii_uppercase());
                }
            }
            KeyCode::Backspace => {
                self.current_guess.pop();
            }
            KeyCode::Enter => self.submit_current_guess(),
            _ => {}
        }
    }

    fn submit_current_guess(&mut self) {
        if self.current_guess.len() != self.word_length {
            self.status = format!("Guess must be exactly {} letters.", self.word_length);
            return;
        }

        if !self.valid_words.contains(&self.current_guess) {
            self.status = format!("{} is not in the dictionary.", self.current_guess);
            return;
        }

        let guess = self.current_guess.clone();
        let evaluation = evaluate_guess(&self.target_word, &guess);

        self.guesses.push(guess.clone());
        self.evaluations.push(evaluation);
        self.current_guess.clear();

        if guess == self.target_word {
            self.state = GameState::Won;
            self.status = format!(
                "You found {}! Press Esc or Ctrl+C to exit.",
                self.target_word
            );
            return;
        }

        if self.guesses.len() >= MAX_GUESSES {
            self.state = GameState::Lost;
            self.status = format!(
                "Out of guesses! The word was {}. Press Esc or Ctrl+C to exit.",
                self.target_word
            );
            return;
        }

        let remaining = MAX_GUESSES - self.guesses.len();
        self.status = format!("{remaining} guesses remaining.");
    }
}

fn evaluate_guess(target: &str, guess: &str) -> Vec<LetterState> {
    let target_chars: Vec<char> = target.chars().collect();
    let guess_chars: Vec<char> = guess.chars().collect();
    let len = target_chars.len();
    let mut result = vec![LetterState::Absent; len];
    let mut remaining = [0u8; 26];

    for i in 0..len {
        if guess_chars[i] == target_chars[i] {
            result[i] = LetterState::Correct;
        } else {
            if let Some(idx) = char_to_index(target_chars[i]) {
                remaining[idx] += 1;
            }
        }
    }

    for i in 0..len {
        if result[i] == LetterState::Correct {
            continue;
        }

        if let Some(idx) = char_to_index(guess_chars[i]) {
            if remaining[idx] > 0 {
                result[i] = LetterState::Present;
                remaining[idx] -= 1;
            }
        }
    }

    result
}

fn char_to_index(c: char) -> Option<usize> {
    let upper = c.to_ascii_uppercase();
    if upper.is_ascii_uppercase() {
        Some((upper as u8 - b'A') as usize)
    } else {
        None
    }
}

fn style_for_state(state: LetterState) -> Style {
    match state {
        LetterState::Correct => Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD),
        LetterState::Present => Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        LetterState::Absent => Style::default().fg(Color::White).bg(Color::DarkGray),
    }
}
