use std::sync::Arc;

use crate::access_point::APFocusedSection;
use crate::app::{App, AppResult, FocusedBlock};
use crate::config::Config;
use crate::event::Event;
use crate::notification::Notification;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use iwdrs::modes::Mode;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

async fn handle_reset_mode_key_event(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
) -> AppResult<()> {
    match key_event.code {
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c' | 'C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Char('j') => {
            if app.selected_mode == Mode::Station {
                app.selected_mode = Mode::Ap;
            }
        }
        KeyCode::Char('k') => {
            if app.selected_mode == Mode::Ap {
                app.selected_mode = Mode::Station;
            }
        }
        KeyCode::Enter => {
            sender.send(Event::Reset(app.selected_mode.clone()))?;
        }
        _ => {}
    }
    Ok(())
}

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    if app.reset_mode {
        return handle_reset_mode_key_event(key_event, app, sender).await;
    }
    match (
        app.adapter.device.mode.clone(),
        key_event.code,
        app.focused_block,
    ) {
        (_, KeyCode::Enter, FocusedBlock::AuthKey) => {
            app.send_passkey().await?;
            app.focused_block = FocusedBlock::Device;
        }
        (_, KeyCode::Esc, FocusedBlock::AuthKey) => {
            app.cancel_auth().await?;
            app.focused_block = FocusedBlock::Device;
        }
        (_, KeyCode::Tab, FocusedBlock::AuthKey) => {
            app.show_password = !app.show_password;
        }
        (_, _, FocusedBlock::AuthKey) => {
            app.passkey_input
                .handle_event(&crossterm::event::Event::Key(key_event));
        }
        (_, KeyCode::Enter, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                ap.start(sender.clone()).await?;
                sender.send(Event::Reset(app.current_mode.clone()))?;
                app.focused_block = FocusedBlock::Device;
            }
        }
        (_, KeyCode::Esc, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &app.adapter.device.access_point {
                // Start AP
                ap.ap_start
                    .store(false, std::sync::atomic::Ordering::Relaxed);
            }
            app.focused_block = FocusedBlock::AccessPoint;
        }
        (_, KeyCode::Tab, FocusedBlock::AccessPointInput) => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                match ap.focused_section {
                    APFocusedSection::SSID => ap.focused_section = APFocusedSection::PSK,
                    APFocusedSection::PSK => ap.focused_section = APFocusedSection::SSID,
                }
            }
        }
        (_, _, FocusedBlock::AccessPointInput) => {
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
        (_, KeyCode::Char('q'), _) => app.quit(),
        (_, KeyCode::Char('c' | 'C'), _) => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit()
            }
        }
        (_, KeyCode::Char('?'), _) => app.focused_block = FocusedBlock::Help,
        (_, KeyCode::Char(c), _)
            if c == config.switch && key_event.modifiers == KeyModifiers::CONTROL =>
        {
            app.reset_mode = true;
        }
        (_, KeyCode::Esc, _) => {
            if app.focused_block == FocusedBlock::Help
                || app.focused_block == FocusedBlock::AdapterInfos
            {
                app.focused_block = FocusedBlock::Device;
            }
        }
        (Mode::Station, KeyCode::Char(c), _) if c == config.station.start_scanning => {
            app.adapter
                .device
                .station
                .as_mut()
                .unwrap()
                .scan(sender)
                .await?;
        }
        (Mode::Ap, KeyCode::Char(c), _) if c == config.station.start_scanning => {
            app.adapter
                .device
                .access_point
                .as_mut()
                .unwrap()
                .scan(sender)
                .await?;
        }
        (_, KeyCode::Char(c), _) if c == config.station.start_scanning => {}
        (Mode::Station, KeyCode::Tab, _) => match app.focused_block {
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
        (Mode::Ap, KeyCode::Tab, FocusedBlock::Device) => {
            app.focused_block = FocusedBlock::AccessPoint
        }
        (Mode::Ap, KeyCode::Tab, FocusedBlock::AccessPoint) => {
            if let Some(ap) = app.adapter.device.access_point.as_ref() {
                if ap.connected_devices.is_empty() {
                    app.focused_block = FocusedBlock::Device;
                } else {
                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                }
            }
        }
        (Mode::Ap, KeyCode::Tab, FocusedBlock::AccessPointConnectedDevices) => {
            app.focused_block = FocusedBlock::Device;
        }
        (_, KeyCode::Tab, _) => {}
        (Mode::Station, KeyCode::BackTab, _) => match app.focused_block {
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
        (Mode::Ap, KeyCode::BackTab, FocusedBlock::Device) => {
            if let Some(ap) = app.adapter.device.access_point.as_ref() {
                if ap.connected_devices.is_empty() {
                    app.focused_block = FocusedBlock::AccessPoint;
                } else {
                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                }
            }
        }
        (Mode::Ap, KeyCode::BackTab, FocusedBlock::AccessPoint) => {
            app.focused_block = FocusedBlock::Device;
        }
        (Mode::Ap, KeyCode::BackTab, FocusedBlock::AccessPointConnectedDevices) => {
            app.focused_block = FocusedBlock::AccessPoint;
        }
        (_, KeyCode::BackTab, _) => {}
        (_, KeyCode::Char(c), FocusedBlock::Device) if c == config.device.infos => {
            app.focused_block = FocusedBlock::AdapterInfos;
        }
        (_, KeyCode::Char(c), FocusedBlock::Device) if c == config.device.toggle_power => {
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
        (_, _, FocusedBlock::Device) => {}
        (Mode::Station, KeyCode::Char(c), _)
            if c == config.station.known_network.remove
                && app.focused_block == FocusedBlock::KnownNetworks =>
        {
            if let Some(net_index) = app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .known_networks_state
                .selected()
            {
                let (net, _signal) =
                    &app.adapter.device.station.as_ref().unwrap().known_networks[net_index];
                let known_net = net.known_network.as_ref().unwrap();
                known_net.forget(sender.clone()).await?;
            }
        }
        // Toggle autoconnect
        (Mode::Station, KeyCode::Char(c), _)
            if c == config.station.known_network.toggle_autoconnect
                && app.focused_block == FocusedBlock::KnownNetworks =>
        {
            if let Some(net_index) = app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .known_networks_state
                .selected()
            {
                let (net, _signal) =
                    &app.adapter.device.station.as_ref().unwrap().known_networks[net_index];
                let known_net = net.known_network.as_ref().unwrap();
                known_net.toggle_autoconnect(sender.clone()).await?;
            }
        }
        // Connect/Disconnect
        (Mode::Station, KeyCode::Char(c), FocusedBlock::NewNetworks)
            if c == config.station.toggle_connect =>
        {
            if let Some(net_index) = app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .new_networks_state
                .selected()
            {
                let (net, _) =
                    app.adapter.device.station.as_ref().unwrap().new_networks[net_index].clone();
                let mode = app.current_mode.clone();
                tokio::spawn(async move {
                    net.connect(sender.clone()).await.unwrap();
                    sender.clone().send(Event::Reset(mode)).unwrap();
                });
            }
        }
        (Mode::Station, KeyCode::Char(c), FocusedBlock::KnownNetworks)
            if c == config.station.toggle_connect =>
        {
            if let Some(connected_net) = &app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .connected_network
            {
                let Some(selected_net_index) = app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .known_networks_state
                    .selected()
                else {
                    return Ok(());
                };
                let (selected_net, _signal) =
                    &app.adapter.device.station.as_ref().unwrap().known_networks
                        [selected_net_index];
                if selected_net.name == connected_net.name {
                    app.adapter
                        .device
                        .station
                        .as_ref()
                        .unwrap()
                        .disconnect(sender.clone())
                        .await?;
                } else {
                    let net_index = app
                        .adapter
                        .device
                        .station
                        .as_ref()
                        .unwrap()
                        .known_networks
                        .iter()
                        .position(|(n, _s)| n.name == selected_net.name);
                    if let Some(index) = net_index {
                        let (net, _) = app.adapter.device.station.as_ref().unwrap().known_networks
                            [index]
                            .clone();
                        app.adapter
                            .device
                            .station
                            .as_ref()
                            .unwrap()
                            .disconnect(sender.clone())
                            .await?;
                        tokio::spawn(async move {
                            net.connect(sender.clone()).await.unwrap();
                        });
                    }
                }
            } else {
                let Some(selected_net_index) = app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .known_networks_state
                    .selected()
                else {
                    return Ok(());
                };
                let (selected_net, _signal) =
                    &app.adapter.device.station.as_ref().unwrap().known_networks
                        [selected_net_index];
                let net_index = app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .known_networks
                    .iter()
                    .position(|(n, _s)| n.name == selected_net.name);
                if let Some(index) = net_index {
                    let (net, _) =
                        app.adapter.device.station.as_ref().unwrap().known_networks[index].clone();
                    tokio::spawn(async move {
                        net.connect(sender.clone()).await.unwrap();
                    });
                }
            }
        }
        // scroll down
        (Mode::Station, KeyCode::Char('j') | KeyCode::Down, FocusedBlock::KnownNetworks) => {
            if !app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .known_networks
                .is_empty()
            {
                let i = match app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .known_networks_state
                    .selected()
                {
                    Some(i) => {
                        if i < app
                            .adapter
                            .device
                            .station
                            .as_ref()
                            .unwrap()
                            .known_networks
                            .len()
                            - 1
                        {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                app.adapter
                    .device
                    .station
                    .as_mut()
                    .unwrap()
                    .known_networks_state
                    .select(Some(i));
            }
        }
        (Mode::Station, KeyCode::Char('j') | KeyCode::Down, FocusedBlock::NewNetworks) => {
            if !app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .new_networks
                .is_empty()
            {
                let i = match app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .new_networks_state
                    .selected()
                {
                    Some(i) => {
                        if i < app
                            .adapter
                            .device
                            .station
                            .as_ref()
                            .unwrap()
                            .new_networks
                            .len()
                            - 1
                        {
                            i + 1
                        } else {
                            i
                        }
                    }
                    None => 0,
                };
                app.adapter
                    .device
                    .station
                    .as_mut()
                    .unwrap()
                    .new_networks_state
                    .select(Some(i));
            }
        }
        (Mode::Station, KeyCode::Char('k') | KeyCode::Up, FocusedBlock::KnownNetworks) => {
            if !app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .known_networks
                .is_empty()
            {
                let i = match app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .known_networks_state
                    .selected()
                {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                app.adapter
                    .device
                    .station
                    .as_mut()
                    .unwrap()
                    .known_networks_state
                    .select(Some(i));
            }
        }
        (Mode::Station, KeyCode::Char('k') | KeyCode::Up, FocusedBlock::NewNetworks) => {
            if !app
                .adapter
                .device
                .station
                .as_ref()
                .unwrap()
                .new_networks
                .is_empty()
            {
                let i = match app
                    .adapter
                    .device
                    .station
                    .as_ref()
                    .unwrap()
                    .new_networks_state
                    .selected()
                {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                app.adapter
                    .device
                    .station
                    .as_mut()
                    .unwrap()
                    .new_networks_state
                    .select(Some(i));
            }
        }
        // Scroll down
        (Mode::Station, KeyCode::Char('j') | KeyCode::Down, FocusedBlock::Help) => {
            app.help.scroll_down();
        }
        // Scroll up
        (Mode::Station, KeyCode::Char('k') | KeyCode::Up, FocusedBlock::Help) => {
            app.help.scroll_up();
        }
        (Mode::Ap, KeyCode::Char(c), _) if c == config.ap.start => {
            if let Some(ap) = &app.adapter.device.access_point {
                // Start AP
                ap.ap_start
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        (Mode::Ap, KeyCode::Char(c), _) if c == config.ap.stop => {
            if let Some(ap) = &mut app.adapter.device.access_point {
                ap.stop(sender).await?;
                ap.connected_devices = Vec::new();
            }
        }
        // Scroll down
        (Mode::Ap, KeyCode::Char('j') | KeyCode::Down, FocusedBlock::Help) => {
            app.help.scroll_down();
        }
        // Scroll up
        (Mode::Ap, KeyCode::Char('k') | KeyCode::Up, FocusedBlock::Help) => {
            app.help.scroll_up();
        }
        _ => {}
    }
    Ok(())
}
