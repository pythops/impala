use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Margin},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Borders, Clear},
};

use crate::{app::AppResult, event::Event};

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
    ) -> AppResult<()> {
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
                            v.focused_input = ttls::FocusedInput::CaCert;
                            v.next();
                        }
                        Eap::PEAP(v) => {
                            v.focused_input = peap::FocusedInput::CaCert;
                            v.next();
                        }
                        Eap::PWD(v) => {
                            v.focused_input = pwd::FocusedInput::Identity;
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
                        ttls::FocusedInput::CaCert => {
                            v.focused_input = ttls::FocusedInput::ServerDomainMask;
                            v.next();
                        }
                        ttls::FocusedInput::ServerDomainMask => {
                            v.focused_input = ttls::FocusedInput::Identity;
                            v.next();
                        }
                        ttls::FocusedInput::Identity => {
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
                        peap::FocusedInput::CaCert => {
                            v.focused_input = peap::FocusedInput::ServerDomainMask;
                            v.next();
                        }
                        peap::FocusedInput::ServerDomainMask => {
                            v.focused_input = peap::FocusedInput::Identity;
                            v.next();
                        }
                        peap::FocusedInput::Identity => {
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
                        ttls::FocusedInput::CaCert => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        ttls::FocusedInput::ServerDomainMask => {
                            v.focused_input = ttls::FocusedInput::CaCert;
                            v.previous();
                        }
                        ttls::FocusedInput::Identity => {
                            v.focused_input = ttls::FocusedInput::ServerDomainMask;
                            v.previous();
                        }
                        ttls::FocusedInput::Phase2Identity => {
                            v.focused_input = ttls::FocusedInput::Identity;
                            v.previous();
                        }
                        ttls::FocusedInput::Phase2Password => {
                            v.focused_input = ttls::FocusedInput::Phase2Identity;
                            v.previous();
                        }
                    },
                    Eap::PEAP(v) => match v.focused_input {
                        peap::FocusedInput::CaCert => {
                            self.focused_section = FocusedSection::EapChoice;
                            v.previous();
                        }
                        peap::FocusedInput::ServerDomainMask => {
                            v.focused_input = peap::FocusedInput::CaCert;
                            v.previous();
                        }
                        peap::FocusedInput::Identity => {
                            v.focused_input = peap::FocusedInput::ServerDomainMask;
                            v.previous();
                        }
                        peap::FocusedInput::Phase2Identity => {
                            v.focused_input = peap::FocusedInput::Identity;
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
                },
            },
            _ => match self.focused_section {
                FocusedSection::EapChoice => match key_event.code {
                    KeyCode::Char('l') | KeyCode::Right => match self.eap {
                        Eap::TTLS(_) => self.eap = Eap::PEAP(peap::PEAP::new()),
                        Eap::PEAP(_) => self.eap = Eap::PWD(pwd::PWD::new()),
                        Eap::PWD(_) => self.eap = Eap::TLS(tls::TLS::new()),
                        Eap::TLS(_) => self.eap = Eap::TTLS(ttls::TTLS::new()),
                    },
                    KeyCode::Char('h') | KeyCode::Left => {}
                    _ => {}
                },
                FocusedSection::Eap => match &mut self.eap {
                    Eap::TLS(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::TTLS(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::PEAP(v) => v.handle_key_events(key_event, sender).await?,
                    Eap::PWD(v) => v.handle_key_events(key_event, sender).await?,
                },

                FocusedSection::Apply => {
                    if let KeyCode::Enter = key_event.code {
                        let result = match &mut self.eap {
                            Eap::TLS(v) => v.apply(self.network_name.as_str()),
                            Eap::TTLS(v) => v.apply(self.network_name.as_str()),
                            Eap::PEAP(v) => v.apply(self.network_name.as_str()),
                            Eap::PWD(v) => v.apply(self.network_name.as_str()),
                        };
                        if result.is_ok() {
                            sender.send(Event::EapNeworkConfigured)?;
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
                Constraint::Length(20),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Percentage(80),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(layout[1])[1];

        frame.render_widget(Clear, block);

        let (eap_choice_block, eap_block, apply_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(10),
                    Constraint::Length(4),
                ])
                .flex(Flex::SpaceBetween)
                .split(block);

            (chunks[0], chunks[1], chunks[2])
        };

        frame.render_widget(
            Block::default()
                .title(format!(" Configure {} Network ", self.network_name))
                .title_alignment(ratatui::layout::Alignment::Center)
                .title_style(Style::default().bold())
                .borders(Borders::ALL)
                .border_type(ratatui::widgets::BorderType::Thick)
                .border_style(Style::default().green()),
            block,
        );

        let choice = match self.eap {
            Eap::TTLS(_) => Text::from(" < TTLS >"),
            Eap::PEAP(_) => Text::from(" < PEAP >"),
            Eap::PWD(_) => Text::from(" < PWD >"),
            Eap::TLS(_) => Text::from(" < TLS >"),
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
                vertical: 2,
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
        }

        let text = if self.focused_section == FocusedSection::Apply {
            Text::from("Apply").bold().green().centered()
        } else {
            Text::from("Apply").centered()
        };

        frame.render_widget(
            text,
            apply_block.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }
}
