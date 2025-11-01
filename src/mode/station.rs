pub mod auth;
pub mod known_network;
pub mod network;

use std::sync::Arc;

use futures::future::join_all;
use iwdrs::{
    session::Session,
    station::{State, diagnostics::ActiveStationDiagnostics},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Row, Table, TableState},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::{AppResult, FocusedBlock},
    config::Config,
    device::Device,
    event::Event,
    notification::{Notification, NotificationLevel},
};

use network::Network;

#[derive(Debug, Clone)]
pub struct Station {
    pub session: Arc<Session>,
    pub state: State,
    pub is_scanning: bool,
    pub connected_network: Option<Network>,
    pub new_networks: Vec<(Network, i16)>,
    pub known_networks: Vec<(Network, i16)>,
    pub known_networks_state: TableState,
    pub new_networks_state: TableState,
    pub diagnostic: Option<ActiveStationDiagnostics>,
}

impl Station {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let iwd_station = session
            .stations()
            .await
            .unwrap()
            .pop()
            .ok_or("no station found")?;

        let iwd_station_diagnostic = session.stations_diagnostics().await.unwrap().pop();

        let state = iwd_station.state().await?;
        let connected_network = {
            if let Some(n) = iwd_station.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network)
            } else {
                None
            }
        };

        let is_scanning = iwd_station.is_scanning().await?;
        let discovered_networks = iwd_station.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        let mut new_networks_state = TableState::default();
        if new_networks.is_empty() {
            new_networks_state.select(None);
        } else {
            new_networks_state.select(Some(0));
        }

        let mut known_networks_state = TableState::default();

        if known_networks.is_empty() {
            known_networks_state.select(None);
        } else {
            known_networks_state.select(Some(0));
        }

        let diagnostic = if let Some(diagnostic) = iwd_station_diagnostic {
            diagnostic.get().await.ok()
        } else {
            None
        };

        Ok(Self {
            session,
            state,
            is_scanning,
            connected_network,
            new_networks,
            known_networks,
            known_networks_state,
            new_networks_state,
            diagnostic,
        })
    }

    pub async fn refresh(&mut self) -> AppResult<()> {
        let iwd_station = self.session.stations().await.unwrap().pop().unwrap();

        self.state = iwd_station.state().await?;
        self.is_scanning = iwd_station.is_scanning().await?;

        let connected_network = {
            if let Some(n) = iwd_station.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network)
            } else {
                None
            }
        };

        let discovered_networks = iwd_station.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        if self.new_networks.len() == new_networks.len() {
            self.new_networks.iter_mut().for_each(|(net, signal)| {
                let n = new_networks
                    .iter()
                    .find(|(refreshed_net, _signal)| refreshed_net.name == net.name);

                if let Some((_, refreshed_signal)) = n {
                    *signal = *refreshed_signal;
                }
            });
        } else {
            let mut new_networks_state = TableState::default();
            if new_networks.is_empty() {
                new_networks_state.select(None);
            } else {
                new_networks_state.select(Some(0));
            }

            self.new_networks_state = new_networks_state;
            self.new_networks = new_networks;
        }

        if self.known_networks.len() == known_networks.len() {
            self.known_networks.iter_mut().for_each(|(net, signal)| {
                let n = known_networks
                    .iter()
                    .find(|(refreshed_net, _signal)| refreshed_net.name == net.name);

                if let Some((refreshed_net, refreshed_signal)) = n {
                    net.known_network.as_mut().unwrap().is_autoconnect =
                        refreshed_net.known_network.as_ref().unwrap().is_autoconnect;
                    *signal = *refreshed_signal;
                }
            });
        } else {
            let mut known_networks_state = TableState::default();
            if known_networks.is_empty() {
                known_networks_state.select(None);
            } else {
                known_networks_state.select(Some(0));
            }
            self.known_networks_state = known_networks_state;
            self.known_networks = known_networks;
        }

        self.connected_network = connected_network;

        let iwd_station_diagnostic = self.session.stations_diagnostics().await.unwrap().pop();
        self.diagnostic = if let Some(diagnostic) = iwd_station_diagnostic {
            diagnostic.get().await.ok()
        } else {
            None
        };

        Ok(())
    }

    pub async fn scan(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        let iwd_station = self.session.stations().await.unwrap().pop().unwrap();
        match iwd_station.scan().await {
            Ok(()) => Notification::send(
                "Start Scanning".to_string(),
                NotificationLevel::Info,
                &sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }

        Ok(())
    }

    pub async fn disconnect(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        let iwd_station = self.session.stations().await.unwrap().pop().unwrap();
        match iwd_station.disconnect().await {
            Ok(()) => Notification::send(
                format!(
                    "Disconnected from {}",
                    self.connected_network.as_ref().unwrap().name
                ),
                NotificationLevel::Info,
                &sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?,
        }
        Ok(())
    }

    pub fn render(
        &mut self,
        frame: &mut Frame,
        focused_block: FocusedBlock,
        device: &Device,
        config: Arc<Config>,
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

        //
        // Device
        //
        let row = Row::new(vec![
            Line::from(device.name.clone()).centered(),
            Line::from("station").centered(),
            {
                if device.is_powered {
                    Line::from("On").centered()
                } else {
                    Line::from("Off").centered()
                }
            },
            Line::from(self.state.to_string()).centered(),
            Line::from(self.is_scanning.to_string()).centered(),
            Line::from({
                if let Some(diagnostic) = &self.diagnostic {
                    format!("{:.2} GHz", diagnostic.frequency_mhz as f32 / 1000.)
                } else {
                    "-".to_string()
                }
            })
            .centered(),
            Line::from({
                if let Some(diagnostic) = &self.diagnostic {
                    diagnostic.security.to_string()
                } else {
                    "-".to_string()
                }
            })
            .centered(),
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
                        Line::from("Name").centered(),
                        Line::from("Mode").centered(),
                        Line::from("Powered").centered(),
                        Line::from("State").centered(),
                        Line::from("Scanning").centered(),
                        Line::from("Frequency").centered(),
                        Line::from("Security").centered(),
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
            .flex(Flex::SpaceAround)
            .row_highlight_style(if focused_block == FocusedBlock::Device {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        let mut device_state = TableState::default().with_selected(0);
        frame.render_stateful_widget(device_table, device_block, &mut device_state);

        //
        // Known networks
        //
        let rows: Vec<Row> = self
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

                if let Some(connected_net) = &self.connected_network {
                    if connected_net.name == net.name {
                        let row = vec![
                            Line::from("󰖩 ").centered(),
                            Line::from(net.name.clone()).centered(),
                            Line::from(net.network_type.to_string()).centered(),
                            Line::from(net.is_hidden.to_string()).centered(),
                            Line::from(net.is_autoconnect.to_string()).centered(),
                            Line::from(signal).centered(),
                        ];

                        Row::new(row)
                    } else {
                        let row = vec![
                            Line::from(""),
                            Line::from(net.name.clone()).centered(),
                            Line::from(net.network_type.to_string()).centered(),
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
                        Line::from(net.network_type.to_string()).centered(),
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
                        Line::from("Name").centered(),
                        Line::from("Security").centered(),
                        Line::from("Hidden").centered(),
                        Line::from("Auto Connect").centered(),
                        Line::from("Signal").centered(),
                    ])
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
            .column_spacing(1)
            .flex(Flex::SpaceAround)
            .row_highlight_style(if focused_block == FocusedBlock::KnownNetworks {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            known_networks_table,
            known_networks_block,
            &mut self.known_networks_state,
        );

        //
        // New networks
        //
        let rows: Vec<Row> = self
            .new_networks
            .iter()
            .map(|(net, signal)| {
                Row::new(vec![
                    Line::from(net.name.clone()).centered(),
                    Line::from(net.network_type.to_string().clone()).centered(),
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
                        Line::from("Name").centered(),
                        Line::from("Security").centered(),
                        Line::from("Signal").centered(),
                    ])
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
            .column_spacing(1)
            .flex(Flex::SpaceAround)
            .row_highlight_style(if focused_block == FocusedBlock::NewNetworks {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            });

        frame.render_stateful_widget(
            new_networks_table,
            new_networks_block,
            &mut self.new_networks_state,
        );

        let help_message = match focused_block {
            FocusedBlock::Device => Line::from(vec![
                Span::from(config.station.start_scanning.to_string()).bold(),
                Span::from(" Scan"),
                Span::from(" | "),
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
            FocusedBlock::KnownNetworks => Line::from(vec![
                Span::from("k,").bold(),
                Span::from("  Up"),
                Span::from(" | "),
                Span::from("j,").bold(),
                Span::from("  Down"),
                Span::from(" | "),
                Span::from(if config.station.toggle_connect == ' ' {
                    "󱁐  or ↵ ".to_string()
                } else {
                    config.station.toggle_connect.to_string()
                })
                .bold(),
                Span::from(" Dis/connect"),
                Span::from(" | "),
                Span::from(config.station.known_network.remove.to_string()).bold(),
                Span::from(" Remove"),
                Span::from(" | "),
                Span::from(config.station.known_network.toggle_autoconnect.to_string()).bold(),
                Span::from(" Autoconnect"),
                Span::from(" | "),
                Span::from(config.station.start_scanning.to_string()).bold(),
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
                Span::from("󱁐  or ↵ ").bold(),
                Span::from(" Connect"),
                Span::from(" | "),
                Span::from(config.station.start_scanning.to_string()).bold(),
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
            FocusedBlock::AdapterInfos => {
                Line::from(vec![Span::from("󱊷 ").bold(), Span::from(" Discard")])
            }
            FocusedBlock::PskAuthKey => Line::from(vec![
                Span::from("⇄").bold(),
                Span::from(" Hide/Show password"),
                Span::from(" | "),
                Span::from("󱊷 ").bold(),
                Span::from(" Discard"),
            ]),
            _ => Line::from(""),
        };

        let help_message = help_message.centered().blue();

        frame.render_widget(help_message, help_block);
    }
}
