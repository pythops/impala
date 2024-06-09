use std::sync::Arc;

use crate::app::{App, AppResult, FocusedBlock};
use crate::config::Config;
use crate::event::Event;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') => {
            app.quit();
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }

        // Show help
        KeyCode::Char('?') => {
            app.focused_block = FocusedBlock::Help;
        }

        // Discard help popup
        KeyCode::Esc => {
            if app.focused_block == FocusedBlock::Help
                || app.focused_block == FocusedBlock::DeviceInfos
            {
                app.focused_block = FocusedBlock::Device;
            }
        }

        // Start Scan
        KeyCode::Char(c) if c == config.start_scanning => {
            app.station.scan(sender).await?;
        }

        KeyCode::Tab => match app.focused_block {
            FocusedBlock::Device => {
                app.focused_block = FocusedBlock::KnownNetworks;
            }
            FocusedBlock::KnownNetworks => {
                app.focused_block = FocusedBlock::NewNetworks;
            }
            FocusedBlock::NewNetworks => {
                app.focused_block = FocusedBlock::Device;
            }
            _ => {}
        },

        _ => {
            match app.focused_block {
                FocusedBlock::AuthKey => match key_event.code {
                    KeyCode::Enter => {
                        app.send_passkey().await?;
                        app.focused_block = FocusedBlock::Device;
                    }
                    _ => {
                        app.passkey_input
                            .handle_event(&crossterm::event::Event::Key(key_event));
                    }
                },

                //TODO:
                FocusedBlock::Device => match key_event.code {
                    KeyCode::Char(c) if c == config.device.infos => {
                        app.focused_block = FocusedBlock::DeviceInfos;
                    }
                    _ => {}
                },

                _ => {
                    match key_event.code {
                        // Remove a known network
                        KeyCode::Char(c) if c == config.known_network.remove => {
                            if let Some(net_index) = app.known_networks_state.selected() {
                                let (net, _signal) = &app.station.known_networks[net_index];

                                let known_net = net.known_network.as_ref().unwrap();
                                known_net.forget(sender.clone()).await?;
                            }
                        }

                        // Connect/Disconnect
                        KeyCode::Char(c) if c == config.toggle_connect => match app.focused_block {
                            FocusedBlock::NewNetworks => {
                                if let Some(net_index) = app.new_networks_state.selected() {
                                    let (net, _) = app.station.new_networks[net_index].clone();
                                    tokio::spawn(async move {
                                        net.connect(sender.clone()).await.unwrap();
                                    });
                                }
                            }
                            FocusedBlock::KnownNetworks => {
                                match &app.station.connected_network {
                                    Some(connected_net) => {
                                        if let Some(selected_net_index) =
                                            app.known_networks_state.selected()
                                        {
                                            let (selected_net, _signal) =
                                                &app.station.known_networks[selected_net_index];

                                            if selected_net.name == connected_net.name {
                                                app.station.disconnect(sender.clone()).await?;
                                            } else {
                                                let net_index =
                                                    app.station.known_networks.iter().position(
                                                        |(n, _s)| n.name == selected_net.name,
                                                    );

                                                if net_index.is_some() {
                                                    let (net, _) = app.station.known_networks
                                                        [net_index.unwrap()]
                                                    .clone();
                                                    app.station.disconnect(sender.clone()).await?;
                                                    tokio::spawn(async move {
                                                        net.connect(sender.clone()).await.unwrap();
                                                    });
                                                }
                                            }
                                        }
                                    }
                                    None => {
                                        if let Some(selected_net_index) =
                                            app.known_networks_state.selected()
                                        {
                                            let (selected_net, _signal) =
                                                &app.station.known_networks[selected_net_index];
                                            let net_index =
                                                app.station.known_networks.iter().position(
                                                    |(n, _s)| n.name == selected_net.name,
                                                );

                                            if net_index.is_some() {
                                                let (net, _) = app.station.known_networks
                                                    [net_index.unwrap()]
                                                .clone();
                                                tokio::spawn(async move {
                                                    net.connect(sender.clone()).await.unwrap();
                                                });
                                            }
                                        }
                                    }
                                }
                                // app.station.disconnect(sender.clone()).await.unwrap();
                            }
                            _ => {}
                        },

                        // Scroll down
                        KeyCode::Char('j') | KeyCode::Down => match app.focused_block {
                            FocusedBlock::Device => {}
                            FocusedBlock::KnownNetworks => {
                                if !app.station.known_networks.is_empty() {
                                    let i = match app.known_networks_state.selected() {
                                        Some(i) => {
                                            if i < app.station.known_networks.len() - 1 {
                                                i + 1
                                            } else {
                                                i
                                            }
                                        }
                                        None => 0,
                                    };

                                    app.known_networks_state.select(Some(i));
                                }
                            }
                            FocusedBlock::NewNetworks => {
                                if !app.station.new_networks.is_empty() {
                                    let i = match app.new_networks_state.selected() {
                                        Some(i) => {
                                            if i < app.station.new_networks.len() - 1 {
                                                i + 1
                                            } else {
                                                i
                                            }
                                        }
                                        None => 0,
                                    };

                                    app.new_networks_state.select(Some(i));
                                }
                            }

                            FocusedBlock::Help => {
                                app.help.scroll_down();
                            }
                            _ => {}
                        },

                        KeyCode::Char('k') | KeyCode::Up => match app.focused_block {
                            FocusedBlock::Device => {}
                            FocusedBlock::KnownNetworks => {
                                if !app.station.known_networks.is_empty() {
                                    let i = match app.known_networks_state.selected() {
                                        Some(i) => {
                                            if i > 1 {
                                                i - 1
                                            } else {
                                                0
                                            }
                                        }
                                        None => 0,
                                    };

                                    app.known_networks_state.select(Some(i));
                                }
                            }
                            FocusedBlock::NewNetworks => {
                                if !app.station.new_networks.is_empty() {
                                    let i = match app.new_networks_state.selected() {
                                        Some(i) => {
                                            if i > 1 {
                                                i - 1
                                            } else {
                                                0
                                            }
                                        }
                                        None => 0,
                                    };

                                    app.new_networks_state.select(Some(i));
                                }
                            }
                            FocusedBlock::Help => {
                                app.help.scroll_up();
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}
