use anyhow::Result;
use anyhow::anyhow;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Clear},
};
use tui_input::Input;

use crate::event::Event;

pub mod eduroam;
pub mod peap;
pub mod pwd;
pub mod requests;
pub mod tls;
pub mod ttls;

const ERROR_PADDING: &str = "                      ";

fn pad_string(input: &str, length: usize) -> String {
    let current_length = input.chars().count();
    if current_length >= length {
        input.to_string()
    } else {
        format!("{:<width$}", input, width = length)
    }
}

#[derive(Debug, Clone, Default)]
struct UserInputField {
    field: Input,
    error: Option<String>,
}

impl UserInputField {
    fn is_empty(&self) -> bool {
        self.field.value().is_empty()
    }

    fn value(&self) -> &str {
        self.field.value()
    }
}

#[derive(Debug, PartialEq)]
enum FocusedSection {
    EapChoice,
    Eap,
    Apply,
}

#[derive(Debug)]
pub struct WPAEntreprise {
    pub eap: Eap,
    pub network_name: String,
    focused_section: FocusedSection,
}

#[derive(Debug)]
pub enum Eap {
    TTLS(ttls::TTLS),
    PEAP(peap::PEAP),
    PWD(pwd::PWD),
    TLS(tls::TLS),
    Eduroam(eduroam::Eduroam),
}

impl Default for Eap {
    fn default() -> Self {
        Self::new()
    }
}
impl Eap {
    pub fn new() -> Self {
        Self::TLS(tls::TLS::new())
    }
}

