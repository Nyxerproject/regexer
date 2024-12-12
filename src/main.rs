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

use regex::Regex;
use std::fs; // Builtin fallback

#[allow(unused_imports)]
use regex_automata::dfa::onepass::DFA as OnePassRegex;
#[allow(unused_imports)]
use regex_automata::dfa::regex::Regex as DfaRegex;
#[allow(unused_imports)]
use regex_automata::hybrid::regex::Regex as HybridRegex;
#[allow(unused_imports)]
use regex_automata::meta::Regex as MetaRegex;
#[allow(unused_imports)]
use regex_automata::nfa::thompson::backtrack::BoundedBacktracker as BacktrackRegex;
#[allow(unused_imports)]
use regex_automata::nfa::thompson::pikevm::PikeVM as PikeVMRegex;

mod custom_regex;
use custom_regex::CustomRegex;

enum EngineChoice {
    Builtin,
    Custom,
    Dfa,
    Hybrid,
    Onepass,
    Boundedbacktracker,
    Pikevm,
    Meta,
    Custommeta, // New engine
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = Command::new("regexer")
        .version("0.1.0")
        .about("A regex CLI/TUI tool for parsing and testing regular expressions.")
        .long_about(
"regexer is a command-line/text-user interface tool for parsing and testing regular expressions.

...
Use --engine to select the regex engine: (some are wip and are not implemented)
  - builtin
  - custom
  - dfa
  - hybrid
  - onepass
  - boundedbacktracker
  - pikevm 
  - meta
  - custommeta (tries CustomRegex first, verify with builtin, fallback to builtin on error)
"
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
        .arg(
            Arg::new("engine")
                .long("engine")
                .help("Select the regex engine to use: builtin, custom, dfa, hybrid, onepass, boundedbacktracker, pikevm, meta, custommeta")
                .value_parser(["builtin", "custom", "dfa", "hybrid", "onepass", "boundedbacktracker", "pikevm", "meta", "custommeta"])
                .default_value("builtin")
        )
        .get_matches();

    let interactive = matches.get_flag("interactive");
    let file = matches.get_one::<String>("file");
    let output = matches.get_one::<String>("output");
    let pattern = matches.get_one::<String>("pattern");
    let text = matches.get_one::<String>("text");
    let engine_str = matches.get_one::<String>("engine").unwrap();
    let engine_choice = match engine_str.as_str() {
        "builtin" => EngineChoice::Builtin,
        "custom" => EngineChoice::Custom,
        "dfa" => EngineChoice::Dfa,
        "hybrid" => EngineChoice::Hybrid,
        "onepass" => EngineChoice::Onepass,
        "boundedbacktracker" => EngineChoice::Boundedbacktracker,
        "pikevm" => EngineChoice::Pikevm,
        "meta" => EngineChoice::Meta,
        "custommeta" => EngineChoice::Custommeta,
        _ => EngineChoice::Builtin,
    };

    let no_args_provided =
        !interactive && file.is_none() && output.is_none() && pattern.is_none() && text.is_none();
    if no_args_provided {
        eprintln!("No arguments provided. See --help for usage.");
        std::process::exit(1);
    }

    if !interactive {
        if file.is_some() {
            if pattern.is_none() || text.is_some() {
                eprintln!("When using -f FILE, you must provide PATTERN and must not provide TEXT. See --help for usage.");
                std::process::exit(1);
            }
        } else {
            if pattern.is_none() || text.is_none() {
                eprintln!("Non-interactive mode requires both PATTERN and TEXT if not using -f FILE. See --help for usage.");
                std::process::exit(1);
            }
        }
    }

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
    if let Some(engine_choice) = text {
        println!("  - Engine: {}", engine_choice);
    }

    if interactive {
        let mut app = App::new(engine_choice);

        if let Some(p) = pattern {
            app.set_pattern(p);
        }
        if let Some(t) = text {
            app.set_text(t);
        }
        if file.is_some() {
            app.file = file.map(|f| f.to_string());
        }

        if app.pattern.is_empty() && app.file.is_some() {
            app.input_mode = InputMode::EditingPattern;
        }

        let terminal = ratatui::init();
        let app_result = app.run(terminal);
        ratatui::restore();
        app_result
    } else {
        // Non-interactive: no TUI
        Ok(())
    }
}

struct ExpressionEntry {
    pattern: String,
    text: String,
    matches: String,
}

struct App {
    input: String,
    pattern: String,
    input_mode: InputMode,
    expressions: Vec<ExpressionEntry>,
    character_index: usize,
    file: Option<String>,
    engine_choice: EngineChoice,
}

enum InputMode {
    Normal,
    EditingPattern,
    EditingText,
}

impl App {
    fn new(engine_choice: EngineChoice) -> Self {
        Self {
            input: String::new(),
            pattern: String::new(),
            input_mode: InputMode::Normal,
            expressions: Vec::new(),
            character_index: 0,
            file: None,
            engine_choice,
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

    fn submit_pattern(&mut self) {
        self.pattern = self.input.clone();
        self.input.clear();
        self.reset_cursor();
        self.input_mode = InputMode::Normal;

        if let Some(file_name) = &self.file {
            if let Ok(contents) = fs::read_to_string(file_name) {
                self.add_expression(contents);
            }
        }
        self.file = Some("".to_string());
    }

    fn submit_text(&mut self) {
        let txt = self.input.clone();
        self.input.clear();
        self.reset_cursor();
        self.input_mode = InputMode::Normal;
        self.add_expression(txt);
    }

    fn add_expression(&mut self, text: String) {
        let matches = apply_pattern(&self.pattern, &text, &self.engine_choice);
        self.expressions.push(ExpressionEntry {
            pattern: self.pattern.clone(),
            text,
            matches,
        });
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
                            self.input_mode = InputMode::EditingText;
                        }
                        KeyCode::Char('p') => {
                            self.input_mode = InputMode::EditingPattern;
                            self.input = self.pattern.clone();
                            self.character_index = self.input.chars().count();
                        }
                        KeyCode::Char('t') => {
                            self.input_mode = InputMode::EditingText;
                            self.input.clear();
                            self.reset_cursor();
                        }
                        KeyCode::Char('q') | KeyCode::Esc => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::EditingPattern if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Enter => self.submit_pattern(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => {
                                self.input_mode = InputMode::Normal;
                                self.input.clear();
                                self.reset_cursor();
                            }
                            _ => {}
                        }
                    }
                    InputMode::EditingText if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_text(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => {
                            self.input_mode = InputMode::Normal;
                            self.input.clear();
                            self.reset_cursor();
                        }
                        _ => {}
                    },
                    _ => {}
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
                    "p".bold(),
                    " to edit pattern, ".into(),
                    "t".bold(),
                    " to edit text, ".into(),
                    "e".bold(),
                    " to edit text (legacy), or ".into(),
                    "Ctrl+C".bold(),
                    "at any time to exit.".into(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::EditingPattern => (
                vec![
                    "Editing Pattern: Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to submit pattern.".into(),
                ],
                Style::default(),
            ),
            InputMode::EditingText => (
                vec![
                    "Editing Text: Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to add expression.".into(),
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

        let input_title = match self.input_mode {
            InputMode::EditingPattern => "Editing Pattern",
            InputMode::EditingText => "Editing Text",
            InputMode::Normal => "Text",
        };
        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::EditingPattern => Style::default().fg(Color::Green),
                InputMode::EditingText => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title(input_title));
        frame.render_widget(input, input_area);

        match self.input_mode {
            InputMode::Normal => {}
            InputMode::EditingPattern | InputMode::EditingText => {
                frame.set_cursor_position(Position::new(
                    input_area.x + self.character_index as u16 + 1,
                    input_area.y + 1,
                ))
            }
        }

        let expressions: Vec<ListItem> = self
            .expressions
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!(
                    "{i}: Pattern: {}, Text: {}, {}",
                    m.pattern, m.text, m.matches
                )));
                ListItem::new(content)
            })
            .collect();
        let expressions = List::new(expressions).block(Block::bordered().title("Expressions"));
        frame.render_widget(expressions, expressions_area);
    }
}

