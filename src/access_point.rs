use std::sync::{atomic::AtomicBool, Arc};

use iwdrs::session::Session;
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
    session: Arc<Session>,
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
    pub connected_devices: Vec<String>,
}

impl AccessPoint {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let iwd_access_point = session.access_point().unwrap();
        let iwd_access_point_diagnostic = session.access_point_diagnostic();

        let has_started = iwd_access_point.has_started().await?;
        let name = iwd_access_point.name().await?;
        let frequency = iwd_access_point.frequency().await?;
        let is_scanning = iwd_access_point.is_scanning().await.ok();
        let supported_ciphers = iwd_access_point.pairwise_ciphers().await?;
        let used_cipher = iwd_access_point.group_cipher().await?;
        let ap_start = Arc::new(AtomicBool::new(false));

        let ssid = Input::default();
        let psk = Input::default();
        let focused_section = APFocusedSection::SSID;

        let connected_devices = {
            if let Some(d) = iwd_access_point_diagnostic {
                match d.get().await {
                    Ok(diagnostic) => diagnostic
                        .iter()
                        .map(|v| v["Address"].clone().trim_matches('"').to_string())
                        .collect(),
                    Err(_) => Vec::new(),
                }
            } else {
                Vec::new()
            }
        };

        Ok(Self {
            session,
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
            connected_devices,
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
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((frame.area().width - 80) / 2),
                    Constraint::Min(80),
                    Constraint::Length((frame.area().width - 80) / 2),
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

    pub async fn refresh(&mut self) -> AppResult<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        let iwd_access_point_diagnostic = self.session.access_point_diagnostic();

        self.has_started = iwd_access_point.has_started().await?;
        self.name = iwd_access_point.name().await?;
        self.frequency = iwd_access_point.frequency().await?;
        self.is_scanning = iwd_access_point.is_scanning().await.ok();
        self.supported_ciphers = iwd_access_point.pairwise_ciphers().await?;
        self.used_cipher = iwd_access_point.group_cipher().await?;

        if let Some(d) = iwd_access_point_diagnostic {
            if let Ok(diagnostic) = d.get().await {
                self.connected_devices = diagnostic
                    .iter()
                    .map(|v| v["Address"].clone().trim_matches('"').to_string())
                    .collect();
            }
        }

        Ok(())
    }

    pub async fn scan(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        match iwd_access_point.scan().await {
            Ok(()) => Notification::send(
                "Start Scanning".to_string(),
                NotificationLevel::Info,
                &sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }

        Ok(())
    }

    pub async fn start(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        match iwd_access_point
            .start(self.ssid.value(), self.psk.value())
            .await
        {
            Ok(()) => Notification::send(
                format!("AP Started\nSSID: {}", self.ssid.value()),
                NotificationLevel::Info,
                &sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }
        self.ap_start
            .store(false, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    pub async fn stop(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        let iwd_access_point = self.session.access_point().unwrap();
        match iwd_access_point.stop().await {
            Ok(()) => {
                Notification::send("AP Stopped".to_string(), NotificationLevel::Info, &sender)?
            }
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }

        Ok(())
    }
}
