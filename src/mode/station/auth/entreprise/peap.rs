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
    ServerDomainMask,
    Identity,
    Phase2Identity,
    Phase2Password,
}

#[derive(Debug, Clone, Default)]
pub struct PEAP {
    ca_cert: UserInputField,
    server_domain_mask: UserInputField,
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

impl PEAP {
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

    pub fn validate_server_domain_mask(&mut self) {
        self.server_domain_mask.error = None;
        if self.server_domain_mask.field.value().is_empty() {
            self.server_domain_mask.error = Some("Required field.".to_string());
        }
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

    pub fn validate(&mut self) -> AppResult<()> {
        self.validate_ca_cert();
        self.validate_server_domain_mask();
        self.validate_identity();
        self.validate_phase2_identity();
        self.validate_phase2_password();
        if self.ca_cert.error.is_some()
            | self.identity.error.is_some()
            | self.server_domain_mask.error.is_some()
            | self.phase2_identity.error.is_some()
            | self.phase2_password.error.is_some()
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

    pub fn selected(&self) -> bool {
        self.state.selected().is_some()
    }

    pub fn apply(&mut self, network_name: &str) -> AppResult<()> {
        self.validate()?;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(format!("/var/lib/iwd/{}.8021x", network_name))?;
        let text = format!(
            "
[Security]
EAP-Method=PEAP
EAP-PEAP-CACert={}
EAP-Identity={}
EAP-PEAP-ServerDomainMask={}
EAP-PEAP-Phase2-Method=MSCHAPV2
EAP-PEAP-Phase2-Identity={}
EAP-PEAP-Phase2-Password={}

[Settings]
AutoConnect=true",
            self.ca_cert.field.value(),
            self.identity.field.value(),
            self.server_domain_mask.field.value(),
            self.phase2_identity.field.value(),
            self.phase2_password.field.value(),
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
                FocusedInput::CaCert => {
                    self.ca_cert
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::ServerDomainMask => {
                    self.server_domain_mask
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
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
                Span::from(pad_string(" Server Domain Mask", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(pad_string(self.server_domain_mask.field.value(), 50))
                    .bg(Color::DarkGray),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), {
                if let Some(error) = &self.server_domain_mask.error {
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
