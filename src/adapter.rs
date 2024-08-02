use std::sync::Arc;

use anyhow::{Context, Result};

use iwdrs::{adapter::Adapter as iwdAdapter, session::Session};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, Padding, Paragraph, Row, Table, TableState,
        Tabs,
    },
    Frame,
};

use crate::{
    app::FocusedBlock,
    config::Config,
    device::Device,
    tui::Palette,
};

#[derive(Debug, Clone)]
pub struct Adapter {
    pub config: Arc<Config>,
    pub adapter: iwdAdapter,
    pub is_powered: bool,
    pub name: String,
    pub model: Option<String>,
    pub vendor: Option<String>,
    pub supported_modes: Vec<String>,
    pub device: Device,
}

impl Adapter {
    pub async fn new(config: Arc<Config>, session: Arc<Session>) -> Result<Self> {
        let adapter = session.adapter().context("No adapter found")?;

        let is_powered = adapter.is_powered().await?;
        let name = adapter.name().await?;
        let model = adapter.model().await.ok();
        let vendor = adapter.vendor().await.ok();
        let supported_modes = adapter.supported_modes().await?;
        let device = Device::new(session.clone()).await?;

        Ok(Self {
            config: config.clone(),
            adapter,
            is_powered,
            name,
            model,
            vendor,
            supported_modes,
            device,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.adapter.is_powered().await?;
        self.device.refresh().await?;
        Ok(())
    }

    pub fn render(&self, palette: &Palette, frame: &mut Frame, focused_block: FocusedBlock) {
        match self.device.mode.as_str() {
            "station" => {
                if self.device.station.is_some() {
                    self.render_station_mode(palette, frame, focused_block);
                } else {
                    self.render_other_mode(palette, frame, focused_block);
                }
            }
            "ap" => {
                if self.device.access_point.is_some() {
                    self.render_access_point_mode(palette, frame, focused_block);
                } else {
                    self.render_other_mode(palette, frame, focused_block);
                }
            }
            _ => self.render_other_mode(palette, frame, focused_block),
        }
    }

    pub fn render_other_mode(&self, palette: &Palette, frame: &mut Frame,
        focused_block: FocusedBlock) {
        let device_block = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Fill(1)])
                .margin(1)
                .split(frame.size());
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(palette.active_table_header),
                        Cell::from("Mode").style(palette.active_table_header),
                        Cell::from("Powered").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(palette.inactive_table_header),
                        Cell::from("Mode").style(palette.inactive_table_header),
                        Cell::from("Powered").style(palette.inactive_table_header),
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
                            palette.active_border
                        } else {
                            palette.inactive_border
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
            .column_spacing(if narrow_mode { 1 } else { 3 })
            .style(palette.text)
            .highlight_style(if focused_block == FocusedBlock::Device {
                palette.active_table_row
            } else {
                palette.text
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);
    }

    pub fn render_access_point_mode(&self, palette: &Palette, frame: &mut Frame,
        focused_block: FocusedBlock) {
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
                .split(frame.size());
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(8),
            Constraint::Length(12),
            Constraint::Length(9),
            Constraint::Fill(1),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(palette.active_table_header),
                        Cell::from("Mode").style(palette.active_table_header),
                        Cell::from("Powered").style(palette.active_table_header),
                        Cell::from("Address").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(palette.inactive_table_header),
                        Cell::from("Mode").style(palette.inactive_table_header),
                        Cell::from("Powered").style(palette.inactive_table_header),
                        Cell::from("Address").style(palette.inactive_table_header),
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
                            palette.active_border
                        } else {
                            palette.inactive_border
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
            .column_spacing(if narrow_mode { 1 } else { 3 })
            .style(palette.text)
            .highlight_style(if focused_block == FocusedBlock::Device {
                palette.active_table_row
            } else {
                palette.text
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

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
                        Cell::from("Started").style(palette.active_table_header),
                        Cell::from("SSID").style(palette.active_table_header),
                        Cell::from("Frequency").style(palette.active_table_header),
                        Cell::from("Cipher").style(palette.active_table_header),
                        Cell::from("Scanning").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Started").style(palette.inactive_table_header),
                        Cell::from("SSID").style(palette.inactive_table_header),
                        Cell::from("Frequency").style(palette.inactive_table_header),
                        Cell::from("Cipher").style(palette.inactive_table_header),
                        Cell::from("Scanning").style(palette.inactive_table_header),
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
                            palette.active_border
                        } else {
                            palette.inactive_border
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
            .column_spacing(if narrow_mode { 1 } else { 3 })
            .style(palette.text)
            .highlight_style(if focused_block == FocusedBlock::AccessPoint {
                palette.active_table_row
            } else {
                palette.text
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
                                palette.active_border
                            } else {
                                palette.inactive_border
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
                .style(palette.text);

            frame.render_widget(connected_devices_list, connected_devices_block);
        }
    }

    pub fn render_device_table(
        &self,
        palette: &Palette,
        frame: &mut Frame,
        device_block: Rect,
        render_title: bool,
        is_focused: bool,
    ) {
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if is_focused {
                    Row::new(vec![
                        Cell::from("Name").style(palette.active_table_header),
                        Cell::from("Mode").style(palette.active_table_header),
                        Cell::from("Powered").style(palette.active_table_header),
                        Cell::from("Address").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(palette.inactive_table_header),
                        Cell::from("Mode").style(palette.inactive_table_header),
                        Cell::from("Powered").style(palette.inactive_table_header),
                        Cell::from("Address").style(palette.inactive_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(if render_title { " Device " } else { "" })
                    .title_style({
                        if is_focused {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if is_focused {
                            palette.active_border
                        } else {
                            palette.inactive_border
                        }
                    })
                    .border_type({
                        if is_focused {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(if narrow_mode { 1 } else { 3 })
            .style(palette.text)
            .highlight_style(if is_focused {
                palette.active_table_row
            } else {
                palette.text
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);
    }

    pub fn render_station_table(
        &self,
        palette: &Palette,
        frame: &mut Frame,
        station_block: Rect,
        render_title: bool,
        is_focused: bool,
    ) {
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(12),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ];

        let station_table = Table::new(vec![row], widths)
            .header({
                if is_focused {
                    Row::new(vec![
                        Cell::from("State").style(palette.active_table_header),
                        Cell::from("Scanning").style(palette.active_table_header),
                        Cell::from("Frequency").style(palette.active_table_header),
                        Cell::from("Security").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("State").style(palette.inactive_table_header),
                        Cell::from("Scanning").style(palette.inactive_table_header),
                        Cell::from("Frequency").style(palette.inactive_table_header),
                        Cell::from("Security").style(palette.inactive_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(if render_title { " Station " } else { "" })
                    .title_style({
                        if is_focused {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if is_focused {
                            palette.active_border
                        } else {
                            palette.inactive_border
                        }
                    })
                    .border_type({
                        if is_focused {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(if narrow_mode { 1 } else { 3 })
            .style(palette.text)
            .highlight_style(if is_focused {
                palette.active_table_row
            } else {
                palette.text
            });

        let mut station_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(station_table, station_block, &mut station_state);
    }

    pub fn render_known_networks_table(
        &self,
        palette: &Palette,
        frame: &mut Frame,
        known_networks_block: Rect,
        render_title: bool,
        is_focused: bool,
    ) {
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
                            Line::from(if self.config.unicode { "󰸞" } else { "*" }),
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

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(1),
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(if narrow_mode { 7 } else { 12 }),
            Constraint::Length(6),
        ];

        let known_networks_table = Table::new(rows, widths)
            .header({
                if is_focused {
                    Row::new(vec![
                        Cell::from(""),
                        Cell::from("Name").style(palette.active_table_header),
                        Cell::from("Security").style(palette.active_table_header),
                        Cell::from("Hidden").style(palette.active_table_header),
                        Cell::from(if narrow_mode {
                            "AutoCon"
                        } else {
                            "Auto Connect"
                        })
                        .style(palette.active_table_header),
                        Cell::from("Signal").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from(""),
                        Cell::from("Name").style(palette.inactive_table_header),
                        Cell::from("Security").style(palette.inactive_table_header),
                        Cell::from("Hidden").style(palette.inactive_table_header),
                        Cell::from("Auto Connect").style(palette.inactive_table_header),
                        Cell::from("Signal").style(palette.inactive_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(if render_title { " Known networks " } else { "" })
                    .title_style({
                        if is_focused {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if is_focused {
                            palette.active_border
                        } else {
                            palette.inactive_border
                        }
                    })
                    .border_type({
                        if is_focused {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(if narrow_mode { 1 } else { 4 })
            .style(palette.text)
            .highlight_style(if is_focused {
                palette.active_table_row
            } else {
                palette.text
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
    }

    pub fn render_new_networks_table(
        &self,
        palette: &Palette,
        frame: &mut Frame,
        new_networks_block: Rect,
        render_title: bool,
        is_focused: bool,
    ) {
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

                        let signal_level = match signal {
                            n if n >= 75 => {
                                if self.config.unicode {
                                    "󰤨"
                                } else {
                                    "(****))"
                                }
                            }
                            n if (50..75).contains(&n) => {
                                if self.config.unicode {
                                    "󰤥"
                                } else {
                                    "(*** )"
                                }
                            }
                            n if (25..50).contains(&n) => {
                                if self.config.unicode {
                                    "󰤢"
                                } else {
                                    "(**  )"
                                }
                            }
                            _ => {
                                if self.config.unicode {
                                    "󰤟"
                                } else {
                                    "(*   )"
                                }
                            }
                        };

                        format!("{:3}% {}", signal, signal_level)
                    }),
                ])
            })
            .collect();

        let narrow_mode = frame.size().width < self.config.small_layout_width;

        let widths = [
            Constraint::Length(15),
            Constraint::Length(8),
            Constraint::Length(if self.config.unicode { 6 } else { 11 }),
        ];

        let new_networks_table = Table::new(rows, widths)
            .header({
                if is_focused {
                    Row::new(vec![
                        Cell::from("Name").style(palette.active_table_header),
                        Cell::from("Security").style(palette.active_table_header),
                        Cell::from("Signal").style(palette.active_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(palette.inactive_table_header),
                        Cell::from("Security").style(palette.inactive_table_header),
                        Cell::from("Signal").style(palette.inactive_table_header),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                }
            })
            .block(
                Block::default()
                    .title(if render_title { " New networks " } else { "" })
                    .title_style({
                        if is_focused {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if is_focused {
                            palette.active_border
                        } else {
                            palette.inactive_border
                        }
                    })
                    .border_type({
                        if is_focused {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(if narrow_mode { 1 } else { 4 })
            .style(palette.text)
            .highlight_style(if is_focused {
                palette.active_table_row
            } else {
                palette.text
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

    pub fn render_status_bar(&self, palette: &Palette, frame: &mut Frame, status_bar_block: Rect) {
        let status_bar_content = Paragraph::new("Q: quit Arrows/HJKL: move Space: connect ?: help")
            .style(palette.status_bar);

        frame.render_widget(status_bar_content, status_bar_block);
    }

    pub fn render_station_mode(&self, palette: &Palette, frame: &mut Frame,
        focused_block: FocusedBlock) {
        if frame.size().height > self.config.small_layout_height {
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
                    .split(frame.size());
                (chunks[0], chunks[1], chunks[2], chunks[3])
            };

            // Device
            self.render_device_table(
                palette,
                frame,
                device_block,
                true, /* render title */
                focused_block == FocusedBlock::Device,
            );

            // Station
            self.render_station_table(
                palette,
                frame,
                station_block,
                true, /* render title */
                focused_block == FocusedBlock::Station,
            );

            // Known networks
            self.render_known_networks_table(
                palette,
                frame,
                known_networks_block,
                true, /* render title */
                focused_block == FocusedBlock::KnownNetworks,
            );

            // New networks
            self.render_new_networks_table(
                palette,
                frame,
                new_networks_block,
                true, /* render title */
                focused_block == FocusedBlock::NewNetworks,
            );

        // Render compact tabs view
        } else {
            let (tab_bar_block, content_block, status_bar_block) = {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Length(1),
                            Constraint::Min(0),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(frame.size());
                (chunks[0], chunks[1], chunks[2])
            };

            // Differentiate selected tab with ASCII
            fn format_tab_title(
                unicode: bool,
                focused_block: FocusedBlock,
                title_block: FocusedBlock,
                title: &str,
            ) -> String {
                if !unicode && (focused_block == title_block) {
                    format!("[{}]", title.to_uppercase())
                } else {
                    format!(" {} ", title)
                }
            }

            let device_tab_title = format_tab_title(
                self.config.unicode,
                focused_block,
                FocusedBlock::Device,
                "Device",
            );
            let station_tab_title = format_tab_title(
                self.config.unicode,
                focused_block,
                FocusedBlock::Station,
                "Station",
            );
            let known_networks_tab_title = format_tab_title(
                self.config.unicode,
                focused_block,
                FocusedBlock::KnownNetworks,
                if frame.size().width >= 56 {
                    "Known networks"
                } else {
                    "Known nets"
                },
            );
            let new_networks_tab_title = format_tab_title(
                self.config.unicode,
                focused_block,
                FocusedBlock::NewNetworks,
                if frame.size().width >= 56 {
                    "New networks"
                } else {
                    "New nets"
                },
            );

            let tabs = Tabs::new(vec![
                device_tab_title,
                station_tab_title,
                known_networks_tab_title,
                new_networks_tab_title,
            ])
            .style(palette.text);

            if focused_block == FocusedBlock::Device {
                frame.render_widget(tabs.select(0), tab_bar_block);
                self.render_device_table(
                    palette,
                    frame,
                    content_block,
                    false, /* don't render title */
                    true,  /* device block focused */
                );
            } else if focused_block == FocusedBlock::Station {
                frame.render_widget(tabs.select(1), tab_bar_block);
                self.render_station_table(
                    palette,
                    frame,
                    content_block,
                    false, /* don't render title */
                    true,  /* station block focused */
                );
            } else if focused_block == FocusedBlock::KnownNetworks {
                frame.render_widget(tabs.select(2), tab_bar_block);
                self.render_known_networks_table(
                    palette,
                    frame,
                    content_block,
                    false, /* don't render title */
                    true,  /* known networks block focused */
                );
            } else if focused_block == FocusedBlock::NewNetworks {
                frame.render_widget(tabs.select(3), tab_bar_block);
                self.render_new_networks_table(
                    palette,
                    frame,
                    content_block,
                    false, /* don't render title */
                    true,  /* new networks block focused */
                );
            }

            self.render_status_bar(palette, frame, status_bar_block);
        }
    }

    pub fn render_adapter(&self, palette: &Palette, frame: &mut Frame) {
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

        let mut rows = vec![
            Row::new(vec![
                Cell::from("name").style(palette.active_table_header.bold()),
                Cell::from(self.name.clone()),
            ]),
            Row::new(vec![
                Cell::from("address").style(palette.active_table_header.bold()),
                Cell::from(self.device.address.clone()),
            ]),
            Row::new(vec![
                Cell::from("Supported modes").style(palette.active_table_header.bold()),
                Cell::from(self.supported_modes.clone().join(" ")),
            ]),
        ];

        if let Some(model) = &self.model {
            rows.push(Row::new(vec![
                Cell::from("model").style(palette.active_table_header.bold()),
                Cell::from(model.clone()),
            ]))
        }

        if let Some(vendor) = &self.vendor {
            rows.push(Row::new(vec![
                Cell::from("vendor").style(palette.active_table_header.bold()),
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
                    .border_style(palette.active_border)
                    .border_type(BorderType::Thick),
            )
            .column_spacing(3)
            .style(palette.text)
            .highlight_style(palette.active_table_row);

        frame.render_widget(Clear, area);
        frame.render_widget(device_infos_table, area);
    }
}
