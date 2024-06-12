use std::sync::Arc;

use anyhow::{Context, Result};

use iwdrs::{adapter::Adapter as iwdAdapter, session::Session};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, Borders, Cell, Clear, Padding, Row, Table, TableState},
    Frame,
};

use crate::{
    app::{ColorMode, FocusedBlock},
    device::Device,
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
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let adapter = session.adapter().context("No adapter found")?;

        let is_powered = adapter.is_powered().await?;
        let name = adapter.name().await?;
        let model = adapter.model().await.ok();
        let vendor = adapter.vendor().await.ok();
        let supported_modes = adapter.supported_modes().await?;
        let device = Device::new(session.clone()).await?;

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

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.adapter.is_powered().await?;
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
                .split(frame.size());
            chunks[0]
        };

        // Device
        let row = Row::new(vec![
            self.device.name.clone(),
            self.device.mode.clone(),
            self.device.is_powered.clone().to_string(),
        ]);

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

    pub fn render_station_mode(
        &self,
        frame: &mut Frame,
        color_mode: ColorMode,
        focused_block: FocusedBlock,
    ) {
        let (device_block, known_networks_block, new_networks_block) = {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .margin(1)
                .split(frame.size());
            (chunks[0], chunks[1], chunks[2])
        };

        // Device
        let row = match self.device.mode.as_str() {
            "station" => Row::new(vec![
                self.device.name.clone(),
                "station".to_string(),
                self.device.is_powered.clone().to_string(),
                self.device
                    .station
                    .as_ref()
                    .unwrap()
                    .is_scanning
                    .clone()
                    .to_string(),
                self.device
                    .station
                    .as_ref()
                    .unwrap()
                    .state
                    .clone()
                    .to_string(),
            ]),
            _ => Row::new(vec![""]),
        };

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if focused_block == FocusedBlock::Device {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Mode").style(Style::default().fg(Color::Yellow)),
                        Cell::from("State").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Scanning").style(Style::default().fg(Color::Yellow)),
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
                        Cell::from("State").style(match color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Scanning").style(match color_mode {
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

        // Known networks

        let rows: Vec<Row> = self
            .device
            .station
            .as_ref()
            .unwrap()
            .known_networks
            .iter()
            .map(|(net, _signal)| {
                let net = net.known_network.as_ref().unwrap();

                if let Some(connected_net) =
                    &self.device.station.as_ref().unwrap().connected_network
                {
                    if connected_net.name == net.name {
                        let mut row = vec![
                            Line::from("󰸞"),
                            Line::from(net.name.clone()),
                            Line::from(net.netowrk_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                        ];
                        if let Some(date) = net.last_connected {
                            let formatted_date = date.format("%Y-%m-%d %H:%M").to_string();
                            row.push(Line::from(formatted_date));
                        }

                        Row::new(row)
                    } else {
                        let mut row = vec![
                            Line::from(""),
                            Line::from(net.name.clone()),
                            Line::from(net.netowrk_type.clone()).centered(),
                            Line::from(net.is_hidden.to_string()),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                        ];
                        if let Some(date) = net.last_connected {
                            let formatted_date = date.format("%Y-%m-%d %H:%M").to_string();
                            row.push(Line::from(formatted_date));
                        }

                        Row::new(row)
                    }
                } else {
                    let mut row = vec![
                        Line::from(""),
                        Line::from(net.name.clone()),
                        Line::from(net.netowrk_type.clone()).centered(),
                        Line::from(net.is_hidden.to_string()),
                        Line::from(net.is_autoconnect.to_string()),
                    ];

                    if let Some(date) = net.last_connected {
                        let formatted_date = date.format("%Y-%m-%d %H:%M").to_string();
                        row.push(Line::from(formatted_date));
                    }

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
            Constraint::Fill(1),
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
                        Cell::from("Last Connected").style(Style::default().fg(Color::Yellow)),
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
                        Cell::from("Last Connected").style(match color_mode {
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
            // .filter(|(net, _signal)| net.kn)
            .map(|(net, signal)| {
                Row::new(vec![
                    Line::from(net.name.clone()),
                    Line::from(net.netowrk_type.clone()).centered(),
                    Line::from({
                        match signal / 100 {
                            n if n >= -25 => String::from("󰤨"),
                            n if (-50..-25).contains(&n) => String::from("󰤥"),
                            n if (-75..-50).contains(&n) => String::from("󰤢"),
                            _ => String::from("󰤟"),
                        }
                    })
                    .centered(),
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
