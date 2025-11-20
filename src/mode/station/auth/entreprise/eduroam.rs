use anyhow::{Result, anyhow};
use std::{fs::OpenOptions, io::Write};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{HighlightSpacing, List, ListState},
};

use tui_input::{Input, backend::crossterm::EventHandler};

use crate::mode::station::auth::entreprise::ERROR_PADDING;

fn pad_string(input: &str, length: usize) -> String {
    let current_length = input.chars().count();
    if current_length >= length {
        input.to_string()
    } else {
        format!("{:<width$}", input, width = length)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusedInput {
    #[default]
    Identity,
    Phase2Identity,
    Phase2Password,
}

#[derive(Debug, Clone, Default)]
pub struct Eduroam {
    identity: UserInputField,
    phase2_identity: UserInputField,
    phase2_password: UserInputField,
    pub focused_input: FocusedInput,
    state: ListState,
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl Eduroam {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_identity(&mut self) {
        self.identity.error = None;
        if self.identity.field.value().is_empty() {
            self.identity.error = Some("Required field.".to_string());
        }
    }

    pub fn validate_phase2_identity(&mut self) {
        self.phase2_identity.error = None;
        if self.phase2_identity.field.value().is_empty() {
            self.phase2_identity.error = Some("Required field.".to_string());
        }
    }
    pub fn validate_phase2_password(&mut self) {
        self.phase2_password.error = None;
        if self.phase2_password.field.value().is_empty() {
            self.phase2_password.error = Some("Required field.".to_string());
        }
    }

    pub fn validate(&mut self) -> Result<()> {
        self.validate_identity();
        self.validate_phase2_identity();
        self.validate_phase2_password();
        if self.identity.error.is_some()
            | self.phase2_identity.error.is_some()
            | self.phase2_password.error.is_some()
        {
            return Err(anyhow!("Valdidation Error"));
        }
        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(4) => None,
            Some(i) => Some(i + 2),
            None => Some(0),
        };

        self.state.select(i);
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) => None,
            Some(i) => Some(i.saturating_sub(2)),
            None => Some(4),
        };

        self.state.select(i);
    }

    pub fn set_last(&mut self) {
        self.state.select(Some(4));
    }

    pub fn selected(&self) -> bool {
        self.state.selected().is_some()
    }

    pub fn apply(&mut self) -> Result<()> {
        self.validate()?;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open("/var/lib/iwd/eduroam.8021x")?;
        let text = format!(
            "
[Security]
EAP-Method=PEAP
EAP-Identity={}
EAP-PEAP-Phase2-Method=MSCHAPV2
EAP-PEAP-Phase2-Identity={}
EAP-PEAP-Phase2-Password={}

[Settings]
AutoConnect=true",
            self.identity.field.value(),
            self.phase2_identity.field.value(),
            self.phase2_password.field.value(),
        );

        let text = text.trim_start();
        file.write_all(text.as_bytes())?;

        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                let _ = self.validate();
            }
            _ => match self.focused_input {
                FocusedInput::Identity => {
                    self.identity
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::Phase2Identity => {
                    self.phase2_identity
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::Phase2Password => {
                    self.phase2_password
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
            },
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(area);

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(80),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        let items = [
            Line::from(vec![
                Span::from(pad_string(" Identity", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.identity.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.identity.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
            Line::from(vec![
                Span::from(pad_string(" Phase2 Identity", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.phase2_identity.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.phase2_identity.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
            Line::from(vec![
                Span::from(pad_string(" Phase2 Password", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.phase2_password.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.phase2_password.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
        ];

        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);

        frame.render_stateful_widget(list, block, &mut self.state);
    }
}
