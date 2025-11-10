use anyhow::{Result, anyhow};
use std::sync::{Arc, atomic::AtomicBool};

use iwdrs::session::Session;
use tokio::sync::mpsc::UnboundedSender;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Clear, List, Padding, Paragraph, Row, Table, TableState,
    },
};

use crate::{
    app::FocusedBlock,
    config::Config,
    device::Device,
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
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let iwd_access_point = session
            .access_points()
            .await
            .unwrap()
            .pop()
            .ok_or(anyhow!("no ap found"))?;
        let iwd_access_point_diagnostic = session.access_points_diagnostics().await.unwrap().pop();

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
            if let Some(diagnostic) = iwd_access_point_diagnostic {
                match diagnostic.get().await {
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

    pub async fn refresh(&mut self) -> Result<()> {
        let iwd_access_point = self.session.access_points().await.unwrap().pop().unwrap();
        let iwd_access_point_diagnostic = self
            .session
            .access_points_diagnostics()
            .await
            .unwrap()
            .pop();

        self.has_started = iwd_access_point.has_started().await?;
        self.name = iwd_access_point.name().await?;
        self.frequency = iwd_access_point.frequency().await?;
        self.is_scanning = iwd_access_point.is_scanning().await.ok();
        self.supported_ciphers = iwd_access_point.pairwise_ciphers().await?;
        self.used_cipher = iwd_access_point.group_cipher().await?;

        if let Some(diagnostic) = iwd_access_point_diagnostic {
            if let Ok(diagnostic) = diagnostic.get().await {
                self.connected_devices = diagnostic
                    .iter()
                    .map(|v| v["Address"].clone().trim_matches('"').to_string())
                    .collect();
            }
        } else {
            self.connected_devices = Vec::new()
        };

        Ok(())
    }

    pub async fn scan(&self, sender: UnboundedSender<Event>) -> Result<()> {
        let iwd_access_point = self.session.access_points().await.unwrap().pop().unwrap();
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

    pub async fn start(&self, sender: UnboundedSender<Event>) -> Result<()> {
        let iwd_access_point = self.session.access_points().await.unwrap().pop().unwrap();
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

    pub async fn stop(&self, sender: UnboundedSender<Event>) -> Result<()> {
        let iwd_access_point = self.session.access_points().await.unwrap().pop().unwrap();
        match iwd_access_point.stop().await {
            Ok(()) => {
                Notification::send("AP Stopped".to_string(), NotificationLevel::Info, &sender)?
            }
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }

        Ok(())
    }

    pub fn render(
        &self,
        frame: &mut Frame,
        focused_block: FocusedBlock,
        device: &Device,
        config: Arc<Config>,
    ) {
        let (access_point_block, connected_devices_block, device_block, help_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if !self.connected_devices.is_empty() {
                    &[
                        Constraint::Length(5),
                        Constraint::Fill(1),
                        Constraint::Length(5),
                        Constraint::Length(1),
                    ]
                } else {
                    &[
                        Constraint::Fill(1),
                        Constraint::Length(0),
                        Constraint::Length(5),
                        Constraint::Length(1),
                    ]
                })
                .margin(1)
                .split(frame.area());
            (chunks[0], chunks[1], chunks[2], chunks[3])
        };

        // Device
        let row = Row::new(vec![
            Line::from(device.name.clone()).centered(),
            Line::from("Access Point").centered(),
            {
                if device.is_powered {
                    Line::from("On").centered()
                } else {
                    Line::from("Off").centered()
                }
            },
            Line::from(device.address.clone()).centered(),
        ]);

        let widths = [
            Constraint::Length(15),
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Length(17),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Line::from("Name").yellow().centered(),
                        Line::from("Mode").yellow().centered(),
                        Line::from("Powered").yellow().centered(),
                        Line::from("Address").yellow().centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Line::from("Name").centered(),
                        Line::from("Mode").centered(),
                        Line::from("Powered").centered(),
                        Line::from("Address").centered(),
                    ])
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(" Device ")
                    .title_style({
                        if focused_block == FocusedBlock::Device {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .padding(Padding::horizontal(1))
                    .border_style({
                        if focused_block == FocusedBlock::Device {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::Device {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(2)
            .flex(Flex::SpaceAround)
            .row_highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        // Access Point
        let ap_name = if self.has_started {
            self.name.as_ref().unwrap().clone()
        } else {
            "-".to_string()
        };

        let ap_frequency = if self.has_started {
            format!("{:.2} GHz", (self.frequency.unwrap() / 1000))
        } else {
            "-".to_string()
        };

        let ap_used_cipher = if self.has_started {
            self.used_cipher.as_ref().unwrap().clone()
        } else {
            "-".to_string()
        };

        let ap_is_scanning = if self.has_started {
            match self.is_scanning {
                Some(v) => v.to_string(),
                None => "-".to_string(),
            }
        } else {
            "-".to_string()
        };

        let row = Row::new(vec![
            Line::from(self.has_started.to_string()).centered(),
            Line::from(ap_name).centered(),
            Line::from(ap_frequency).centered(),
            Line::from(ap_used_cipher).centered(),
            Line::from(ap_is_scanning).centered(),
        ]);

        let widths = [
            Constraint::Length(10),
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let access_point_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::AccessPoint {
                    Row::new(vec![
                        Line::from("Started").yellow().centered(),
                        Line::from("SSID").yellow().centered(),
                        Line::from("Frequency").yellow().centered(),
                        Line::from("Cipher").yellow().centered(),
                        Line::from("Scanning").yellow().centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Line::from("Started").centered(),
                        Line::from("SSID").centered(),
                        Line::from("Frequency").centered(),
                        Line::from("Cipher").centered(),
                        Line::from("Scanning").centered(),
                    ])
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(" Access Point ")
                    .title_style({
                        if focused_block == FocusedBlock::AccessPoint {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if focused_block == FocusedBlock::AccessPoint {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::AccessPoint {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    })
                    .padding(Padding::horizontal(1)),
            )
            .column_spacing(2)
            .flex(Flex::SpaceAround)
            .row_highlight_style(if focused_block == FocusedBlock::AccessPoint {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        //TODO: maybe make it stateless
        let mut access_point_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(
            access_point_table,
            access_point_block,
            &mut access_point_state,
        );

        // Connected devices
        if !self.connected_devices.is_empty() {
            let connected_devices_list = List::new(self.connected_devices.clone()).block(
                Block::bordered()
                    .title("Connected Devices")
                    .title_style({
                        if focused_block == FocusedBlock::AccessPointConnectedDevices {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if focused_block == FocusedBlock::AccessPointConnectedDevices {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::AccessPointConnectedDevices {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    })
                    .padding(Padding::uniform(1)),
            );

            frame.render_widget(connected_devices_list, connected_devices_block);
        }

        let help_message = match focused_block {
            FocusedBlock::Device => Line::from(vec![
                Span::from(config.device.infos.to_string()).bold(),
                Span::from(" Infos"),
                Span::from(" | "),
                Span::from(config.device.toggle_power.to_string()).bold(),
                Span::from(" Toggle Power"),
                Span::from(" | "),
                Span::from("ctrl+r").bold(),
                Span::from(" Switch Mode"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            FocusedBlock::AdapterInfos | FocusedBlock::AccessPointInput => Line::from(vec![
                Span::from("󱊷 ").bold(),
                Span::from(" Discard"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            FocusedBlock::AccessPoint => Line::from(vec![
                Span::from(config.ap.start.to_string()).bold(),
                Span::from(" New AP"),
                Span::from(" | "),
                Span::from(config.ap.stop.to_string()).bold(),
                Span::from(" Stop AP"),
                Span::from(" | "),
                Span::from("ctrl+r").bold(),
                Span::from(" Switch Mode"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            _ => Line::from(""),
        };

        let help_message = help_message.centered().blue();
        frame.render_widget(help_message, help_block);
    }
}
