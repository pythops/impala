use std::{fs::OpenOptions, io::Write, path::Path};

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
    CaCert,
    Identity,
    ClientCert,
    ClientKey,
    KeyPassphrase,
}

#[derive(Debug, Clone, Default)]
pub struct TLS {
    ca_cert: UserInputField,
    identity: UserInputField,
    client_cert: UserInputField,
    client_key: UserInputField,
    key_passphrase: UserInputField,
    pub focused_input: FocusedInput,
    state: ListState,
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl TLS {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_ca_cert(&mut self) {
        self.ca_cert.error = None;
        if self.ca_cert.field.value().is_empty() {
            self.ca_cert.error = Some("Required field.".to_string());
            return;
        }
        let path = Path::new(self.ca_cert.field.value());

        if !path.is_absolute() {
            self.ca_cert.error = Some("The file path should be absolute.".to_string());
            return;
        }

        if !path.exists() {
            self.ca_cert.error = Some("The file does not exist.".to_string());
        }
    }

    pub fn validate_identity(&mut self) {
        self.identity.error = None;
        if self.identity.field.value().is_empty() {
            self.identity.error = Some("Required field.".to_string());
        }
    }

    pub fn validate_client_cert(&mut self) {
        self.client_cert.error = None;
        if self.client_cert.field.value().is_empty() {
            self.client_cert.error = Some("Required field.".to_string());
            return;
        }

        let path = Path::new(self.client_cert.field.value());

        if !path.is_absolute() {
            self.client_cert.error = Some("The file path should be absolute.".to_string());
            return;
        }

        if !path.exists() {
            self.client_cert.error = Some("The file does not exist.".to_string());
        }
    }
    pub fn validate_client_key(&mut self) {
        self.client_key.error = None;
        if self.client_key.field.value().is_empty() {
            self.client_key.error = Some("Required field.".to_string());
            return;
        }

        let path = Path::new(self.client_key.field.value());

        if !path.is_absolute() {
            self.client_key.error = Some("The file path should be absolute.".to_string());
            return;
        }

        if !path.exists() {
            self.client_key.error = Some("The file does not exist.".to_string());
        }
    }

    pub fn validate(&mut self) -> AppResult<()> {
        self.validate_ca_cert();
        self.validate_identity();
        self.validate_client_cert();
        self.validate_client_key();
        if self.ca_cert.error.is_some()
            | self.identity.error.is_some()
            | self.client_cert.error.is_some()
            | self.client_key.error.is_some()
        {
            return Err("Valdidation Error".into());
        }
        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(8) => None,
            Some(i) => Some(i + 2),
            None => Some(0),
        };

        self.state.select(i);
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) => None,
            Some(i) => Some(i.saturating_sub(2)),
            None => Some(8),
        };

        self.state.select(i);
    }

    pub fn set_last(&mut self) {
        self.state.select(Some(8));
    }

    pub fn deselect(&mut self) {
        self.state.select(None);
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
        let mut text = format!(
            "
[Security]
EAP-Method=TLS
EAP-TLS-CACert={}
EAP-Identity={}
EAP-TLS-ClientCert={}
EAP-TLS-ClientKey={}
",
            self.ca_cert.field.value(),
            self.identity.field.value(),
            self.client_cert.field.value(),
            self.client_key.field.value(),
        );

        if !self.key_passphrase.field.value().is_empty() {
            text.push_str(
                format!(
                    "EAP-TLS-ClientKeyPassphrase={}",
                    self.key_passphrase.field.value()
                )
                .as_str(),
            );
        }

        text.push_str(
            "

[Settings]
AutoConnect=true",
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
            KeyCode::Enter => {}
            _ => match self.focused_input {
                FocusedInput::CaCert => {
                    self.ca_cert
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::Identity => {
                    self.identity
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::ClientCert => {
                    self.client_cert
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::ClientKey => {
                    self.client_key
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::KeyPassphrase => {
                    self.key_passphrase
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
                Constraint::Max(70),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        let items = [
            Line::from(vec![
                Span::from(pad_string(" CA Cert", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.ca_cert.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.ca_cert.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
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
                Span::from(pad_string(" Client Cert", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.client_cert.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.client_cert.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
            Line::from(vec![
                Span::from(pad_string(" Client Key", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.client_key.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.client_key.error {
                    Span::from(error)
                } else {
                    Span::from("")
                }
            }])
            .red(),
            Line::from(vec![
                Span::from(pad_string(" Key Passphrase", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.key_passphrase.field.value(), 50)).bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.key_passphrase.error {
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
