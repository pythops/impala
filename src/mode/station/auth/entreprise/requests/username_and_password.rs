use crate::agent::AuthAgent;
use crate::event::Event;
use anyhow::Result;
use tokio::sync::mpsc::UnboundedSender;

use crossterm::event::{KeyCode, KeyEvent};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List},
};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

#[derive(Debug, PartialEq)]
enum FocusedSection {
    Username,
    Password,
    Submit,
}

#[derive(Debug)]
pub struct RequestUsernameAndPassword {
    pub password: Input,
    pub username: Input,
    pub show_password: bool,
    focused_section: FocusedSection,
    network_name: String,
}

impl RequestUsernameAndPassword {
    pub fn new(network_name: String) -> Self {
        Self {
            password: Input::default(),
            username: Input::default(),
            show_password: true,
            focused_section: FocusedSection::Username,
            network_name,
        }
    }

    pub async fn handle_key_events(
        &mut self,
        key_event: KeyEvent,
        sender: UnboundedSender<Event>,
    ) -> Result<()> {
        match key_event.code {
            KeyCode::Tab => match self.focused_section {
                FocusedSection::Username => {
                    self.focused_section = FocusedSection::Password;
                }
                FocusedSection::Password => {
                    self.focused_section = FocusedSection::Submit;
                }
                FocusedSection::Submit => {
                    self.focused_section = FocusedSection::Username;
                }
            },
            KeyCode::BackTab => match self.focused_section {
                FocusedSection::Username => {
                    self.focused_section = FocusedSection::Submit;
                }
                FocusedSection::Password => {
                    self.focused_section = FocusedSection::Username;
                }
                FocusedSection::Submit => {
                    self.focused_section = FocusedSection::Password;
                }
            },
            _ => match self.focused_section {
                FocusedSection::Username => {
                    self.username
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedSection::Password => {
                    self.password
                        .handle_event(&crossterm::event::Event::Key(key_event));
                }
                FocusedSection::Submit => {
                    sender.send(Event::UsernameAndPasswordSubmit)?;
                }
            },
        }
        Ok(())
    }
    pub async fn submit(&mut self, agent: &AuthAgent) -> Result<()> {
        let username: String = self.username.value().into();
        let password: String = self.password.value().into();
        agent
            .tx_username_password
            .send((username, password))
            .await?;
        agent
            .username_and_password_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> Result<()> {
        agent.tx_cancel.send(()).await?;
        agent
            .username_and_password_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.username.reset();
        self.password.reset();
        Ok(())
    }
    pub fn render(&self, frame: &mut Frame) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(80),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(popup_layout[1])[1];

        let (title_area, form_area, show_password_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(5), Constraint::Length(5)])
                .split(area);

            let area2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Percentage(20),
                ])
                .flex(ratatui::layout::Flex::Center)
                .split(chunks[1]);

            (chunks[0], area2[1], area2[2])
        };

        let title = Line::from(vec![
            Span::raw("Authentication Required for "),
            Span::from(&self.network_name).bold(),
        ])
        .centered();

        let items = vec![
            Line::from(vec![
                Span::raw(" Username ").bold().bg(Color::DarkGray),
                Span::from("  "),
                Span::from(self.username.value()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw(" Password ").bold().bg(Color::DarkGray),
                Span::from("  "),
                Span::from({
                    if self.show_password {
                        self.password.value().to_string()
                    } else {
                        "*".repeat(self.password.value().len())
                    }
                }),
            ]),
        ];

        let form = List::new(items);

        let show_password_icon = if self.show_password {
            Text::from("\n\n ").centered()
        } else {
            Text::from("\n\n ").centered()
        };

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(
            title,
            title_area.inner(Margin {
                horizontal: 1,
                vertical: 2,
            }),
        );
        frame.render_widget(
            form,
            form_area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
        frame.render_widget(
            show_password_icon,
            show_password_area.inner(Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
    }
}
