use crate::event::Event;

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear},
};

use tui_input::{Input, backend::crossterm::EventHandler};

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

    fn len(&self) -> usize {
        self.field.value().len()
    }
}
#[derive(Clone, Default)]
pub struct ConnectHiddenNetwork {
    ssid: UserInputField,
}

impl ConnectHiddenNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    fn validate(&mut self) {
        if self.ssid.is_empty() {
            self.ssid.error = Some("SSID can not be empty".to_string());
        }
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent, sender: UnboundedSender<Event>) {
        match key_event.code {
            KeyCode::Enter => {
                self.validate();

                if self.ssid.error.is_none() {
                    let _ =
                        sender.send(Event::ConnectToHiddenNetwork(self.ssid.value().to_string()));
                }
            }
            _ => {
                self.ssid
                    .field
                    .handle_event(&crossterm::event::Event::Key(key_event));
            }
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
                Constraint::Length(80),
                Constraint::Fill(1),
            ])
            .flex(ratatui::layout::Flex::SpaceBetween)
            .split(popup_layout[1])[1];

        let (message_area, ssid_area, error_area) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(2), // margin
                    Constraint::Length(1), // msg
                    Constraint::Length(1), // margin
                    Constraint::Length(1), // ssid
                    Constraint::Length(1), // margin
                    Constraint::Length(1), // error
                    Constraint::Length(2), // margin
                ])
                .split(area);

            (chunks[1], chunks[3], chunks[5])
        };

        let message = Text::from("Enter the SSID of the hidden network").centered();

        let ssid = Text::from(self.ssid.value()).bg(Color::DarkGray).centered();
        let error = Text::from(self.ssid.error.clone().unwrap_or_default())
            .red()
            .centered();

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(
            message,
            message_area.inner(Margin {
                horizontal: 2,
                vertical: 0,
            }),
        );
        frame.render_widget(
            ssid,
            ssid_area.inner(Margin {
                horizontal: 2,
                vertical: 0,
            }),
        );

        frame.render_widget(
            error,
            error_area.inner(Margin {
                horizontal: 2,
                vertical: 0,
            }),
        );

        let ssid_len = self.ssid.len();

        let inner_width = ssid_area.width.saturating_sub(2) as usize;
        let pad_left = if inner_width > ssid_len {
            inner_width.saturating_sub(ssid_len) / 2
        } else {
            0
        };

        let visual_cursor = self.ssid.field.visual_cursor().min(ssid_len);

        let x_in_inner = pad_left + visual_cursor;

        let cursor_x = ssid_area.x + 1 + x_in_inner as u16;
        frame.set_cursor_position((cursor_x, ssid_area.y));
    }
}
