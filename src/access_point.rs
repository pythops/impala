use std::sync::{atomic::AtomicBool, Arc};

use anyhow::Result;
use iwdrs::access_point::AccessPoint as iwdAccessPoint;
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::{
    app::AppResult,
    event::Event,
    notification::{Notification, NotificationLevel},
};
use tui_input::Input;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum APFocusedSection {
    SSID,
    PSK,
}

#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub a: iwdAccessPoint,
    pub has_started: bool,
    pub name: Option<String>,
    pub frequency: Option<u32>,
    pub is_scanning: Option<bool>,
    pub supported_ciphers: Option<Vec<String>>,
    pub used_cipher: Option<String>,
    pub ap_start: Arc<AtomicBool>,
    pub ssid: Input,
    pub psk: Input,
    pub focused_section: APFocusedSection,
}

impl AccessPoint {
    pub async fn new(a: iwdAccessPoint) -> Result<Self> {
        let has_started = a.has_started().await?;
        let name = a.name().await?;
        let frequency = a.frequency().await?;
        let is_scanning = a.is_scanning().await.ok();
        let supported_ciphers = a.pairwise_ciphers().await?;
        let used_cipher = a.group_cipher().await?;
        let ap_start = Arc::new(AtomicBool::new(false));

        let ssid = Input::default();
        let psk = Input::default();
        let focused_section = APFocusedSection::SSID;

        Ok(Self {
            a,
            has_started,
            name,
            frequency,
            is_scanning,
            supported_ciphers,
            used_cipher,
            ap_start,
            ssid,
            psk,
            focused_section,
        })
    }

    pub fn render_input(&self, frame: &mut Frame) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Min(7),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((frame.size().width - 80) / 2),
                    Constraint::Min(80),
                    Constraint::Length((frame.size().width - 80) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

        let ((ssid_msg_area, ssid_input_area), (psk_msg_area, psk_input_area)) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(area);

            let ssid_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Percentage(45),
                        Constraint::Percentage(45),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(chunks[1]);

            let psk_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Percentage(45),
                        Constraint::Percentage(45),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(chunks[3]);

            (
                (ssid_chunks[1], ssid_chunks[2]),
                (psk_chunks[1], psk_chunks[2]),
            )
        };

        let ssid_text = match self.focused_section {
            APFocusedSection::SSID => Text::from("> SSID name"),
            _ => Text::from("  SSID name"),
        };

        let psk_text = match self.focused_section {
            APFocusedSection::PSK => Text::from("> SSID password"),
            _ => Text::from("  SSID password"),
        };

        let ssid_msg = Paragraph::new(ssid_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::left(2)));

        let ssid_input = Paragraph::new(self.ssid.value())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::new().style(Style::default().bg(Color::DarkGray)));

        let psk_msg = Paragraph::new(psk_text)
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::left(2)));

        let psk_input = Paragraph::new(self.psk.value())
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::new().style(Style::default().bg(Color::DarkGray)));

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(ssid_msg, ssid_msg_area);
        frame.render_widget(ssid_input, ssid_input_area);

        frame.render_widget(psk_msg, psk_msg_area);
        frame.render_widget(psk_input, psk_input_area);
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.has_started = self.a.has_started().await?;
        self.name = self.a.name().await?;
        self.frequency = self.a.frequency().await?;
        self.is_scanning = self.a.is_scanning().await.ok();
        self.supported_ciphers = self.a.pairwise_ciphers().await?;
        self.used_cipher = self.a.group_cipher().await?;

        Ok(())
    }

    pub async fn scan(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.a.scan().await {
            Ok(_) => Notification::send(
                "Start Scanning".to_string(),
                NotificationLevel::Info,
                sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?,
        }

        Ok(())
    }

    pub async fn start(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.a.start(self.ssid.value(), self.psk.value()).await {
            Ok(_) => Notification::send(
                format!("AP Started\nSSID: {}", self.ssid.value()),
                NotificationLevel::Info,
                sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?,
        }
        self.ap_start
            .store(false, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub async fn stop(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.a.stop().await {
            Ok(_) => Notification::send("AP Stopped".to_string(), NotificationLevel::Info, sender)?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?,
        }

        Ok(())
    }
}
