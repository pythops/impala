use iwdrs::modes::Mode;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

#[derive(Debug)]
pub struct Reset {
    pub enable: bool,
    pub selected_mode: Mode,
    pub current_mode: Mode,
}

impl Reset {
    pub fn new(current_mode: Mode) -> Self {
        Self {
            enable: false,
            selected_mode: Mode::Station,
            current_mode,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(10),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(50),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(popup_layout[1])[1];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        let (message_area, station_choice_area, ap_choice_area, help_area) =
            (chunks[1], chunks[2], chunks[3], chunks[6]);

        let station_choice_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(2),
            ])
            .split(station_choice_area)[1];

        let ap_choice_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(2),
                Constraint::Fill(1),
                Constraint::Length(2),
            ])
            .split(ap_choice_area)[1];

        let message_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(message_area)[1];

        let (ap_text, station_text) = match self.selected_mode {
            Mode::Ap => match self.current_mode {
                Mode::Ap => (
                    Text::from("  Access Point (current)"),
                    Text::from("   Station"),
                ),
                Mode::Station => (
                    Text::from("  Access Point"),
                    Text::from("   Station (current)"),
                ),
            },
            Mode::Station => match self.current_mode {
                Mode::Ap => (
                    Text::from("   Access Point (current)"),
                    Text::from("  Station"),
                ),
                Mode::Station => (
                    Text::from("   Access Point"),
                    Text::from("  Station (current)"),
                ),
            },
        };

        let message = Paragraph::new("Select the desired mode:")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::uniform(1)));

        let station_choice = Paragraph::new(station_text)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::horizontal(10)));

        let ap_choice = Paragraph::new(ap_text)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::horizontal(10)));

        let help = Paragraph::new(
            Text::from(" Scroll down: j | Scroll up: k | Enter: Confirm ")
                .style(Style::default().blue()),
        )
        .alignment(Alignment::Center)
        .style(Style::default())
        .block(Block::new().padding(Padding::horizontal(1)));

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(message, message_area);
        frame.render_widget(ap_choice, ap_choice_area);
        frame.render_widget(station_choice, station_choice_area);
        frame.render_widget(help, help_area);
    }
}
