use std::{
    error,
    sync::{atomic::AtomicBool, Arc},
};
use tui_input::Input;

use async_channel::{Receiver, Sender};
use futures::FutureExt;
use iwdrs::{agent::Agent, session::Session};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
    Frame,
};
use ratatui::{text::Line, widgets::TableState};

use crate::{
    config::Config, device::Device, help::Help, notification::Notification, station::Station,
};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Device,
    KnownNetworks,
    NewNetworks,
    Help,
    AuthKey,
    DeviceInfos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Dark,
    Light,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub focused_block: FocusedBlock,
    pub help: Help,
    pub color_mode: ColorMode,
    pub notifications: Vec<Notification>,
    pub session: Arc<Session>,
    pub device: Device,
    pub station: Station,
    pub agent_manager: iwdrs::agent::AgentManager,
    pub authentication_required: Arc<AtomicBool>,
    pub passkey_sender: Sender<String>,
    pub known_networks_state: TableState,
    pub new_networks_state: TableState,
    pub passkey_input: Input,
    pub refresh_new_network_state: Arc<AtomicBool>,
    pub refresh_known_network_state: Arc<AtomicBool>,
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx: Receiver<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    authentication_required.store(true, std::sync::atomic::Ordering::Relaxed);
    match rx.recv().await {
        Ok(passkey) => Ok(passkey),
        Err(e) => Err(e.into()),
    }
}

impl App {
    pub async fn new(config: Arc<Config>) -> AppResult<Self> {
        let session = Arc::new(iwdrs::session::Session::new().await?);

        let (s, r) = async_channel::unbounded();

        let iwdrs_station = session.station().unwrap();

        let station = Station::new(iwdrs_station).await?;

        let mut new_networks_state = TableState::default();
        if station.new_networks.is_empty() {
            new_networks_state.select(None);
        } else {
            new_networks_state.select(Some(0));
        }
        let mut known_networks_state = TableState::default();
        if station.known_networks.is_empty() {
            known_networks_state.select(None);
        } else {
            known_networks_state.select(Some(0));
        }

        let iwdrs_device = session.device().unwrap();
        let device = Device::new(iwdrs_device).await?;

        let authentication_required = Arc::new(AtomicBool::new(false));
        let authentication_required_caller = authentication_required.clone();

        let agent = Agent {
            request_passphrase_fn: Box::new(move || {
                {
                    let auth_clone = authentication_required_caller.clone();
                    request_confirmation(auth_clone, r.clone())
                }
                .boxed()
            }),
        };

        let agent_manager = session.register_agent(agent).await?;

        let color_mode = match terminal_light::luma() {
            Ok(luma) if luma > 0.6 => ColorMode::Light,
            Ok(_) => ColorMode::Dark,
            Err(_) => ColorMode::Dark,
        };

        let refresh_new_network_state = Arc::new(AtomicBool::new(false));
        let refresh_known_network_state = Arc::new(AtomicBool::new(false));

        Ok(Self {
            running: true,
            focused_block: FocusedBlock::Device,
            help: Help::new(config),
            color_mode,
            notifications: Vec::new(),
            session,
            device,
            station,
            agent_manager,
            authentication_required: authentication_required.clone(),
            passkey_sender: s,
            known_networks_state,
            new_networks_state,
            passkey_input: Input::default(),
            refresh_new_network_state,
            refresh_known_network_state,
        })
    }

