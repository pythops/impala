use std::sync::Arc;

use anyhow::Context;

use iwdrs::{adapter::Adapter as iwdAdapter, modes::Mode, session::Session};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, List, Padding, Row, Table, TableState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{AppResult, ColorMode, FocusedBlock},
    config::Config,
    device::Device,
    event::Event,
};

#[derive(Debug, Clone)]
pub struct Adapter {
    pub adapter: iwdAdapter,
    pub is_powered: bool,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
    pub device: Device,
    pub config: Arc<Config>,
}

impl Adapter {
    pub async fn new(
        session: Arc<Session>,
        sender: UnboundedSender<Event>,
        config: Arc<Config>,
    ) -> AppResult<Self> {
        let adapter = session.adapter().context("No adapter found")?;

        let is_powered = adapter.is_powered().await?;
        let name = adapter.name().await?;
        let model = adapter.model().await.ok();
        let vendor = adapter.vendor().await.ok();
        let supported_modes = adapter.supported_modes().await?;
        let device = Device::new(session.clone(), sender).await?;

        Ok(Self {
            adapter,
            is_powered,
            name,
            model,
            vendor,
            supported_modes,
            device,
            config,
        })
    }

    pub async fn refresh(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.is_powered = self.adapter.is_powered().await?;
        self.device.refresh(sender).await?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, color_mode: ColorMode, focused_block: FocusedBlock) {
        match self.device.mode {
            Mode::Station => {
                self.render_station_mode(frame, color_mode, focused_block);
            }
            Mode::Ap => {
                if self.device.access_point.is_some() {
                    self.render_access_point_mode(frame, color_mode, focused_block);
                }
            }
            _ => {}
        }
    }

