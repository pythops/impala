use crate::agent::AuthAgent;
use anyhow::Result;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};
use tui_input::Input;

#[derive(Debug)]
pub struct RequestKeyPassphrase {
    pub passphrase: Input,
    pub show_password: bool,
    network_name: String,
}

impl RequestKeyPassphrase {
    pub fn new(network_name: String) -> Self {
        Self {
            passphrase: Input::default(),
            show_password: true,
            network_name,
        }
    }
    pub async fn submit(&mut self, agent: &AuthAgent) -> Result<()> {
        let passkey: String = self.passphrase.value().into();
        agent.tx_passphrase.send(passkey).await?;
        agent
            .private_key_passphrase_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passphrase.reset();
        Ok(())
    }

    pub async fn cancel(&mut self, agent: &AuthAgent) -> Result<()> {
        agent.tx_cancel.send(()).await?;
        agent
            .private_key_passphrase_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passphrase.reset();
        Ok(())
    }
    pub fn render(&self, frame: &mut Frame) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(8),
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

        let (text_area, passkey_area, show_password_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ])
                .split(area);

            let area1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .flex(ratatui::layout::Flex::Center)
                .split(chunks[1]);

            let area2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Percentage(20),
                ])
                .flex(ratatui::layout::Flex::Center)
                .split(chunks[2]);

            (area1[1], area2[1], area2[2])
        };

        let text = Line::from(vec![
            Span::raw("Enter the key passphrase for "),
            Span::from(&self.network_name).bold(),
        ]);

        let text = Paragraph::new(text.centered())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::uniform(1)));

        let passkey = Paragraph::new({
            if self.show_password {
                self.passphrase.value().to_string()
            } else {
                "*".repeat(self.passphrase.value().len())
            }
        })
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
        .block(Block::new().style(Style::default().bg(Color::DarkGray)));

        let show_password_icon = if self.show_password {
            Text::from(" ").centered()
        } else {
            Text::from(" ").centered()
        };

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(text, text_area);
        frame.render_widget(passkey, passkey_area);
        frame.render_widget(show_password_icon, show_password_area);
    }
}
