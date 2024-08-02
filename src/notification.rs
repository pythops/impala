use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{app::AppResult, event::Event};

use crate::tui::Palette;

#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub ttl: u16,
}

#[derive(Debug, Clone)]
pub enum NotificationLevel {
    Error,
    Warning,
    Info,
}

impl Notification {
    pub fn render(&self, index: usize, palette: &Palette, frame: &mut Frame) {
        let (style, title) = match self.level {
            NotificationLevel::Info => (palette.notification_info, "Info"),
            NotificationLevel::Warning => (palette.notification_warning, "Warning"),
            NotificationLevel::Error => (palette.notification_error, "Error"),
        };

        let mut text = Text::from(vec![
            Line::from(title).style(style.add_modifier(Modifier::BOLD))
        ]);

        text.extend(Text::from(self.message.as_str()));

        let notification_height = text.height() as u16 + 2;
        let notification_width = text.width() as u16 + 4;

        let block = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default())
                    .border_type(BorderType::Thick)
                    .border_style(style),
            );

        let area = notification_rect(
            index as u16,
            notification_height,
            notification_width,
            frame.size(),
        );

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);
    }
    pub fn send(
        message: String,
        level: NotificationLevel,
        sender: UnboundedSender<Event>,
    ) -> AppResult<()> {
        let notif = Notification {
            message,
            level,
            ttl: 8,
        };

        sender.send(Event::Notification(notif))?;

        Ok(())
    }
}

pub fn notification_rect(offset: u16, height: u16, width: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(height * offset),
                Constraint::Length(height),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(width),
                Constraint::Length(2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
