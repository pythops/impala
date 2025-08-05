use std::sync::Arc;

use crate::access_point::APFocusedSection;
use crate::app::{App, AppResult, FocusedBlock};
use crate::config::Config;
use crate::event::Event;
use crate::notification::Notification;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

pub async fn handle_station_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    let station = &mut app.adapter.device.station;
    match (key_event.code, app.focused_block) {
        (KeyCode::Enter, FocusedBlock::AuthKey) => {
            app.send_passkey().await?;
            app.focused_block = FocusedBlock::Device;
        }
        (KeyCode::Esc, FocusedBlock::AuthKey) => {
            app.cancel_auth().await?;
            app.focused_block = FocusedBlock::Device;
        }
        (KeyCode::Tab, FocusedBlock::AuthKey) => {
            app.show_password = !app.show_password;
        }
        (_, FocusedBlock::AuthKey) => {
            app.passkey_input
                .handle_event(&crossterm::event::Event::Key(key_event));
        }
        (KeyCode::Enter, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                ap.start(sender.clone()).await?;
                sender.send(Event::Reset(app.current_mode.clone()))?;
                app.focused_block = FocusedBlock::Device;
            }
        }
        (KeyCode::Esc, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &app.adapter.device.access_point {
                // Start AP
                ap.ap_start
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            app.focused_block = FocusedBlock::AccessPoint;
        }
        (KeyCode::Tab, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                match ap.focused_section {
                    APFocusedSection::SSID => ap.focused_section = APFocusedSection::PSK,
                    APFocusedSection::PSK => ap.focused_section = APFocusedSection::SSID,
                }
            }
        }
        (_, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                match ap.focused_section {
                    APFocusedSection::SSID => ap
                        .ssid
                        .handle_event(&crossterm::event::Event::Key(key_event)),
                    APFocusedSection::PSK => ap
                        .psk
                        .handle_event(&crossterm::event::Event::Key(key_event)),
                };
            }
        }
        (KeyCode::Char('q'), _) => app.quit(),
        (KeyCode::Char('c' | 'C'), _) => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
        (KeyCode::Char('?'), _) => app.focused_block = FocusedBlock::Help,
        (KeyCode::Char(c), _)
            if c == config.switch && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.reset_mode = true;
        }
        (KeyCode::Esc, _) => {
            if app.focused_block == FocusedBlock::Help
                || app.focused_block == FocusedBlock::AdapterInfos
            {
                app.focused_block = FocusedBlock::Device;
            }
        }
        (KeyCode::Char(c), FocusedBlock::Device) if c == config.device.infos => {
            app.focused_block = FocusedBlock::AdapterInfos;
        }
        (KeyCode::Char(c), FocusedBlock::Device) if c == config.device.toggle_power => {
            if app.adapter.device.is_powered {
                match app.adapter.device.power_off().await {
                    Ok(()) => {
                        sender.send(Event::Reset(app.current_mode.clone()))?;
                        Notification::send(
                            "Device Powered Off".to_string(),
                            crate::notification::NotificationLevel::Info,
                            &sender.clone(),
                        )?;
                    }
                    Err(e) => {
                        Notification::send(
                            e.to_string(),
                            crate::notification::NotificationLevel::Error,
                            &sender.clone(),
                        )?;
                    }
                }
            } else {
                match app.adapter.device.power_on().await {
                    Ok(()) => {
                        sender.send(Event::Reset(app.current_mode.clone()))?;
                        Notification::send(
                            "Device Powered On".to_string(),
                            crate::notification::NotificationLevel::Info,
                            &sender.clone(),
                        )?;
                    }
                    Err(e) => {
                        Notification::send(
                            e.to_string(),
                            crate::notification::NotificationLevel::Error,
                            &sender.clone(),
                        )?;
                    }
                }
            }
        }
        (KeyCode::Char(c), _) if c == config.station.start_scanning => {
            station.as_mut().unwrap().scan(sender).await?;
        }
        (KeyCode::Tab, _) => match app.focused_block {
            FocusedBlock::Device => {
                app.focused_block = FocusedBlock::Station;
            }
            FocusedBlock::Station => {
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
        (KeyCode::BackTab, _) => match app.focused_block {
            FocusedBlock::Device => {
                app.focused_block = FocusedBlock::NewNetworks;
            }
            FocusedBlock::Station => {
                app.focused_block = FocusedBlock::Device;
            }
            FocusedBlock::KnownNetworks => {
                app.focused_block = FocusedBlock::Station;
            }
            FocusedBlock::NewNetworks => {
                app.focused_block = FocusedBlock::KnownNetworks;
            }
            _ => {}
        },
        (KeyCode::Char(c), _)
            if c == config.station.known_network.remove
                && app.focused_block == FocusedBlock::KnownNetworks =>
        {
            if let Some(net_index) = station.as_ref().unwrap().known_networks_state.selected() {
                let (net, _signal) = &station.as_ref().unwrap().known_networks[net_index];
                let known_net = net.known_network.as_ref().unwrap();
                known_net.forget(sender.clone()).await?;
            }
        }
        // Toggle autoconnect
        (KeyCode::Char(c), _)
            if c == config.station.known_network.toggle_autoconnect
                && app.focused_block == FocusedBlock::KnownNetworks =>
        {
            if let Some(net_index) = station.as_ref().unwrap().known_networks_state.selected() {
                let (net, _signal) = &station.as_ref().unwrap().known_networks[net_index];
                let known_net = net.known_network.as_ref().unwrap();
                known_net.toggle_autoconnect(sender.clone()).await?;
            }
        }
        // Connect/Disconnect
        (KeyCode::Char(c), FocusedBlock::NewNetworks) if c == config.station.toggle_connect => {
            if let Some(net_index) = station.as_ref().unwrap().new_networks_state.selected() {
                let (net, _) = station.as_ref().unwrap().new_networks[net_index].clone();
                let mode = app.current_mode.clone();
                tokio::spawn(async move {
                    net.connect(sender.clone()).await.unwrap();
                    sender.clone().send(Event::Reset(mode)).unwrap();
                });
            }
        }
        (KeyCode::Char(c), FocusedBlock::KnownNetworks) if c == config.station.toggle_connect => {
            if let Some(connected_net) = &station.as_ref().unwrap().connected_network {
                let Some(selected_net_index) =
                    station.as_ref().unwrap().known_networks_state.selected()
                else {
                    return Ok(());
                };
                let (selected_net, _signal) =
                    &station.as_ref().unwrap().known_networks[selected_net_index];
                if selected_net.name == connected_net.name {
                    station.as_ref().unwrap().disconnect(sender.clone()).await?;
                } else {
                    let net_index = station
                        .as_ref()
                        .unwrap()
                        .known_networks
                        .iter()
                        .position(|(n, _s)| n.name == selected_net.name);
                    if let Some(index) = net_index {
                        let (net, _) = station.as_ref().unwrap().known_networks[index].clone();
                        station.as_ref().unwrap().disconnect(sender.clone()).await?;
                        tokio::spawn(async move {
                            net.connect(sender.clone()).await.unwrap();
                        });
                    }
                }
            } else {
                let Some(selected_net_index) =
                    station.as_ref().unwrap().known_networks_state.selected()
                else {
                    return Ok(());
                };
                let (selected_net, _signal) =
                    &station.as_ref().unwrap().known_networks[selected_net_index];
                let net_index = station
                    .as_ref()
                    .unwrap()
                    .known_networks
                    .iter()
                    .position(|(n, _s)| n.name == selected_net.name);
                if let Some(index) = net_index {
                    let (net, _) = station.as_ref().unwrap().known_networks[index].clone();
                    tokio::spawn(async move {
                        net.connect(sender.clone()).await.unwrap();
                    });
                }
            }
        }
        (KeyCode::Char('j') | KeyCode::Down, FocusedBlock::KnownNetworks) => {
            if !station.as_ref().unwrap().known_networks.is_empty() {
                let i = match station.as_ref().unwrap().known_networks_state.selected() {
                    Some(i) => {
                        if i < station.as_ref().unwrap().known_networks.len() - 1 {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                station
                    .as_mut()
                    .unwrap()
                    .known_networks_state
                    .select(Some(i));
            }
        }
        (KeyCode::Char('j') | KeyCode::Down, FocusedBlock::NewNetworks) => {
            if !station.as_ref().unwrap().new_networks.is_empty() {
                let i = match station.as_ref().unwrap().new_networks_state.selected() {
                    Some(i) => {
                        if i < station.as_ref().unwrap().new_networks.len() - 1 {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                station.as_mut().unwrap().new_networks_state.select(Some(i));
            }
        }
        (KeyCode::Char('k') | KeyCode::Up, FocusedBlock::KnownNetworks) => {
            if !station.as_ref().unwrap().known_networks.is_empty() {
                let i = match station.as_ref().unwrap().known_networks_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                station
                    .as_mut()
                    .unwrap()
                    .known_networks_state
                    .select(Some(i));
            }
        }
        (KeyCode::Char('k') | KeyCode::Up, FocusedBlock::NewNetworks) => {
            if !station.as_ref().unwrap().new_networks.is_empty() {
                let i = match station.as_ref().unwrap().new_networks_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                station.as_mut().unwrap().new_networks_state.select(Some(i));
            }
        }
        (KeyCode::Char('j') | KeyCode::Down, FocusedBlock::Help) => {
            app.help.scroll_down();
        }
        (KeyCode::Char('k') | KeyCode::Up, FocusedBlock::Help) => {
            app.help.scroll_up();
        }
        _ => {}
    }
    Ok(())
}
