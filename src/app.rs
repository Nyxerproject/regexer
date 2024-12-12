use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use std::fs;

use crate::engines::{apply_pattern, EngineChoice};

pub struct ExpressionEntry {
    pattern: String,
    text: String,
    matches: String,
}

pub enum InputMode {
    Normal,
    EditingPattern,
    EditingText,
}

pub struct App {
    pub input: String,
    pub pattern: String,
    pub input_mode: InputMode,
    pub expressions: Vec<ExpressionEntry>,
    pub character_index: usize,
    pub file: Option<String>,
    pub engine_choice: EngineChoice,
}

impl App {
    pub fn new(engine_choice: EngineChoice) -> Self {
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

    pub fn set_pattern(&mut self, p: &str) {
        self.pattern = p.to_string();
    }

    pub fn set_text(&mut self, t: &str) {
        self.input = t.to_string();
        self.character_index = self.input.chars().count();
    }

    pub fn set_file(&mut self, f: Option<String>) {
        self.file = f;
    }

    pub fn pattern_is_empty(&self) -> bool {
        self.pattern.is_empty()
    }

    pub fn has_file(&self) -> bool {
        self.file.is_some()
    }

    pub fn enter_pattern_mode(&mut self) {
        self.input_mode = InputMode::EditingPattern;
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

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
                    " at any time to exit.".into(),
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
