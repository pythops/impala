use anyhow::Result;
use qrcode::QrCode;
use std::fs;
use tui_qrcode::{Colors, QrCodeWidget};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear},
};

use crate::iwd_network_name;

#[derive(Clone)]
pub struct Share {
    pub qr_code: QrCode,
    pub network_name: String,
}

impl Share {
    pub fn new(network_name: String) -> Result<Self> {
        let encoded_network_name = iwd_network_name(&network_name);
        let content = fs::read_to_string(format!("/var/lib/iwd/{}.psk", encoded_network_name))?;

        if let Some(line) = content
            .lines()
            .find(|&line| line.starts_with("Passphrase="))
            && let Some((_, passphrase)) = line.split_once('=')
        {
            let message = format!("WIFI:T:WPA;S:{network_name};P:{passphrase};;");
            let qr_code = QrCode::new(message)?;
            Ok(Self {
                qr_code,
                network_name,
            })
        } else {
            unreachable!()
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let widget = QrCodeWidget::new(self.qr_code.clone()).colors(Colors::Inverted);
        let sim_area = Rect::new(0, 0, 50, 50);
        let size = widget.size(sim_area);

        let block = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(size.height + 8),
                Constraint::Fill(1),
            ])
            .flex(Flex::SpaceBetween)
            .split(frame.area())[1];

        let block = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(size.width + 6),
                Constraint::Fill(1),
            ])
            .flex(Flex::SpaceBetween)
            .split(block)[1];

        let (title_block, qr_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Fill(1)])
                .margin(3)
                .flex(Flex::SpaceBetween)
                .split(block);

            (chunks[0], chunks[1])
        };

        frame.render_widget(Clear, block);
        frame.render_widget(
            Block::new()
                .borders(Borders::all())
                .border_type(BorderType::Thick)
                .border_style(Style::new().green()),
            block,
        );
        frame.render_widget(
            Text::from(self.network_name.clone()).centered().bold(),
            title_block,
        );
        frame.render_widget(widget, qr_block)
    }
}
