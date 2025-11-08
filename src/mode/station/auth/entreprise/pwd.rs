use std::{fs::OpenOptions, io::Write};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{HighlightSpacing, List, ListState},
};

use tokio::sync::mpsc::UnboundedSender;
use tui_input::{Input, backend::crossterm::EventHandler};

use crate::{app::AppResult, event::Event, mode::station::auth::entreprise::ERROR_PADDING};

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
    Password,
}

#[derive(Debug, Clone, Default)]
pub struct PWD {
    identity: UserInputField,
    password: UserInputField,
    pub focused_input: FocusedInput,
    state: ListState,
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl PWD {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_identity(&mut self) {
        self.identity.error = None;
        if self.identity.field.value().is_empty() {
            self.identity.error = Some("Required field.".to_string());
        }
    }

    pub fn validate_password(&mut self) {
        self.password.error = None;
        if self.password.field.value().is_empty() {
            self.password.error = Some("Required field.".to_string());
        }
    }

    pub fn validate(&mut self) -> AppResult<()> {
        self.validate_identity();
        self.validate_password();
        if self.identity.error.is_some() | self.password.error.is_some() {
            return Err("Valdidation Error".into());
        }
        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(2) => None,
            Some(_) => Some(2),
            None => Some(0),
        };

        self.state.select(i);
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) => None,
            Some(_) => Some(0),
            None => Some(2),
        };

        self.state.select(i);
    }

    pub fn set_last(&mut self) {
        self.state.select(Some(2));
    }

    pub fn selected(&self) -> bool {
        self.state.selected().is_some()
    }

    pub fn apply(&mut self, network_name: &str) -> AppResult<()> {
        self.validate()?;

        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open(format!("/var/lib/iwd/{}.8021x", network_name))?;
        let text = format!(
            "
[Security]
EAP-Method=PWD
EAP-Identity={}
EAP-Password={}

[Settings]
AutoConnect=true",
            self.identity.field.value(),
            self.password.field.value(),
        );

        let text = text.trim_start();
        file.write_all(text.as_bytes())?;

        Ok(())
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        _sender: UnboundedSender<Event>,
    ) -> AppResult<()> {
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
                FocusedInput::Password => {
                    self.password
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
            },
        }
        Ok(())
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
                Span::from(pad_string(" Password", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.password.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.password.error {
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
