use anyhow::Result;
use std::sync::Arc;

use crate::app::{App, FocusedBlock};
use crate::config::Config;
use crate::device::Device;
use crate::event::Event;
use crate::mode::ap::APFocusedSection;
use crate::notification::Notification;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use iwdrs::modes::Mode;
use iwdrs::network::NetworkType;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

pub async fn toggle_connect(app: &mut App, sender: UnboundedSender<Event>) -> Result<()> {
    if let Some(station) = &mut app.device.station {
        match app.focused_block {
            FocusedBlock::NewNetworks => {
                if let Some(net_index) = station.new_networks_state.selected() {
                    let (net, _) = station.new_networks[net_index].clone();

                    if net.network_type == NetworkType::Eap {
                        sender.send(Event::ConfigureNewEapNetwork(net.name.clone()))?;
                        return Ok(());
                    }
                    tokio::spawn(async move {
                        let _ = net.connect(sender.clone()).await;
                    });
                }
            }
            FocusedBlock::KnownNetworks => match &station.connected_network {
                Some(connected_net) => {
                    if let Some(selected_net_index) = station.known_networks_state.selected() {
                        if selected_net_index > station.known_networks.len() - 1 {
                            // Can not connect to unavailble network
                            return Ok(());
                        }

                        let (selected_net, _signal) = &station.known_networks[selected_net_index];

                        if selected_net.name == connected_net.name {
                            station.disconnect(sender.clone()).await?;
                        } else {
                            let net_index = station
                                .known_networks
                                .iter()
                                .position(|(n, _s)| n.name == selected_net.name);

                            if let Some(index) = net_index {
                                let (net, _) = station.known_networks[index].clone();
                                station.disconnect(sender.clone()).await?;
                                tokio::spawn(async move {
                                    let _ = net.connect(sender.clone()).await;
                                });
                            }
                        }
                    }
                }
                None => {
                    if let Some(selected_net_index) = station.known_networks_state.selected() {
                        if selected_net_index > station.known_networks.len() - 1 {
                            // Can not connect to unavailble network
                            return Ok(());
                        }
                        let (selected_net, _signal) = &station.known_networks[selected_net_index];
                        let net_index = station
                            .known_networks
                            .iter()
                            .position(|(n, _s)| n.name == selected_net.name);

                        if let Some(index) = net_index {
                            let (net, _) = station.known_networks[index].clone();
                            tokio::spawn(async move {
                                let _ = net.connect(sender.clone()).await;
                            });
                        }
                    }
                }
            },
            _ => {}
        }
    }
    Ok(())
}