impl WPAEntreprise {
    pub fn new(network_name: String) -> Self {
        Self {
            eap: Eap::new(),
            network_name,
            focused_section: FocusedSection::EapChoice,
        }
    }
    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        sender: UnboundedSender<Event>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Tab => match self.focused_section {
                FocusedSection::EapChoice => {
                    self.focused_section = FocusedSection::Eap;
                    match &mut self.eap {
                        Eap::TLS(v) => {
                            v.focused_input = tls::FocusedInput::CaCert;
                            v.next();
                        }
                        Eap::TTLS(v) => {
                            v.focused_input = ttls::FocusedInput::Identity;
                            v.next();
                        }
                        Eap::PEAP(v) => {
                            v.focused_input = peap::FocusedInput::Identity;
                            v.next();
                        }
                        Eap::PWD(v) => {
                            v.focused_input = pwd::FocusedInput::Identity;
                            v.next();
                        }
                        Eap::Eduroam(v) => {
                            v.focused_input = eduroam::FocusedInput::Identity;
                            v.next();
                        }
                    };
                }
                FocusedSection::Eap => match &mut self.eap {
                    Eap::TLS(v) => match v.focused_input {
                        tls::FocusedInput::CaCert => {
                            v.focused_input = tls::FocusedInput::Identity;
                            v.next();
                        }
                        tls::FocusedInput::Identity => {
                            v.focused_input = tls::FocusedInput::ClientCert;
                            v.next();
                        }
                        tls::FocusedInput::ClientCert => {
                            v.focused_input = tls::FocusedInput::ClientKey;
                            v.next();
                        }
                        tls::FocusedInput::ClientKey => {
                            v.focused_input = tls::FocusedInput::KeyPassphrase;
                            v.next();
                        }
                        tls::FocusedInput::KeyPassphrase => {
                            self.focused_section = FocusedSection::Apply;
                            v.deselect();
                        }
                    },
                    Eap::TTLS(v) => match v.focused_input {
                        ttls::FocusedInput::Identity => {
                            v.focused_input = ttls::FocusedInput::ServerDomainMask;
                            v.next();
                        }
                        ttls::FocusedInput::ServerDomainMask => {
                            v.focused_input = ttls::FocusedInput::CaCert;
                            v.next();
                        }
                        ttls::FocusedInput::CaCert => {
                            v.focused_input = ttls::FocusedInput::ClientCert;
                            v.next();
                        }
                        ttls::FocusedInput::ClientCert => {
                            v.focused_input = ttls::FocusedInput::ClientKey;
                            v.next();
                        }
                        ttls::FocusedInput::ClientKey => {
                            v.focused_input = ttls::FocusedInput::KeyPassphrase;
                            v.next();
                        }
                        ttls::FocusedInput::KeyPassphrase => {
                            v.focused_input = ttls::FocusedInput::Phase2Method;
                            v.next();
                        }
                        ttls::FocusedInput::Phase2Method => {
                            v.focused_input = ttls::FocusedInput::Phase2Identity;
                            v.next();
                        }
                        ttls::FocusedInput::Phase2Identity => {
                            v.focused_input = ttls::FocusedInput::Phase2Password;
                            v.next();
                        }
                        ttls::FocusedInput::Phase2Password => {
                            v.focused_input = ttls::FocusedInput::CaCert;
                            self.focused_section = FocusedSection::Apply;
                            v.next();
                        }
                    },
                    Eap::PEAP(v) => match v.focused_input {
                        peap::FocusedInput::Identity => {
                            v.focused_input = peap::FocusedInput::ServerDomainMask;
                            v.next();
                        }
                        peap::FocusedInput::ServerDomainMask => {
                            v.focused_input = peap::FocusedInput::CaCert;
                            v.next();
                        }
                        peap::FocusedInput::CaCert => {
                            v.focused_input = peap::FocusedInput::ClientCert;
                            v.next();
                        }
                        peap::FocusedInput::ClientCert => {
                            v.focused_input = peap::FocusedInput::ClientKey;
                            v.next();
                        }
                        peap::FocusedInput::ClientKey => {
                            v.focused_input = peap::FocusedInput::KeyPassphrase;
                            v.next();
                        }
                        peap::FocusedInput::KeyPassphrase => {
                            v.focused_input = peap::FocusedInput::Phase2Method;
                            v.next();
                        }
                        peap::FocusedInput::Phase2Method => {
                            v.focused_input = peap::FocusedInput::Phase2Identity;
                            v.next();
                        }
                        peap::FocusedInput::Phase2Identity => {
                            v.focused_input = peap::FocusedInput::Phase2Password;
                            v.next();
                        }
                        peap::FocusedInput::Phase2Password => {
                            v.focused_input = peap::FocusedInput::CaCert;
                            self.focused_section = FocusedSection::Apply;
                            v.next();
                        }
                    },
                    Eap::PWD(v) => match v.focused_input {
                        pwd::FocusedInput::Identity => {
                            v.focused_input = pwd::FocusedInput::Password;
                            v.next();
                        }
                        pwd::FocusedInput::Password => {
                            v.focused_input = pwd::FocusedInput::Identity;
                            self.focused_section = FocusedSection::Apply;
                            v.next();
                        }
                    },
                    Eap::Eduroam(v) => match v.focused_input {
                        eduroam::FocusedInput::Identity => {
                            v.focused_input = eduroam::FocusedInput::Phase2Identity;
                            v.next();
                        }
                        eduroam::FocusedInput::Phase2Identity => {
                            v.focused_input = eduroam::FocusedInput::Phase2Password;
                            v.next();
                        }
                        eduroam::FocusedInput::Phase2Password => {
                            v.focused_input = eduroam::FocusedInput::Identity;
                            self.focused_section = FocusedSection::Apply;
                            v.next();
                        }
                    },
                },
                FocusedSection::Apply => self.focused_section = FocusedSection::EapChoice,
            },
            KeyCode::BackTab => match self.focused_section {
                FocusedSection::EapChoice => self.focused_section = FocusedSection::Apply,
                FocusedSection::Eap => match &mut self.eap {
                    Eap::TLS(v) => match v.focused_input {
                        tls::FocusedInput::CaCert => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        tls::FocusedInput::Identity => {
                            v.focused_input = tls::FocusedInput::CaCert;
                            v.previous();
                        }
                        tls::FocusedInput::ClientCert => {
                            v.focused_input = tls::FocusedInput::Identity;
                            v.previous();
                        }
                        tls::FocusedInput::ClientKey => {
                            v.focused_input = tls::FocusedInput::ClientCert;
                            v.previous();
                        }
                        tls::FocusedInput::KeyPassphrase => {
                            v.focused_input = tls::FocusedInput::ClientKey;
                            v.previous();
                        }
                    },
                    Eap::TTLS(v) => match v.focused_input {
                        ttls::FocusedInput::Identity => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        ttls::FocusedInput::ServerDomainMask => {
                            v.focused_input = ttls::FocusedInput::Identity;
                            v.previous();
                        }
                        ttls::FocusedInput::CaCert => {
                            v.focused_input = ttls::FocusedInput::ServerDomainMask;
                            v.previous();
                        }
                        ttls::FocusedInput::ClientCert => {
                            v.focused_input = ttls::FocusedInput::CaCert;
                            v.previous();
                        }
                        ttls::FocusedInput::ClientKey => {
                            v.focused_input = ttls::FocusedInput::ClientCert;
                            v.previous();
                        }
                        ttls::FocusedInput::KeyPassphrase => {
                            v.focused_input = ttls::FocusedInput::ClientKey;
                            v.previous();
                        }
                        ttls::FocusedInput::Phase2Method => {
                            v.focused_input = ttls::FocusedInput::KeyPassphrase;
                            v.previous();
                        }
                        ttls::FocusedInput::Phase2Identity => {
                            v.focused_input = ttls::FocusedInput::Phase2Method;
                            v.previous();
                        }
                        ttls::FocusedInput::Phase2Password => {
                            v.focused_input = ttls::FocusedInput::Phase2Identity;
                            v.previous();
                        }
                    },
                    Eap::PEAP(v) => match v.focused_input {
                        peap::FocusedInput::Identity => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        peap::FocusedInput::ServerDomainMask => {
                            v.focused_input = peap::FocusedInput::Identity;
                            v.previous();
                        }
                        peap::FocusedInput::CaCert => {
                            v.focused_input = peap::FocusedInput::ServerDomainMask;
                            v.previous();
                        }
                        peap::FocusedInput::ClientCert => {
                            v.focused_input = peap::FocusedInput::CaCert;
                            v.previous();
                        }
                        peap::FocusedInput::ClientKey => {
                            v.focused_input = peap::FocusedInput::ClientCert;
                            v.previous();
                        }
                        peap::FocusedInput::KeyPassphrase => {
                            v.focused_input = peap::FocusedInput::ClientKey;
                            v.previous();
                        }
                        peap::FocusedInput::Phase2Method => {
                            v.focused_input = peap::FocusedInput::KeyPassphrase;
                            v.previous();
                        }
                        peap::FocusedInput::Phase2Identity => {
                            v.focused_input = peap::FocusedInput::Phase2Method;
                            v.previous();
                        }
                        peap::FocusedInput::Phase2Password => {
                            v.focused_input = peap::FocusedInput::Phase2Identity;
                            v.previous();
                        }
                    },
                    Eap::PWD(v) => match v.focused_input {
                        pwd::FocusedInput::Identity => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        pwd::FocusedInput::Password => {
                            v.focused_input = pwd::FocusedInput::Identity;
                            v.previous();
                        }
                    },
                    Eap::Eduroam(v) => match v.focused_input {
                        eduroam::FocusedInput::Identity => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        eduroam::FocusedInput::Phase2Identity => {
                            v.focused_input = eduroam::FocusedInput::Identity;
                            v.previous();
                        }
                        eduroam::FocusedInput::Phase2Password => {
                            v.focused_input = eduroam::FocusedInput::Phase2Identity;
                            v.previous();
                        }
                    },
                },
                FocusedSection::Apply => match &mut self.eap {
                    Eap::TLS(v) => {
                        v.focused_input = tls::FocusedInput::KeyPassphrase;
                        self.focused_section = FocusedSection::Eap;
                        v.set_last();
                    }
                    Eap::TTLS(v) => {
                        v.focused_input = ttls::FocusedInput::Phase2Password;
                        self.focused_section = FocusedSection::Eap;
                        v.set_last();
                    }
                    Eap::PEAP(v) => {
                        v.focused_input = peap::FocusedInput::Phase2Password;
                        self.focused_section = FocusedSection::Eap;
                        v.set_last();
                    }
                    Eap::PWD(v) => {
                        v.focused_input = pwd::FocusedInput::Password;
                        self.focused_section = FocusedSection::Eap;
                        v.set_last();
                    }
                    Eap::Eduroam(v) => {
                        v.focused_input = eduroam::FocusedInput::Phase2Password;
                        self.focused_section = FocusedSection::Eap;
                        v.set_last();
                    }
                },
            },
            _ => match self.focused_section {
                // TLS => TTLS =>  PEAP => PWD => Eduroam
                FocusedSection::EapChoice => match key_event.code {
                    KeyCode::Char('l') | KeyCode::Right => match self.eap {
                        Eap::TLS(_) => self.eap = Eap::TTLS(ttls::TTLS::new()),
                        Eap::TTLS(_) => self.eap = Eap::PEAP(peap::PEAP::new()),
                        Eap::PEAP(_) => self.eap = Eap::PWD(pwd::PWD::new()),
                        Eap::PWD(_) => self.eap = Eap::Eduroam(eduroam::Eduroam::new()),
                        Eap::Eduroam(_) => self.eap = Eap::TLS(tls::TLS::new()),
                    },
                    KeyCode::Char('h') | KeyCode::Left => match self.eap {
                        Eap::Eduroam(_) => self.eap = Eap::PWD(pwd::PWD::new()),
                        Eap::PWD(_) => self.eap = Eap::PEAP(peap::PEAP::new()),
                        Eap::PEAP(_) => self.eap = Eap::TTLS(ttls::TTLS::new()),
                        Eap::TTLS(_) => self.eap = Eap::TLS(tls::TLS::new()),
                        Eap::TLS(_) => self.eap = Eap::Eduroam(eduroam::Eduroam::new()),
                    },

                    _ => {}
                },
                FocusedSection::Eap => match &mut self.eap {
                    Eap::TLS(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::TTLS(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::PEAP(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::PWD(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::Eduroam(v) => v.handle_key_events(key_event, sender).await?,
                },

                FocusedSection::Apply => {
                    if let KeyCode::Enter = key_event.code {
                        if unsafe { libc::geteuid() } != 0 {
                            return Err(anyhow!(
                                "impala must be run as root to configure WPA Entreprise networks"
                            ));
                        }

                        let result = match &mut self.eap {
                            Eap::TLS(v) => v.apply(self.network_name.as_str()),
                            Eap::TTLS(v) => v.apply(self.network_name.as_str()),
                            Eap::PEAP(v) => v.apply(self.network_name.as_str()),
                            Eap::PWD(v) => v.apply(self.network_name.as_str()),
                            Eap::Eduroam(v) => v.apply(),
                        };
                        if result.is_ok() {
                            sender.send(Event::Tick)?;
                            sender.send(Event::EapNeworkConfigured(self.network_name.clone()))?;
                        }
                    }
                }
            },
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(30),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Max(80),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        frame.render_widget(Clear, block);

        let (title_block, eap_choice_block, eap_block, apply_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2),
                    Constraint::Length(1), // Title
                    Constraint::Length(2),
                    Constraint::Length(1), // Eap choice
                    Constraint::Length(2),
                    Constraint::Length(30), // Form
                    Constraint::Length(2),
                    Constraint::Length(1), // Submit
                    Constraint::Length(2),
                ])
                .split(block);

            (chunks[1], chunks[3], chunks[5], chunks[7])
        };

        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().green()),
            block,
        );

        let title = Text::from(format!("Configure the network {}", self.network_name))
            .centered()
            .bold();
        frame.render_widget(title, title_block);

        let choice = match self.eap {
            Eap::TTLS(_) => Text::from("< TTLS >").centered(),
            Eap::PEAP(_) => Text::from("< PEAP >").centered(),
            Eap::PWD(_) => Text::from("< PWD >").centered(),
            Eap::TLS(_) => Text::from("< TLS >").centered(),
            Eap::Eduroam(_) => Text::from("< Eduroam >").centered(),
        };

        let choice = if self.focused_section == FocusedSection::EapChoice {
            choice.bold().green()
        } else {
            choice
        };

        frame.render_widget(
            choice.centered(),
            eap_choice_block.inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );

        match &mut self.eap {
            Eap::TLS(v) => {
                v.render(frame, eap_block);
            }
            Eap::PWD(v) => {
                v.render(frame, eap_block);
            }
            Eap::TTLS(v) => {
                v.render(frame, eap_block);
            }
            Eap::PEAP(v) => {
                v.render(frame, eap_block);
            }
            Eap::Eduroam(v) => {
                v.render(frame, eap_block);
            }
        }

        let text = if self.focused_section == FocusedSection::Apply {
            Text::from("APPLY").centered().green().bold()
        } else {
            Text::from("APPLY").centered()
        };

        frame.render_widget(
            text,
            apply_block.inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );
    }
}