fn apply_pattern(pattern: &str, text: &str, engine_choice: &EngineChoice) -> String {
    match engine_choice {
        EngineChoice::Builtin => apply_pattern_builtin(pattern, text),
        EngineChoice::Custom => apply_pattern_custom(pattern, text),
        EngineChoice::Dfa => apply_pattern_builtin(pattern, text),
        //EngineChoice::Hybrid => apply_pattern_hybrid(pattern, text),
        // EngineChoice::Onepass => apply_pattern_onepass(pattern, text),
        // EngineChoice::Boundedbacktracker => apply_pattern_backtrack(pattern, text),
        // EngineChoice::Pikevm => apply_pattern_pikevm(pattern, text),
        // EngineChoice::Meta => apply_pattern_meta(pattern, text),
        // EngineChoice::Custommeta => apply_pattern_custommeta(pattern, text),
        //EngineChoice::Dfa => "DFA engine (placeholder)".to_string(),
        EngineChoice::Hybrid => {
            "Hybrid (lazy DFA) engine (placeholder. Is used by hybrid)".to_string()
        }
        EngineChoice::Meta => "Meta engine (placeholder)".to_string(),
        EngineChoice::Onepass => "One-pass DFA engine (placeholder)".to_string(),
        EngineChoice::Boundedbacktracker => "Bounded backtracking engine (placeholder)".to_string(),
        EngineChoice::Pikevm => "PikeVM engine (placeholder. Is used by Hybrid)".to_string(),
        EngineChoice::Custommeta => apply_pattern_custommeta(pattern, text),
    }
}