    pub fn render_access_point_mode(
        &self,
        frame: &mut Frame,
        color_mode: ColorMode,
        focused_block: FocusedBlock,
    ) {
        let any_connected_devices = match self.device.access_point.as_ref() {
            Some(ap) => !ap.connected_devices.is_empty(),
            None => false,
        };

        let (access_point_block, connected_devices_block, device_block, help_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if any_connected_devices {
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
            Line::from(self.device.name.clone()).centered(),
            Line::from("Access Point").centered(),
            {
                if self.device.is_powered {
                    Line::from("On").centered()
                } else {
                    Line::from("Off").centered()
                }
            },
            Line::from(self.device.address.clone()).centered(),
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
                        Line::from("Name")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Mode")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Powered")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Address")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                    ])
                    .style(Style::new().bold())
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
            .flex(Flex::SpaceBetween)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        // Access Point

        let ap_name = match self.device.access_point.as_ref() {
            Some(ap) => {
                if ap.has_started {
                    ap.name.as_ref().unwrap().clone()
                } else {
                    "-".to_string()
                }
            }
            None => "-".to_string(),
        };

        let ap_frequency = match self.device.access_point.as_ref() {
            Some(ap) => {
                if ap.has_started {
                    format!("{:.2} GHz", (ap.frequency.unwrap() / 1000))
                } else {
                    "-".to_string()
                }
            }
            None => "-".to_string(),
        };

        let ap_used_cipher = match self.device.access_point.as_ref() {
            Some(ap) => {
                if ap.has_started {
                    ap.used_cipher.as_ref().unwrap().clone()
                } else {
                    "-".to_string()
                }
            }
            None => "-".to_string(),
        };

        let ap_is_scanning = match self.device.access_point.as_ref() {
            Some(ap) => {
                if ap.has_started {
                    match ap.is_scanning {
                        Some(v) => v.to_string(),
                        None => "-".to_string(),
                    }
                } else {
                    "-".to_string()
                }
            }

            None => "-".to_string(),
        };

        let row = Row::new(vec![
            Line::from(
                self.device
                    .access_point
                    .as_ref()
                    .unwrap()
                    .has_started
                    .clone()
                    .to_string(),
            )
            .centered(),
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
                        Line::from("Started")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("SSID")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Frequency")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Cipher")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Scanning")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                    ])
                    .style(Style::new().bold())
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
            .flex(Flex::SpaceBetween)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(if focused_block == FocusedBlock::AccessPoint {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut access_point_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(
            access_point_table,
            access_point_block,
            &mut access_point_state,
        );

        // Connected devices
        if any_connected_devices {
            let devices = self
                .device
                .access_point
                .as_ref()
                .unwrap()
                .connected_devices
                .clone();

            let connected_devices_list = List::new(devices)
                .block(
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
                )
                .style(match color_mode {
                    ColorMode::Dark => Style::default().fg(Color::White),
                    ColorMode::Light => Style::default().fg(Color::Black),
                });

            frame.render_widget(connected_devices_list, connected_devices_block);
        }

        let help_message = match focused_block {
            FocusedBlock::Device => Line::from(vec![
                Span::from(self.config.device.infos.to_string()).bold(),
                Span::from(" Infos"),
                Span::from(" | "),
                Span::from(self.config.device.toggle_power.to_string()).bold(),
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
                Span::from(self.config.ap.start.to_string()).bold(),
                Span::from(" New AP"),
                Span::from(" | "),
                Span::from(self.config.ap.stop.to_string()).bold(),
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

    pub fn render_station_mode(
        &self,
        frame: &mut Frame,
        color_mode: ColorMode,
        focused_block: FocusedBlock,
    ) {
        let (known_networks_block, new_networks_block, device_block, help_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(5),
                    Constraint::Min(5),
                    Constraint::Length(5),
                    Constraint::Length(1),
                ])
                .margin(1)
                .split(frame.area());
            (chunks[0], chunks[1], chunks[2], chunks[3])
        };

        // Device

        let station_frequency = {
            match self.device.station.as_ref() {
                Some(station) => {
                    if station.state == "connected" {
                        match station.diagnostic.get("Frequency") {
                            Some(f) => {
                                let f: f32 = f.parse().unwrap();
                                format!("{:.2} GHz", f / 1000.0)
                            }
                            None => String::from("-"),
                        }
                    } else {
                        String::from("-")
                    }
                }
                None => String::from("-"),
            }
        };

        let station_security = {
            match self.device.station.as_ref() {
                Some(station) => {
                    if station.state == "connected" {
                        match station.diagnostic.get("Security") {
                            Some(f) => f.trim_matches('"').to_string(),
                            None => String::from("-"),
                        }
                    } else {
                        String::from("-")
                    }
                }
                None => String::from("-"),
            }
        };

        let mut station_state = "".to_string();
        let mut station_is_scanning = "".to_string();
        if let Some(station) = self.device.station.as_ref() {
            station_state = station.state.clone();
            station_is_scanning = station.is_scanning.clone().to_string();
        }

        let row = Row::new(vec![
            Line::from(self.device.name.clone()).centered(),
            Line::from("station").centered(),
            {
                if self.device.is_powered {
                    Line::from("On").centered()
                } else {
                    Line::from("Off").centered()
                }
            },
            Line::from(station_state).centered(),
            Line::from(station_is_scanning).centered(),
            Line::from(station_frequency).centered(),
            Line::from(station_security).centered(),
        ]);

        let widths = [
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(15),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Line::from("Name").yellow().centered(),
                        Line::from("Mode").yellow().centered(),
                        Line::from("Powered").yellow().centered(),
                        Line::from("State").yellow().centered(),
                        Line::from("Scanning").yellow().centered(),
                        Line::from("Frequency").yellow().centered(),
                        Line::from("Security").yellow().centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Line::from("Name")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Mode")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Powered")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("State")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Scanning")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Frequency")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Security")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                    ])
                    .style(Style::new().bold())
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
                    })
                    .padding(Padding::horizontal(1)),
            )
            .column_spacing(1)
            .flex(Flex::SpaceBetween)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        // Known networks
        let known_networks = if let Some(station) = self.device.station.as_ref() {
            &station.known_networks
        } else {
            &vec![]
        };
        let rows: Vec<Row> = known_networks
            .iter()
            .map(|(net, signal)| {
                let net = net.known_network.as_ref().unwrap();
                let signal = format!("{}%", {
                    if *signal / 100 >= -50 {
                        100
                    } else {
                        2 * (100 + signal / 100)
                    }
                });

                if let Some(connected_net) =
                    &self.device.station.as_ref().unwrap().connected_network
                {
                    if connected_net.name == net.name {
                        let row = vec![
                            Line::from("󰸞").centered(),
                            Line::from(net.name.clone()).centered(),
                            Line::from(net.network_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()).centered(),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                            Line::from(signal).centered(),
                        ];

                        Row::new(row)
                    } else {
                        let row = vec![
                            Line::from(""),
                            Line::from(net.name.clone()).centered(),
                            Line::from(net.network_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()).centered(),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                            Line::from(signal).centered(),
                        ];

                        Row::new(row)
                    }
                } else {
                    let row = vec![
                        Line::from("").centered(),
                        Line::from(net.name.clone()).centered(),
                        Line::from(net.network_type.clone()).centered(),
                        Line::from(net.is_hidden.to_string()).centered(),
                        Line::from(net.is_autoconnect.to_string()).centered(),
                        Line::from(signal).centered(),
                    ];

                    Row::new(row)
                }
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(25),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(6),
        ];

        let known_networks_table = Table::new(rows, widths)
            .header({
                if focused_block == FocusedBlock::KnownNetworks {
                    Row::new(vec![
                        Line::from(""),
                        Line::from("Name").yellow().centered(),
                        Line::from("Security").yellow().centered(),
                        Line::from("Hidden").yellow().centered(),
                        Line::from("Auto Connect").yellow().centered(),
                        Line::from("Signal").yellow().centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Line::from(""),
                        Line::from("Name")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Security")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Hidden")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Auto Connect")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Signal")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(" Known Networks ")
                    .title_style({
                        if focused_block == FocusedBlock::KnownNetworks {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if focused_block == FocusedBlock::KnownNetworks {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::KnownNetworks {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    })
                    .padding(Padding::horizontal(1)),
            )
            .column_spacing(2)
            .flex(Flex::SpaceBetween)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(if focused_block == FocusedBlock::KnownNetworks {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut known_networks_state = if let Some(station) = self.device.station.as_ref() {
            station.known_networks_state.clone()
        } else {
            TableState::default()
        };
        frame.render_stateful_widget(
            known_networks_table,
            known_networks_block,
            &mut known_networks_state,
        );

        // New networks
        let new_networks = if let Some(station) = self.device.station.as_ref() {
            &station.new_networks
        } else {
            &vec![]
        };
        let rows: Vec<Row> = new_networks
            .iter()
            .map(|(net, signal)| {
                Row::new(vec![
                    Line::from(net.name.clone()).centered(),
                    Line::from(net.network_type.clone()).centered(),
                    Line::from({
                        let signal = {
                            if *signal / 100 >= -50 {
                                100
                            } else {
                                2 * (100 + signal / 100)
                            }
                        };
                        match signal {
                            n if n >= 75 => format!("{signal:3}% 󰤨"),
                            n if (50..75).contains(&n) => format!("{signal:3}% 󰤥"),
                            n if (25..50).contains(&n) => format!("{signal:3}% 󰤢"),
                            _ => format!("{signal:3}% 󰤟"),
                        }
                    })
                    .centered(),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(25),
            Constraint::Length(15),
            Constraint::Length(8),
        ];

        let new_networks_table = Table::new(rows, widths)
            .header({
                if focused_block == FocusedBlock::NewNetworks {
                    Row::new(vec![
                        Line::from("Name").yellow().centered(),
                        Line::from("Security").yellow().centered(),
                        Line::from("Signal").yellow().centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Line::from("Name")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Security")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                        Line::from("Signal")
                            .style(match color_mode {
                                ColorMode::Dark => Style::default().fg(Color::White),
                                ColorMode::Light => Style::default().fg(Color::Black),
                            })
                            .centered(),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(" New Networks ")
                    .title_style({
                        if focused_block == FocusedBlock::NewNetworks {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if focused_block == FocusedBlock::NewNetworks {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::NewNetworks {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    })
                    .padding(Padding::horizontal(1)),
            )
            .column_spacing(2)
            .flex(Flex::SpaceBetween)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(if focused_block == FocusedBlock::NewNetworks {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut new_networks_state = if let Some(station) = self.device.station.as_ref() {
            station.new_networks_state.clone()
        } else {
            TableState::default()
        };
        frame.render_stateful_widget(
            new_networks_table,
            new_networks_block,
            &mut new_networks_state,
        );

        let help_message = match focused_block {
            FocusedBlock::Device => Line::from(vec![
                Span::from(self.config.station.start_scanning.to_string()).bold(),
                Span::from(" Scan"),
                Span::from(" | "),
                Span::from(self.config.device.infos.to_string()).bold(),
                Span::from(" Infos"),
                Span::from(" | "),
                Span::from(self.config.device.toggle_power.to_string()).bold(),
                Span::from(" Toggle Power"),
                Span::from(" | "),
                Span::from("ctrl+r").bold(),
                Span::from(" Switch Mode"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            FocusedBlock::KnownNetworks => Line::from(vec![
                Span::from("k,").bold(),
                Span::from("  Up"),
                Span::from(" | "),
                Span::from("j,").bold(),
                Span::from("  Down"),
                Span::from(" | "),
                Span::from(if self.config.station.toggle_connect == ' ' {
                    "󱁐 ".to_string()
                } else {
                    self.config.station.toggle_connect.to_string()
                })
                .bold(),
                Span::from(" Dis/connect"),
                Span::from(" | "),
                Span::from(self.config.station.known_network.remove.to_string()).bold(),
                Span::from(" Remove"),
                Span::from(" | "),
                Span::from(
                    self.config
                        .station
                        .known_network
                        .toggle_autoconnect
                        .to_string(),
                )
                .bold(),
                Span::from(" Autoconnect"),
                Span::from(" | "),
                Span::from(self.config.station.start_scanning.to_string()).bold(),
                Span::from(" Scan"),
                Span::from(" | "),
                Span::from("󱊷 ").bold(),
                Span::from(" Discard"),
                Span::from(" | "),
                Span::from("ctrl+r").bold(),
                Span::from(" Switch Mode"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            FocusedBlock::NewNetworks => Line::from(vec![
                Span::from("k,").bold(),
                Span::from("  Up"),
                Span::from(" | "),
                Span::from("j,").bold(),
                Span::from("  Down"),
                Span::from(" | "),
                Span::from("󱁐 ").bold(),
                Span::from(" Connect"),
                Span::from(" | "),
                Span::from(self.config.station.start_scanning.to_string()).bold(),
                Span::from(" Scan"),
                Span::from(" | "),
                Span::from("󱊷 ").bold(),
                Span::from(" Discard"),
                Span::from(" | "),
                Span::from("ctrl+r").bold(),
                Span::from(" Switch Mode"),
                Span::from(" | "),
                Span::from("⇄").bold(),
                Span::from(" Nav"),
            ]),
            FocusedBlock::AdapterInfos | FocusedBlock::AuthKey => {
                Line::from(vec![Span::from("󱊷 ").bold(), Span::from(" Discard")])
            }
            _ => Line::from(""),
        };

        let help_message = help_message.centered().blue();

        frame.render_widget(help_message, help_block);
    }

    pub fn render_adapter(&self, frame: &mut Frame, color_mode: ColorMode) {
        let popup_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(9),
                Constraint::Fill(1),
            ])
            .flex(Flex::Start)
            .split(frame.area());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Min(80),
                Constraint::Fill(1),
            ])
            .split(popup_layout[1])[1];

        let mut rows = vec![
            Row::new(vec![
                Cell::from("name").style(Style::default().bold().yellow()),
                Cell::from(self.name.clone()),
            ]),
            Row::new(vec![
                Cell::from("address").style(Style::default().bold().yellow()),
                Cell::from(self.device.address.clone()),
            ]),
            Row::new(vec![
                Cell::from("Supported modes").style(Style::default().bold().yellow()),
                Cell::from(self.supported_modes.clone().join(" ")),
            ]),
        ];

        if let Some(model) = &self.model {
            rows.push(Row::new(vec![
                Cell::from("model").style(Style::default().bold().yellow()),
                Cell::from(model.clone()),
            ]));
        }

        if let Some(vendor) = &self.vendor {
            rows.push(Row::new(vec![
                Cell::from("vendor").style(Style::default().bold().yellow()),
                Cell::from(vendor.clone()),
            ]));
        }

        let widths = [Constraint::Length(20), Constraint::Fill(1)];

        let device_infos_table = Table::new(rows, widths)
            .block(
                Block::default()
                    .title(" Adapter Infos ")
                    .title_style(Style::default().bold())
                    .title_alignment(Alignment::Center)
                    .padding(Padding::uniform(1))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .border_type(BorderType::Thick),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

        frame.render_widget(Clear, area);
        frame.render_widget(device_infos_table, area);
    }
}
