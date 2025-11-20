use anyhow::{Result, anyhow};
use std::{fs::OpenOptions, io::Write, path::Path};

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Margin, Rect},
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{HighlightSpacing, List, ListState},
};

use tui_input::backend::crossterm::EventHandler;

use crate::mode::station::auth::entreprise::{ERROR_PADDING, UserInputField};

fn pad_string(input: &str, length: usize) -> String {
    let current_length = input.chars().count();
    if current_length >= length {
        input.to_string()
    } else {
        format!("{:<width$}", input, width = length)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Default, strum_macros::Display)]
enum Phase2Method {
    #[default]
    MSCHAPV2,
    SIM,
    GTC,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FocusedInput {
    #[default]
    Identity,
    ServerDomainMask,
    CaCert,
    ClientCert,
    ClientKey,
    KeyPassphrase,
    Phase2Method,
    Phase2Identity,
    Phase2Password,
}

#[derive(Debug, Clone, Default)]
pub struct PEAP {
    identity: UserInputField,
    server_domain_mask: UserInputField,
    ca_cert: UserInputField,
    client_cert: UserInputField,
    client_key: UserInputField,
    key_passphrase: UserInputField,
    phase2_method: Phase2Method,
    phase2_identity: UserInputField,
    phase2_password: UserInputField,
    pub focused_input: FocusedInput,
    state: ListState,
}

impl PEAP {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_ca_cert(&mut self) {
        self.ca_cert.error = None;
        if !self.ca_cert.value().is_empty() {
            let path = Path::new(self.ca_cert.value());

            if !path.is_absolute() {
                self.ca_cert.error = Some("The file path should be absolute.".to_string());
                return;
            }

            if !path.exists() {
                self.ca_cert.error = Some("The file does not exist.".to_string());
            }
        }
    }

    pub fn validate_client_cert(&mut self) {
        self.client_cert.error = None;
        if !self.client_cert.value().is_empty() {
            let path = Path::new(self.client_cert.value());

            if !path.is_absolute() {
                self.client_cert.error = Some("The file path should be absolute.".to_string());
                return;
            }

            if !path.exists() {
                self.client_cert.error = Some("The file does not exist.".to_string());
            }
        }
    }

    pub fn validate_client_key(&mut self) {
        self.client_key.error = None;
        if !self.client_key.value().is_empty() {
            let path = Path::new(self.client_key.value());

            if !path.is_absolute() {
                self.client_key.error = Some("The file path should be absolute.".to_string());
                return;
            }

            if !path.exists() {
                self.client_key.error = Some("The file does not exist.".to_string());
            }
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

    pub fn validate(&mut self) -> Result<()> {
        self.validate_identity();
        self.validate_ca_cert();
        self.validate_client_cert();
        self.validate_client_key();
        self.validate_phase2_identity();
        self.validate_phase2_password();
        if self.ca_cert.error.is_some()
            | self.client_cert.error.is_some()
            | self.client_key.error.is_some()
            | self.identity.error.is_some()
            | self.phase2_identity.error.is_some()
            | self.phase2_password.error.is_some()
        {
            return Err(anyhow!("Valdidation Error"));
        }
        Ok(())
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(16) => None,
            Some(i) => Some(i + 2),
            None => Some(0),
        };

        self.state.select(i);
    }
    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(0) => None,
            Some(i) => Some(i.saturating_sub(2)),
            None => Some(16),
        };

        self.state.select(i);
    }

    pub fn set_last(&mut self) {
        self.state.select(Some(16));
    }

    pub fn selected(&self) -> bool {
        self.state.selected().is_some()
    }

    pub fn apply(&mut self, network_name: &str) -> Result<()> {
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
EAP-Method=PEAP
EAP-Identity={}
",
            self.identity.field.value()
        );

        if !self.server_domain_mask.is_empty() {
            text.push_str(
                format!(
                    "EAP-PEAP-ServerDomainMask={}
",
                    self.server_domain_mask.value()
                )
                .as_str(),
            );
        }

        if !self.ca_cert.is_empty() {
            text.push_str(
                format!(
                    "EAP-PEAP-CACert={}
",
                    self.ca_cert.field.value()
                )
                .as_str(),
            );
        }

        if !self.client_cert.is_empty() {
            text.push_str(
                format!(
                    "EAP-PEAP-ClientCert={}
",
                    self.client_cert.field.value()
                )
                .as_str(),
            );
        }

        if !self.client_key.is_empty() {
            text.push_str(
                format!(
                    "EAP-PEAP-ClientKey={}
",
                    self.client_key.field.value()
                )
                .as_str(),
            );
        }

        if !self.key_passphrase.is_empty() {
            text.push_str(
                format!(
                    "EAP-PEAP-ClientKeyPassphrase={}
",
                    self.key_passphrase.field.value()
                )
                .as_str(),
            );
        }

        text.push_str(
            format!(
                "EAP-PEAP-Phase2-Method={}
EAP-TTLS-Phase2-Identity={}
EAP-TTLS-Phase2-Password={}
",
                self.phase2_method,
                self.phase2_identity.value(),
                self.phase2_password.value()
            )
            .as_str(),
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
                FocusedInput::ServerDomainMask => {
                    self.server_domain_mask
                        .field
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedInput::CaCert => {
                    self.ca_cert
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
                FocusedInput::Phase2Method => match key_event.code {
                    KeyCode::Char('l') | KeyCode::Right => match self.phase2_method {
                        Phase2Method::MSCHAPV2 => self.phase2_method = Phase2Method::SIM,
                        Phase2Method::SIM => self.phase2_method = Phase2Method::GTC,
                        Phase2Method::GTC => self.phase2_method = Phase2Method::MSCHAPV2,
                    },
                    KeyCode::Char('h') | KeyCode::Left => {}
                    _ => {}
                },
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
            Line::from(vec![
                Span::from(pad_string(" Phase2 Method", 20))
                    .bold()
                    .bg(Color::DarkGray),
                Span::from("  "),
                Span::from(format!("< {} >", self.phase2_method)),
            ]),
            Line::from(vec![Span::from(ERROR_PADDING), Span::from("")]).red(),
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

        frame.render_stateful_widget(
            list,
            area.inner(Margin {
                horizontal: 2,
                vertical: 0,
            }),
            &mut self.state,
        );
    }
}
