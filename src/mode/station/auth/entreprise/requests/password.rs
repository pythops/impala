use crate::agent::AuthAgent;
use anyhow::Result;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, List},
};
use tui_input::Input;

#[derive(Debug)]
pub struct RequestPassword {
    pub password: Input,
    pub show_password: bool,
    network_name: String,
    user_name: Option<String>,
}

impl RequestPassword {
    pub fn new(network_name: String, user_name: Option<String>) -> Self {
        Self {
            password: Input::default(),
            show_password: true,
            network_name,
            user_name,
        }
    }
    pub async fn submit(&mut self, agent: &AuthAgent) -> Result<()> {
        let passkey: String = self.password.value().into();
        agent.tx_passphrase.send(passkey).await?;
        agent
            .password_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.password.reset();
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> Result<()> {
        agent.tx_cancel.send(()).await?;
        agent
            .password_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
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
            {
                if let Some(user_name) = &self.user_name {
                    Line::from(vec![
                        Span::raw(" Username ").bold().bg(Color::DarkGray),
                        Span::from("  "),
                        Span::from(user_name),
                    ])
                } else {
                    Line::from("")
                }
            },
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