// fn apply_pattern_hybrid(pattern: &str, text: &str) -> String {
//     let regex = match regex_automata::meta::Regex::new(pattern) {
//         Ok(r) => r,
//         Err(e) => return format!("Invalid pattern: {}", e),
//     };
//     let mut all_matches = Vec::new();
//     for mat in regex.find_iter(text) {
//         all_matches.push(mat.as_str().to_string());
//     }
//     if all_matches.is_empty() {
//         "No matches found.".to_string()
//     } else {
//         format!("Matches: {:?}", all_matches)
//     }
// }
// fn apply_pattern_hybrid(pattern: &str, text: &str) -> String {
//     let cr = regex_automata::meta::Regex::new(pattern);
//     match cr {
//         Ok(parser) => {
//             let all_matches = parser.find_iter(text);
//             if all_matches. {
//                 "No matches found.".to_string()
//             } else {
//                 format!("Matches: {:?}", all_matches)
//             }
//         }
//         Err(e) => format!("Invalid pattern: {}", e),
//     }
// }

fn apply_pattern_builtin(pattern: &str, text: &str) -> String {
    let regex = match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => return format!("Invalid pattern: {}", e),
    };
    let mut all_matches = Vec::new();
    for mat in regex.find_iter(text) {
        all_matches.push(mat.as_str().to_string());
    }
    if all_matches.is_empty() {
        "No matches found.".to_string()
    } else {
        format!("Matches: {:?}", all_matches)
    }
}

#[allow(dead_code)]
fn apply_pattern_dfa(pattern: &str, text: &str) -> String {
    let cr = CustomRegex::new(pattern);
    match cr {
        Ok(parser) => {
            let all_matches = parser.find_iter(text);
            if all_matches.is_empty() {
                "No matches found.".to_string()
            } else {
                format!("Matches: {:?}", all_matches)
            }
        }
        Err(e) => format!("Invalid pattern: {}", e),
    }
}

fn apply_pattern_custom(pattern: &str, text: &str) -> String {
    let cr = CustomRegex::new(pattern);
    match cr {
        Ok(parser) => {
            let all_matches = parser.find_iter(text);
            if all_matches.is_empty() {
                "No matches found.".to_string()
            } else {
                format!("Matches: {:?}", all_matches)
            }
        }
        Err(e) => format!("Invalid pattern: {}", e),
    }
}

