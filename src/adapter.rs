use std::sync::Arc;

use anyhow::Context;

use iwdrs::{adapter::Adapter as iwdAdapter, session::Session};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Cell, Clear, List, Padding, Row, Table, TableState},
    Frame,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{AppResult, ColorMode, FocusedBlock},
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
}

impl Adapter {
    pub async fn new(session: Arc<Session>, sender: UnboundedSender<Event>) -> AppResult<Self> {
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
        })
    }

    pub async fn refresh(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.is_powered = self.adapter.is_powered().await?;
        self.device.refresh(sender).await?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame, color_mode: ColorMode, focused_block: FocusedBlock) {
        match self.device.mode.as_str() {
            "station" => {
                if self.device.station.is_some() {
                    self.render_station_mode(frame, color_mode, focused_block);
                } else {
                    self.render_other_mode(frame, color_mode, focused_block);
                }
            }
            "ap" => {
                if self.device.access_point.is_some() {
                    self.render_access_point_mode(frame, color_mode, focused_block);
                } else {
                    self.render_other_mode(frame, color_mode, focused_block);
                }
            }
            _ => self.render_other_mode(frame, color_mode, focused_block),
        }
    }

    pub fn render_other_mode(
        &self,
        frame: &mut Frame,
        color_mode: ColorMode,
        focused_block: FocusedBlock,
    ) {
        let device_block = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1)])
                .margin(1)
                .split(frame.area());
            chunks[0]
        };

        // Device
        let row = Row::new(vec![self.device.name.clone(), self.device.mode.clone(), {
            if self.device.is_powered {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        }]);

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Mode").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Powered").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Mode").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Powered").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);
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

        let (device_block, access_point_block, connected_devices_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(if any_connected_devices {
                    &[
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                        Constraint::Percentage(33),
                    ]
                } else {
                    &[
                        Constraint::Percentage(50),
                        Constraint::Percentage(50),
                        Constraint::Fill(1),
                    ]
                })
                .margin(1)
                .split(frame.area());
            (chunks[0], chunks[1], chunks[2])
        };

        // Device
        let row = Row::new(vec![
            self.device.name.clone(),
            "Access Point".to_string(),
            {
                if self.device.is_powered {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }
            },
            self.device.address.clone(),
        ]);

        let widths = [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Mode").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Powered").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Address").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Mode").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Powered").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Address").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray)
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
                    format!("{:.2} GHz", (ap.frequency.unwrap() as f32 / 1000.0))
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
            self.device
                .access_point
                .as_ref()
                .unwrap()
                .has_started
                .clone()
                .to_string(),
            ap_name,
            ap_frequency,
            ap_used_cipher,
            ap_is_scanning,
        ]);

        let widths = [
            Constraint::Length(10),
            Constraint::Length(15),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let access_point_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::AccessPoint {
                    Row::new(vec![
                        Cell::from("Started").style(Style::default().fg(Color::Yellow)),
                        Cell::from("SSID").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Frequency").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Cipher").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Scanning").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Started").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("SSID").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Frequency").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Cipher").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Scanning").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::AccessPoint {
                Style::default().bg(Color::DarkGray)
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
                        }),
                )
                .style(match color_mode {
                    ColorMode::Dark => Style::default().fg(Color::White),
                    ColorMode::Light => Style::default().fg(Color::Black),
                });

            frame.render_widget(connected_devices_list, connected_devices_block);
        }
    }

    pub fn render_station_mode(
        &self,
        frame: &mut Frame,
        color_mode: ColorMode,
        focused_block: FocusedBlock,
    ) {
        let (device_block, station_block, known_networks_block, new_networks_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(16),
                    Constraint::Percentage(16),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .margin(1)
                .split(frame.area());
            (chunks[0], chunks[1], chunks[2], chunks[3])
        };

        // Device
        let row = Row::new(vec![
            self.device.name.clone(),
            "station".to_string(),
            {
                if self.device.is_powered {
                    "On".to_string()
                } else {
                    "Off".to_string()
                }
            },
            self.device.address.clone(),
        ]);

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Mode").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Powered").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Address").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Mode").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Powered").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Address").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        // Station

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

        let row = vec![
            self.device
                .station
                .as_ref()
                .unwrap()
                .state
                .clone()
                .to_string(),
            self.device
                .station
                .as_ref()
                .unwrap()
                .is_scanning
                .clone()
                .to_string(),
            station_frequency,
            station_security,
        ];

        let row = Row::new(row);

        let widths = [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let station_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Station {
                    Row::new(vec![
                        Cell::from("State").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Scanning").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Frequency").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Security").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("State").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Scanning").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Frequency").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Security").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(" Station ")
                    .title_style({
                        if focused_block == FocusedBlock::Station {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if focused_block == FocusedBlock::Station {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if focused_block == FocusedBlock::Station {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(3)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::Station {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        let mut station_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(station_table, station_block, &mut station_state);

        // Known networks

        let rows: Vec<Row> = self
            .device
            .station
            .as_ref()
            .unwrap()
            .known_networks
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
                            Line::from("󰸞"),
                            Line::from(net.name.clone()),
                            Line::from(net.netowrk_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                            Line::from(signal).centered(),
                        ];

                        Row::new(row)
                    } else {
                        let row = vec![
                            Line::from(""),
                            Line::from(net.name.clone()),
                            Line::from(net.netowrk_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                            Line::from(signal).centered(),
                        ];

                        Row::new(row)
                    }
                } else {
                    let row = vec![
                        Line::from(""),
                        Line::from(net.name.clone()),
                        Line::from(net.netowrk_type.clone()).centered(),
                        Line::from(net.is_hidden.to_string()),
                        Line::from(net.is_autoconnect.to_string()),
                        Line::from(signal).centered(),
                    ];

                    Row::new(row)
                }
            })
            .collect();

        let widths = [
            Constraint::Length(2),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(6),
        ];

        let known_networks_table = Table::new(rows, widths)
            .header({
                if focused_block == FocusedBlock::KnownNetworks {
                    Row::new(vec![
                        Cell::from(""),
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Security").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Hidden").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Auto Connect").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Signal").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from(""),
                        Cell::from("Name").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Security").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Hidden").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Auto Connect").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Signal").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(4)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::KnownNetworks {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            known_networks_table,
            known_networks_block,
            &mut self
                .device
                .station
                .as_ref()
                .unwrap()
                .known_networks_state
                .clone(),
        );

        // New networks

        let rows: Vec<Row> = self
            .device
            .station
            .as_ref()
            .unwrap()
            .new_networks
            .iter()
            .map(|(net, signal)| {
                Row::new(vec![
                    Line::from(net.name.clone()),
                    Line::from(net.netowrk_type.clone()).centered(),
                    Line::from({
                        let signal = {
                            if *signal / 100 >= -50 {
                                100
                            } else {
                                2 * (100 + signal / 100)
                            }
                        };
                        match signal {
                            n if n >= 75 => format!("{:3}% 󰤨", signal),
                            n if (50..75).contains(&n) => format!("{:3}% 󰤥", signal),
                            n if (25..50).contains(&n) => format!("{:3}% 󰤢", signal),
                            _ => format!("{:3}% 󰤟", signal),
                        }
                    }),
                ])
            })
            .collect();

        let widths = [
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(6),
        ];

        let new_networks_table = Table::new(rows, widths)
            .header({
                if focused_block == FocusedBlock::NewNetworks {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Security").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Signal").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Security").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Signal").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
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
                    }),
            )
            .column_spacing(4)
            .style(match color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if focused_block == FocusedBlock::NewNetworks {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            new_networks_table,
            new_networks_block,
            &mut self
                .device
                .station
                .as_ref()
                .unwrap()
                .new_networks_state
                .clone(),
        );
    }

    pub fn render_adapter(&self, frame: &mut Frame, color_mode: ColorMode) {
        let popup_layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(10),
                    Constraint::Length(9),
                    Constraint::Fill(1),
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
            ]))
        }

        if let Some(vendor) = &self.vendor {
            rows.push(Row::new(vec![
                Cell::from("vendor").style(Style::default().bold().yellow()),
                Cell::from(vendor.clone()),
            ]))
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
            .highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(Clear, area);
        frame.render_widget(device_infos_table, area);
    }
}
