use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::tui::Palette;

pub struct Auth;

impl Auth {
    pub fn render(&self, palette: &Palette, frame: &mut Frame, passkey: &str) {

        let width = if frame.size().width > 80 {
            (frame.size().width - 80) / 2
        } else {
            frame.size().width
        };

        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Min(8),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(width),
                    Constraint::Min(80),
                    Constraint::Length(width),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

        let (text_area, passkey_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(area);

            // (chunks[1], chunks[2])

            let area1 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Fill(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let area2 = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Fill(1),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(chunks[2]);

            (area1[1], area2[1])
        };

        let text = Paragraph::new("Enter the password")
            .alignment(Alignment::Center)
            .style(palette.text)
            .block(Block::new().padding(Padding::uniform(1)));

        let passkey = Paragraph::new(passkey)
            .style(palette.input_text)
            .block(Block::new().style(palette.input_box));

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .style(palette.active_border)
                .border_style(palette.active_border),
            area,
        );
        frame.render_widget(text, text_area);
        frame.render_widget(passkey, passkey_area);
    }
}
