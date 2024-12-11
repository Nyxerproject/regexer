use clap::{Arg, ArgAction, Command};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = Command::new("regexer")
        .version("0.1.0")
        .about("A regex CLI/TUI tool for parsing and testing regular expressions.")
        .long_about(
"regexer is a command-line/text-user interface tool for parsing and testing regular expressions.

If run without the interactive mode:
  - If no arguments are provided, it will exit.
  - If not provided with both PATTERN and TEXT (and no -f), it will exit.
  - If provided with -f FILE, it should not take a TEXT argument, only PATTERN is required. Otherwise, it will exit.

If run in interactive mode (-i):
  - If one of PATTERN, TEXT, or FILE is provided, the TUI fields will be pre-filled and you can add or modify as needed.
  - If no PATTERN, TEXT, or FILE is provided, you will be prompted to input both PATTERN and TEXT interactively.
"
        )
        .override_usage("regexer [OPTIONS] [PATTERN] [TEXT]")
        .after_help(
"Examples:
  regexer \"pattern\" \"text to search\"
  regexer -f input.txt \"pattern\"
  regexer -i
  regexer -i \"pattern\" \"text\"
  regexer -i -f input.txt \"pattern\"

Exit Codes:
  0   Success
  1   Error in regex pattern
  2   File not found
  3   Other errors

For more information, visit:
  GitHub: https://github.com/Nyxerproject
  Documentation: <link to docs>"
        )
        .arg(
            Arg::new("pattern")
                .help("The regular expression pattern to match")
                .required(false)
        )
        .arg(
            Arg::new("text")
                .help("The text to search within (use -f to read from a file)")
                .required(false)
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Launch the interactive TUI mode")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("Read text from a file instead of standard input")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Write the output to a file instead of standard output")
                .value_name("FILE"),
        )
        .get_matches();

    let interactive = matches.get_flag("interactive");
    let file = matches.get_one::<String>("file");
    let output = matches.get_one::<String>("output");
    let pattern = matches.get_one::<String>("pattern");
    let text = matches.get_one::<String>("text");

    // Check argument conditions:
    // If no arguments provided at all
    let no_args_provided = !interactive && file.is_none() && output.is_none() && pattern.is_none() && text.is_none();
    if no_args_provided {
        eprintln!("No arguments provided. See --help for usage.");
        std::process::exit(1);
    }

    if !interactive {
        // Non-interactive mode conditions:
        if file.is_some() {
            // File provided: must have pattern, must NOT have text
            if pattern.is_none() || text.is_some() {
                eprintln!("When using -f FILE, you must provide PATTERN and must not provide TEXT. See --help for usage.");
                std::process::exit(1);
            }
        } else {
            // No file: must have both pattern and text
            if pattern.is_none() || text.is_none() {
                eprintln!("Non-interactive mode requires both PATTERN and TEXT if not using -f FILE. See --help for usage.");
                std::process::exit(1);
            }
        }
    } else {
        // Interactive mode conditions:
        // If pattern/text/file are provided, they will pre-fill fields.
        // If none are provided, user will input both in TUI.
    }

    // Print out what we are doing
    println!("Running regexer with the following options:");
    if interactive {
        println!("  - Running in interactive mode");
    }
    if let Some(file_name) = file {
        println!("  - Using file input: {}", file_name);
    }
    if let Some(output_file) = output {
        println!("  - Output file: {}", output_file);
    }
    if let Some(p) = pattern {
        println!("  - Pattern: {}", p);
    }
    if let Some(t) = text {
        println!("  - Text: {}", t);
    }

    if interactive {
        let mut app = App::new();
        // Pre-fill if provided:
        if let Some(p) = pattern {
            app.set_pattern(p);
        }
        if let Some(t) = text {
            app.set_text(t);
        }

        let terminal = ratatui::init();
        let app_result = app.run(terminal);
        ratatui::restore();
        app_result
    } else {
        // Just exit after printing arguments in non-interactive mode.
        Ok(())
    }
}

/// App holds the state of the application
struct App {
    /// Current value of the input box (for text)
    input: String,
    /// Current pattern (separate from input text)
    pattern: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded expressions
    expressions: Vec<String>,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            pattern: String::new(),
            input_mode: InputMode::Normal,
            expressions: Vec::new(),
            character_index: 0,
        }
    }

    fn set_pattern(&mut self, p: &str) {
        self.pattern = p.to_string();
    }

    fn set_text(&mut self, t: &str) {
        self.input = t.to_string();
        self.character_index = self.input.chars().count();
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_expression(&mut self) {
        self.expressions.push(format!("Pattern: {}, Text: {}", self.pattern, self.input));
        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C globally
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(());
                }

                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_expression(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, pattern_area, input_area, expressions_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " or ".into(),
                    "Esc".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start typing, or Ctrl+C at any time to exit.".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to save expression to the list, or Ctrl+C at any time to exit.".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let pattern_par = Paragraph::new(self.pattern.as_str())
            .style(Style::default().fg(Color::Cyan))
            .block(Block::bordered().title("Pattern"));
        frame.render_widget(pattern_par, pattern_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Text"));
        frame.render_widget(input, input_area);

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::Editing => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16 + 1,
                input_area.y + 1,
            )),
        }

        let expressions: Vec<ListItem> = self
            .expressions
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let expressions = List::new(expressions).block(Block::bordered().title("Expressions"));
        frame.render_widget(expressions, expressions_area);
    }
}