async fn toggle_device_power(sender: UnboundedSender<Event>, device: &Device) -> Result<()> {
    if device.is_powered {
        match device.power_off().await {
            Ok(()) => {
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
        match device.power_on().await {
            Ok(()) => {
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
    Ok(())
}

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> Result<()> {
    if app.reset.enable {
        match key_event.code {
            KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Esc if app.config.esc_quit => {
                app.quit();
            }
            KeyCode::Char('c' | 'C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    app.quit();
                }
            }

            KeyCode::Char('j') => {
                if app.reset.selected_mode == Mode::Station {
                    app.reset.selected_mode = Mode::Ap;
                }
            }

            KeyCode::Char('k') => {
                if app.reset.selected_mode == Mode::Ap {
                    app.reset.selected_mode = Mode::Station;
                }
            }

            KeyCode::Enter => {
                sender.send(Event::Reset(app.reset.selected_mode))?;
            }

            _ => {}
        }
        return Ok(());
    }

    if !app.device.is_powered {
        match app.focused_block {
            FocusedBlock::AdapterInfos => {
                if key_event.code == KeyCode::Esc {
                    app.focused_block = FocusedBlock::Device;
                }
            }

            FocusedBlock::Device => match key_event.code {
                KeyCode::Char('q') => {
                    app.quit();
                }
                KeyCode::Esc if app.config.esc_quit => {
                    app.quit();
                }

                KeyCode::Char('c' | 'C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.quit();
                    }
                }

                KeyCode::Char(c) if c == config.device.infos => {
                    app.focused_block = FocusedBlock::AdapterInfos;
                }
                KeyCode::Char(c) if c == config.device.toggle_power => {
                    toggle_device_power(sender, &app.device).await?;
                }
                _ => {}
            },
            _ => {}
        }

        return Ok(());
    }

    match app.device.mode {
        Mode::Station => {
            if let Some(station) = &mut app.device.station {
                match app.focused_block {
                    FocusedBlock::PskAuthKey => match key_event.code {
                        KeyCode::Enter => {
                            app.auth.psk.submit(&app.agent).await?;
                            app.focused_block = FocusedBlock::NewNetworks;
                        }

                        KeyCode::Esc => {
                            app.auth.psk.cancel(&app.agent).await?;
                            app.focused_block = FocusedBlock::NewNetworks;
                        }

                        KeyCode::Tab => {
                            app.auth.psk.show_password = !app.auth.psk.show_password;
                        }

                        _ => {
                            app.auth
                                .psk
                                .passphrase
                                .handle_event(&crossterm::event::Event::Key(key_event));
                        }
                    },

                    FocusedBlock::RequestKeyPasshphrase => {
                        if let Some(req) = &mut app.auth.request_key_passphrase {
                            match key_event.code {
                                KeyCode::Enter => {
                                    req.submit(&app.agent).await?;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                KeyCode::Esc => {
                                    req.cancel(&app.agent).await?;
                                    app.auth.request_key_passphrase = None;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                KeyCode::Tab => {
                                    req.show_password = !req.show_password;
                                }

                                _ => {
                                    req.passphrase
                                        .handle_event(&crossterm::event::Event::Key(key_event));
                                }
                            }
                        }
                    }
                    FocusedBlock::RequestPassword => {
                        if let Some(req) = &mut app.auth.request_password {
                            match key_event.code {
                                KeyCode::Enter => {
                                    req.submit(&app.agent).await?;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                KeyCode::Esc => {
                                    req.cancel(&app.agent).await?;
                                    app.auth.request_password = None;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                KeyCode::Tab => {
                                    req.show_password = !req.show_password;
                                }

                                _ => {
                                    req.password
                                        .handle_event(&crossterm::event::Event::Key(key_event));
                                }
                            }
                        }
                    }
                    FocusedBlock::RequestUsernameAndPassword => {
                        if let Some(req) = &mut app.auth.request_username_and_password {
                            match key_event.code {
                                KeyCode::Enter => {
                                    req.submit(&app.agent).await?;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                KeyCode::Esc => {
                                    req.cancel(&app.agent).await?;
                                    app.auth.request_username_and_password = None;
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }

                                _ => {
                                    req.handle_key_events(key_event, sender).await?;
                                }
                            }
                        }
                    }

                    FocusedBlock::WpaEntrepriseAuth => match key_event.code {
                        KeyCode::Esc => {
                            app.focused_block = FocusedBlock::NewNetworks;
                            app.auth.eap = None;
                        }

                        _ => {
                            if let Some(eap) = &mut app.auth.eap {
                                eap.handle_key_events(key_event, sender);
                            }
                        }
                    },
                    FocusedBlock::AdapterInfos => {
                        if key_event.code == KeyCode::Esc {
                            app.focused_block = FocusedBlock::Device;
                        }
                    }
                    _ => {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                app.quit();
                            }
                            KeyCode::Esc if app.config.esc_quit => {
                                app.quit();
                            }

                            KeyCode::Char('c' | 'C') => {
                                if key_event.modifiers == KeyModifiers::CONTROL {
                                    app.quit();
                                }
                            }

                            // Switch mode
                            KeyCode::Char(c)
                                if c == config.switch
                                    && key_event.modifiers == KeyModifiers::CONTROL =>
                            {
                                app.reset.enable = true;
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
                            KeyCode::BackTab => match app.focused_block {
                                FocusedBlock::Device => {
                                    app.focused_block = FocusedBlock::NewNetworks;
                                }
                                FocusedBlock::NewNetworks => {
                                    app.focused_block = FocusedBlock::KnownNetworks;
                                }
                                FocusedBlock::KnownNetworks => {
                                    app.focused_block = FocusedBlock::Device;
                                }
                                _ => {}
                            },

                            KeyCode::Char(c) if c == config.station.start_scanning => {
                                station.scan(sender).await?;
                            }
                            _ => match app.focused_block {
                                FocusedBlock::Device => match key_event.code {
                                    KeyCode::Char(c) if c == config.device.infos => {
                                        app.focused_block = FocusedBlock::AdapterInfos;
                                    }
                                    KeyCode::Char(c) if c == config.device.toggle_power => {
                                        toggle_device_power(sender, &app.device).await?;
                                    }
                                    _ => {}
                                },

                                FocusedBlock::KnownNetworks => {
                                    match key_event.code {
                                        // Remove a known network
                                        KeyCode::Char(c)
                                            if c == config.station.known_network.remove =>
                                        {
                                            if let Some(net_index) =
                                                station.known_networks_state.selected()
                                            {
                                                if net_index > station.known_networks.len() - 1 {
                                                    let index = net_index.saturating_sub(
                                                        station.known_networks.len(),
                                                    );
                                                    let network =
                                                        &station.unavailable_known_networks[index];
                                                    network.forget(sender.clone()).await?;
                                                } else {
                                                    let (net, _signal) =
                                                        &station.known_networks[net_index];

                                                    if let Some(known_net) = &net.known_network {
                                                        known_net.forget(sender.clone()).await?;
                                                    }
                                                }
                                            }
                                        }

                                        // Toggle autoconnect
                                        KeyCode::Char(c)
                                            if c == config
                                                .station
                                                .known_network
                                                .toggle_autoconnect =>
                                        {
                                            if let Some(net_index) =
                                                station.known_networks_state.selected()
                                                && net_index < station.known_networks.len()
                                            {
                                                let (net, _) = &station.known_networks[net_index];

                                                if let Some(known_net) = &net.known_network {
                                                    known_net
                                                        .toggle_autoconnect(sender.clone())
                                                        .await?;
                                                }
                                            }
                                        }

                                        // Show / Hide unavailable networks
                                        KeyCode::Char(c)
                                            if c == config.station.known_network.show_all =>
                                        {
                                            station.show_unavailable_known_networks =
                                                !station.show_unavailable_known_networks;
                                        }

                                        // Connect/Disconnect
                                        KeyCode::Enter => toggle_connect(app, sender).await?,
                                        KeyCode::Char(c) if c == config.station.toggle_connect => {
                                            toggle_connect(app, sender).await?
                                        }

                                        // Scroll down
                                        KeyCode::Char('j') | KeyCode::Down => {
                                            if !station.known_networks.is_empty() {
                                                let i =
                                                    match station.known_networks_state.selected() {
                                                        Some(i) => {
                                                            let limit = if station
                                                                .show_unavailable_known_networks
                                                            {
                                                                station.new_networks.len()
                                                                    + station
                                                                        .unavailable_known_networks
                                                                        .len()
                                                                    - 1
                                                            } else {
                                                                station.new_networks.len() - 1
                                                            };

                                                            if i < limit { i + 1 } else { i }
                                                        }
                                                        None => 0,
                                                    };

                                                station.known_networks_state.select(Some(i));
                                            }
                                        }
                                        KeyCode::Char('k') | KeyCode::Up => {
                                            if !station.known_networks.is_empty() {
                                                let i =
                                                    match station.known_networks_state.selected() {
                                                        Some(i) => i.saturating_sub(1),
                                                        None => 0,
                                                    };

                                                station.known_networks_state.select(Some(i));
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                FocusedBlock::NewNetworks => match key_event.code {
                                    KeyCode::Enter => toggle_connect(app, sender).await?,
                                    KeyCode::Char(c) if c == config.station.toggle_connect => {
                                        toggle_connect(app, sender).await?
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        if !station.new_networks.is_empty() {
                                            let i = match station.new_networks_state.selected() {
                                                Some(i) => {
                                                    if i < station.new_networks.len() - 1 {
                                                        i + 1
                                                    } else {
                                                        i
                                                    }
                                                }
                                                None => 0,
                                            };

                                            station.new_networks_state.select(Some(i));
                                        }
                                    }
                                    KeyCode::Char('k') | KeyCode::Up => {
                                        if !station.new_networks.is_empty() {
                                            let i = match station.new_networks_state.selected() {
                                                Some(i) => i.saturating_sub(1),
                                                None => 0,
                                            };

                                            station.new_networks_state.select(Some(i));
                                        }
                                    }
                                    _ => {}
                                },
                                _ => {}
                            },
                        }
                    }
                }
            } else {
                sender.send(Event::Reset(Mode::Station))?;
            }
        }

        Mode::Ap => {
            if let Some(ap) = &mut app.device.ap {
                match app.focused_block {
                    FocusedBlock::AccessPointInput => match key_event.code {
                        KeyCode::Enter => {
                            ap.start(sender.clone()).await?;
                            app.focused_block = FocusedBlock::Device;
                        }

                        KeyCode::Esc => {
                            ap.ap_start
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                            app.focused_block = FocusedBlock::AccessPoint;
                        }
                        KeyCode::Tab => match ap.focused_section {
                            APFocusedSection::SSID => {
                                ap.focused_section = APFocusedSection::PSK;
                            }
                            APFocusedSection::PSK => {
                                ap.focused_section = APFocusedSection::SSID;
                            }
                        },
                        _ => match ap.focused_section {
                            APFocusedSection::SSID => {
                                ap.ssid
                                    .handle_event(&crossterm::event::Event::Key(key_event));
                            }
                            APFocusedSection::PSK => {
                                ap.psk
                                    .handle_event(&crossterm::event::Event::Key(key_event));
                            }
                        },
                    },

                    FocusedBlock::AdapterInfos => {
                        if key_event.code == KeyCode::Esc {
                            app.focused_block = FocusedBlock::Device;
                        }
                    }
                    _ => {
                        match key_event.code {
                            KeyCode::Char('q') => {
                                app.quit();
                            }
                            KeyCode::Esc if app.config.esc_quit => {
                                app.quit();
                            }

                            KeyCode::Char('c' | 'C') => {
                                if key_event.modifiers == KeyModifiers::CONTROL {
                                    app.quit();
                                }
                            }

                            // Switch mode
                            KeyCode::Char(c)
                                if c == config.switch
                                    && key_event.modifiers == KeyModifiers::CONTROL =>
                            {
                                app.reset.enable = true;
                            }

                            KeyCode::Tab => match app.focused_block {
                                FocusedBlock::Device => {
                                    app.focused_block = FocusedBlock::AccessPoint;
                                }
                                FocusedBlock::AccessPoint => {
                                    if ap.connected_devices.is_empty() {
                                        app.focused_block = FocusedBlock::Device;
                                    } else {
                                        app.focused_block =
                                            FocusedBlock::AccessPointConnectedDevices;
                                    }
                                }
                                FocusedBlock::AccessPointConnectedDevices => {
                                    app.focused_block = FocusedBlock::Device;
                                }

                                _ => {}
                            },

                            _ => {
                                if app.focused_block == FocusedBlock::Device {
                                    match key_event.code {
                                        KeyCode::Char(c) if c == config.device.infos => {
                                            app.focused_block = FocusedBlock::AdapterInfos;
                                        }
                                        KeyCode::Char(c) if c == config.device.toggle_power => {
                                            toggle_device_power(sender, &app.device).await?;
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                sender.send(Event::Reset(Mode::Ap))?;
            }
        }
    }

    Ok(())
}