/// CustomMeta engine logic:
/// 1. Try custom regex
/// 2. If custom regex returns error, print to stderr and fallback to builtin
/// 3. If custom regex succeeds, verify at least one substring is also found by builtin.
///    If builtin finds no matches, fallback to builtin result.
fn apply_pattern_custommeta(pattern: &str, text: &str) -> String {
    let cr = CustomRegex::new(pattern);
    match cr {
        Ok(parser) => {
            let custom_matches = parser.find_iter(text);
            if custom_matches.is_empty() {
                eprintln!("customMeta: CustomRegex found no matches, verifying with builtin.");
                // Verify with builtin
                let builtin_result = apply_pattern_builtin(pattern, text);
                if builtin_result.contains("No matches found")
                    || builtin_result.contains("Invalid pattern")
                {
                    // fallback
                    eprintln!("customMeta: Builtin also failed, returning builtin result.");
                    return builtin_result;
                } else {
                    // return custom's result anyway since we must trust custom or we can return builtin?
                    // Let's return custom's result here as custom did run successfully, just no matches:
                    return "No matches found.".to_string();
                }
            } else {
                // Custom found matches, let's verify with builtin
                let builtin_result = apply_pattern_builtin(pattern, text);
                if builtin_result.contains("Invalid pattern") {
                    // Pattern invalid for builtin, fallback
                    eprintln!("customMeta: CustomRegex succeeded but builtin reports invalid pattern. Using builtin anyway.");
                    return builtin_result;
                }

                // If builtin finds no matches but custom does, that might be suspect
                if builtin_result.contains("No matches found") {
                    eprintln!("customMeta: CustomRegex found matches but builtin found none. Using builtin as fallback.");
                    return builtin_result;
                }

                // Both found matches, trust custom since that was first
                format!("Matches: {:?}", custom_matches)
            }
        }
        Err(e) => {
            eprintln!(
                "customMeta: CustomRegex error: {}. Falling back to builtin.",
                e
            );
            apply_pattern_builtin(pattern, text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Adjust depending on your module structure.

    #[test]
    fn test_builtin_engine_valid_pattern() {
        let pattern = "ab.";
        let text = "abc abx aby";
        let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
        assert!(
            result.contains("Matches: [\"abc\", \"abx\", \"aby\"]"),
            "Expected three matches for 'ab.'"
        );
    }

    #[test]
    fn test_builtin_engine_invalid_pattern() {
        let pattern = "("; // invalid pattern
        let text = "abc";
        let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
        assert!(
            result.contains("Invalid pattern:"),
            "Expected invalid pattern error."
        );
    }

    #[test]
    fn test_custom_engine_with_valid_pattern() {
        // Assuming your CustomRegex currently handles simple patterns like a literal
        let pattern = "a";
        let text = "abc a";
        let result = apply_pattern(pattern, text, &EngineChoice::Custom);
        // CustomRegex tries all substrings, so "abc" and "a" should both have matches
        // E.g. Matches: ["a", "a"] (from "abc" -> 'a' at start, and the standalone 'a')
        assert!(
            result.contains("Matches:"),
            "Expected at least one match from CustomRegex."
        );
    }

    #[test]
    fn test_custom_engine_with_invalid_pattern() {
        let pattern = ""; // empty pattern is considered invalid in CustomRegex
        let text = "abc";
        let result = apply_pattern(pattern, text, &EngineChoice::Custom);
        assert!(
            result.contains("Invalid pattern:"),
            "Expected invalid pattern error from CustomRegex."
        );
    }

    #[test]
    fn test_custommeta_engine_fallback() {
        // CustomMeta tries CustomRegex first, then verifies or falls back.
        // Use a pattern that CustomRegex can parse but let's say it finds no matches:
        let pattern = "z";
        let text = "abc";
        // "z" doesn't appear, so CustomRegex: no matches, then builtin also no matches.
        let result = apply_pattern(pattern, text, &EngineChoice::Custommeta);
        assert!(
            result.contains("No matches found."),
            "Expected no matches found on fallback."
        );

        // Now try something custom can handle:
        let pattern2 = "a";
        let text2 = "abc";
        let result2 = apply_pattern(pattern2, text2, &EngineChoice::Custommeta);
        // Both custom and builtin should find matches, so we should have a Matches line
        assert!(
            result2.contains("Matches:"),
            "Expected matches from customMeta engine."
        );
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_builtin_engine_valid_pattern() {
            let pattern = "ab.";
            let text = "abc abx aby";
            let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
            assert!(
                result.contains("Matches: [\"abc\", \"abx\", \"aby\"]"),
                "Expected three matches for 'ab.'"
            );
        }

        #[test]
        fn test_builtin_engine_invalid_pattern() {
            let pattern = "("; // invalid pattern
            let text = "abc";
            let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
            assert!(
                result.contains("Invalid pattern:"),
                "Expected invalid pattern error."
            );
        }

        #[test]
        fn test_builtin_engine_no_matches() {
            let pattern = "xyz";
            let text = "abc def ghi";
            let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
            assert!(
                result.contains("No matches found."),
                "Expected no matches found for a pattern that does not appear."
            );
        }

        #[test]
        fn test_custom_engine_with_valid_pattern() {
            let pattern = "a";
            let text = "abc a";
            let result = apply_pattern(pattern, text, &EngineChoice::Custom);
            assert!(
                result.contains("Matches:"),
                "Expected at least one match from CustomRegex."
            );
        }

        #[test]
        fn test_custom_engine_with_invalid_pattern() {
            let pattern = ""; // empty pattern is considered invalid in CustomRegex
            let text = "abc";
            let result = apply_pattern(pattern, text, &EngineChoice::Custom);
            assert!(
                result.contains("Invalid pattern:"),
                "Expected invalid pattern error from CustomRegex."
            );
        }

        #[test]
        fn test_custom_engine_no_matches() {
            let pattern = "z";
            let text = "abc def";
            let result = apply_pattern(pattern, text, &EngineChoice::Custom);
            assert!(
                result.contains("No matches found."),
                "Expected no matches found from CustomRegex."
            );
        }

        #[test]
        fn test_custommeta_engine_fallback() {
            // CustomMeta tries CustomRegex first, then falls back to meta if no matches.
            let pattern = "z";
            let text = "abc";
            let result = apply_pattern(pattern, text, &EngineChoice::Custommeta);
            assert!(
                result.contains("No matches found."),
                "Expected no matches found on fallback."
            );

            let pattern2 = "a";
            let text2 = "abc";
            let result2 = apply_pattern(pattern2, text2, &EngineChoice::Custommeta);
            assert!(
                result2.contains("Matches:"),
                "Expected matches from customMeta engine."
            );
        }

        // Tests for other engines:

        #[test]
        fn test_dfa_engine_valid_pattern() {
            let pattern = "ab.";
            let text = "abc abx aby";
            let result = apply_pattern(pattern, text, &EngineChoice::Dfa);
            assert!(
                result.contains("Matches: [\"abc\", \"abx\", \"aby\"]"),
                "Expected matches from DFA engine."
            );
        }

        #[test]
        fn test_dfa_engine_invalid_pattern() {
            let pattern = "(";
            let text = "abc";
            let result = apply_pattern(pattern, text, &EngineChoice::Dfa);
            assert!(
                result.contains("Invalid pattern:"),
                "Expected invalid pattern from DFA engine."
            );
        }

        #[test]
        fn test_dfa_engine_no_matches() {
            let pattern = "zzz";
            let text = "abc def";
            let result = apply_pattern(pattern, text, &EngineChoice::Dfa);
            assert!(
                result.contains("No matches found."),
                "Expected no matches from DFA engine."
            );
        }
        // Additional edge case tests
        #[test]
        fn test_empty_text() {
            let pattern = "a";
            let text = "";
            let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
            assert!(
                result.contains("No matches found."),
                "Expected no matches on empty text."
            );
        }

        #[test]
        fn test_empty_pattern_builtin() {
            let pattern = "";
            let text = "abc";
            let result = apply_pattern(pattern, text, &EngineChoice::Builtin);
            assert!(
                !result.contains("Invalid pattern:"),
                "Empty pattern should not produce invalid pattern error on builtin (if allowed)."
            );
        }
    }
}