    pub async fn send_passkey(&mut self) -> AppResult<()> {
        let passkey: String = self.passkey_input.value().into();
        self.passkey_sender.send(passkey).await?;
        self.authentication_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passkey_input.reset();
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame) {
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
        let row = Row::new(vec![
            self.device.name.clone(),
            self.device.mode.clone(),
            self.station.state.clone(),
            self.station.is_scanning.clone().to_string(),
            self.device.is_powered.clone().to_string(),
        ]);

        let widths = [
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
        ];

        let device_table = Table::new(vec![row], widths)
            .header({
                if self.focused_block == FocusedBlock::Device {
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
                        Cell::from("Name").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Mode").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("State").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Scanning").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Powered").style(match self.color_mode {
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
                        if self.focused_block == FocusedBlock::Device {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if self.focused_block == FocusedBlock::Device {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if self.focused_block == FocusedBlock::Device {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(3)
            .style(match self.color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if self.focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        // Known networks

        let rows: Vec<Row> = self
            .station
            .known_networks
            .iter()
            .map(|(net, _signal)| {
                let net = net.known_network.as_ref().unwrap();

                if let Some(connected_net) = &self.station.connected_network {
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
                if self.focused_block == FocusedBlock::KnownNetworks {
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
                        Cell::from("Name").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Security").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Hidden").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Auto Connect").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Last Connected").style(match self.color_mode {
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
                        if self.focused_block == FocusedBlock::KnownNetworks {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if self.focused_block == FocusedBlock::KnownNetworks {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if self.focused_block == FocusedBlock::KnownNetworks {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(4)
            .style(match self.color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if self.focused_block == FocusedBlock::KnownNetworks {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            known_networks_table,
            known_networks_block,
            &mut self.known_networks_state.clone(),
        );

        // New networks

        let rows: Vec<Row> = self
            .station
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
                if self.focused_block == FocusedBlock::NewNetworks {
                    Row::new(vec![
                        Cell::from("Name").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Security").style(Style::default().fg(Color::Yellow)),
                        Cell::from("Signal").style(Style::default().fg(Color::Yellow)),
                    ])
                    .style(Style::new().bold())
                    .bottom_margin(1)
                } else {
                    Row::new(vec![
                        Cell::from("Name").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Security").style(match self.color_mode {
                            ColorMode::Dark => Style::default().fg(Color::White),
                            ColorMode::Light => Style::default().fg(Color::Black),
                        }),
                        Cell::from("Signal").style(match self.color_mode {
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
                        if self.focused_block == FocusedBlock::NewNetworks {
                            Style::default().bold()
                        } else {
                            Style::default()
                        }
                    })
                    .borders(Borders::ALL)
                    .border_style({
                        if self.focused_block == FocusedBlock::NewNetworks {
                            Style::default().fg(Color::Green)
                        } else {
                            Style::default()
                        }
                    })
                    .border_type({
                        if self.focused_block == FocusedBlock::NewNetworks {
                            BorderType::Thick
                        } else {
                            BorderType::default()
                        }
                    }),
            )
            .column_spacing(4)
            .style(match self.color_mode {
                ColorMode::Dark => Style::default().fg(Color::White),
                ColorMode::Light => Style::default().fg(Color::Black),
            })
            .highlight_style(if self.focused_block == FocusedBlock::NewNetworks {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            new_networks_table,
            new_networks_block,
            &mut self.new_networks_state.clone(),
        );
    }

    pub fn refresh_network_state(&mut self) {
        if self
            .refresh_new_network_state
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let mut new_networks_state = TableState::default();
            if self.station.new_networks.is_empty() {
                new_networks_state.select(None);
            } else {
                new_networks_state.select(Some(0));
            }

            self.new_networks_state = new_networks_state;
        }

        if self
            .refresh_known_network_state
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            let mut known_networks_state = TableState::default();
            if self.station.known_networks.is_empty() {
                known_networks_state.select(None);
            } else {
                known_networks_state.select(Some(0));
            }
            self.known_networks_state = known_networks_state;
        }

        self.refresh_new_network_state
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.refresh_known_network_state
            .store(false, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn tick(&mut self) -> AppResult<()> {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        self.device.refresh().await?;

        self.station
            .refresh(
                self.refresh_new_network_state.clone(),
                self.refresh_known_network_state.clone(),
            )
            .await?;

        self.refresh_network_state();

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
