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

pub async fn handle_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    if app.reset_mode {
        match key_event.code {
            KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Char('c') | KeyCode::Char('C') => {
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
        return Ok(());
    }

    match app.focused_block {
        FocusedBlock::AuthKey => match key_event.code {
            KeyCode::Enter => {
                app.send_passkey().await?;
                app.focused_block = FocusedBlock::Device;
            }

            KeyCode::Esc => {
                app.cancel_auth().await?;
                app.focused_block = FocusedBlock::Device;
            }

            KeyCode::Tab => {
                app.show_password = !app.show_password;
            }

            _ => {
                app.passkey_input
                    .handle_event(&crossterm::event::Event::Key(key_event));
            }
        },
        FocusedBlock::AccessPointInput => match key_event.code {
            KeyCode::Enter => {
                if let Some(ap) = &mut app.adapter.device.access_point {
                    ap.start(sender.clone()).await?;
                    sender.send(Event::Reset(app.current_mode.clone()))?;
                    app.focused_block = FocusedBlock::Device;
                }
            }

            KeyCode::Esc => {
                if let Some(ap) = &app.adapter.device.access_point {
                    // Start AP
                    ap.ap_start
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                }
                app.focused_block = FocusedBlock::AccessPoint;
            }
            KeyCode::Tab => {
                if let Some(ap) = &mut app.adapter.device.access_point {
                    match ap.focused_section {
                        APFocusedSection::SSID => ap.focused_section = APFocusedSection::PSK,
                        APFocusedSection::PSK => ap.focused_section = APFocusedSection::SSID,
                    }
                }
            }
            _ => {
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
        },
        _ => {
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

                // Switch mode
                KeyCode::Char(c)
                    if c == config.switch && key_event.modifiers == KeyModifiers::CONTROL =>
                {
                    app.reset_mode = true;
                }

                // Discard help popup
                KeyCode::Esc => {
                    if app.focused_block == FocusedBlock::Help
                        || app.focused_block == FocusedBlock::AdapterInfos
                    {
                        app.focused_block = FocusedBlock::Device;
                    }
                }

                // Start Scan
                KeyCode::Char(c) if c == config.station.start_scanning => {
                    match app.adapter.device.mode {
                        Mode::Station => {
                            app.adapter
                                .device
                                .station
                                .as_mut()
                                .unwrap()
                                .scan(sender)
                                .await?
                        }
                        Mode::Ap => {
                            app.adapter
                                .device
                                .access_point
                                .as_mut()
                                .unwrap()
                                .scan(sender)
                                .await?
                        }
                        _ => {}
                    };
                }

                KeyCode::Tab => match app.adapter.device.mode {
                    Mode::Station => match app.focused_block {
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
                    Mode::Ap => match app.focused_block {
                        FocusedBlock::Device => {
                            app.focused_block = FocusedBlock::AccessPoint;
                        }
                        FocusedBlock::AccessPoint => {
                            if let Some(ap) = app.adapter.device.access_point.as_ref() {
                                if ap.connected_devices.is_empty() {
                                    app.focused_block = FocusedBlock::Device;
                                } else {
                                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                                }
                            }
                        }
                        FocusedBlock::AccessPointConnectedDevices => {
                            app.focused_block = FocusedBlock::Device;
                        }
                        FocusedBlock::AccessPointInput => {
                            if let Some(ap) = &mut app.adapter.device.access_point {
                                match ap.focused_section {
                                    APFocusedSection::SSID => {
                                        ap.focused_section = APFocusedSection::PSK
                                    }
                                    APFocusedSection::PSK => {
                                        ap.focused_section = APFocusedSection::SSID
                                    }
                                };
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },

                KeyCode::BackTab => match app.adapter.device.mode {
                    Mode::Station => match app.focused_block {
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
                    Mode::Ap => match app.focused_block {
                        FocusedBlock::Device => {
                            if let Some(ap) = app.adapter.device.access_point.as_ref() {
                                if ap.connected_devices.is_empty() {
                                    app.focused_block = FocusedBlock::AccessPoint;
                                } else {
                                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                                }
                            }
                        }
                        FocusedBlock::AccessPoint => {
                            app.focused_block = FocusedBlock::Device;
                        }
                        FocusedBlock::AccessPointConnectedDevices => {
                            app.focused_block = FocusedBlock::AccessPoint;
                        }
                        FocusedBlock::AccessPointInput => {
                            if let Some(ap) = &mut app.adapter.device.access_point {
                                match ap.focused_section {
                                    APFocusedSection::SSID => {
                                        ap.focused_section = APFocusedSection::PSK
                                    }
                                    APFocusedSection::PSK => {
                                        ap.focused_section = APFocusedSection::SSID
                                    }
                                };
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },

                _ => {
                    match app.focused_block {
                        FocusedBlock::Device => match key_event.code {
                            KeyCode::Char(c) if c == config.device.infos => {
                                app.focused_block = FocusedBlock::AdapterInfos;
                            }

                            KeyCode::Char(c) if c == config.device.toggle_power => {
                                if app.adapter.device.is_powered {
                                    match app.adapter.device.power_off().await {
                                        Ok(()) => {
                                            sender.send(Event::Reset(app.current_mode.clone()))?;
                                            Notification::send(
                                                "Device Powered Off".to_string(),
                                                crate::notification::NotificationLevel::Info,
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                crate::notification::NotificationLevel::Error,
                                                sender.clone(),
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
                                                sender.clone(),
                                            )?;
                                        }
                                        Err(e) => {
                                            Notification::send(
                                                e.to_string(),
                                                crate::notification::NotificationLevel::Error,
                                                sender.clone(),
                                            )?;
                                        }
                                    }
                                }
                            }

                            _ => {}
                        },

                        _ => {
                            match app.adapter.device.mode {
                                Mode::Station => {
                                    match key_event.code {
                                        // Remove a known network
                                        KeyCode::Char(c)
                                            if c == config.station.known_network.remove
                                                && app.focused_block
                                                    == FocusedBlock::KnownNetworks =>
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
                                                let (net, _signal) = &app
                                                    .adapter
                                                    .device
                                                    .station
                                                    .as_ref()
                                                    .unwrap()
                                                    .known_networks[net_index];

                                                let known_net = net.known_network.as_ref().unwrap();
                                                known_net.forget(sender.clone()).await?;
                                            }
                                        }

                                        // Toggle autoconnect
                                        KeyCode::Char(c)
                                            if c == config
                                                .station
                                                .known_network
                                                .toggle_autoconnect
                                                && app.focused_block
                                                    == FocusedBlock::KnownNetworks =>
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
                                                let (net, _signal) = &app
                                                    .adapter
                                                    .device
                                                    .station
                                                    .as_ref()
                                                    .unwrap()
                                                    .known_networks[net_index];

                                                let known_net = net.known_network.as_ref().unwrap();
                                                known_net
                                                    .toggle_autoconnect(sender.clone())
                                                    .await?;
                                            }
                                        }

                                        // Connect/Disconnect
                                        KeyCode::Char(c) if c == config.station.toggle_connect => {
                                            match app.focused_block {
                                                FocusedBlock::NewNetworks => {
                                                    if let Some(net_index) = app
                                                        .adapter
                                                        .device
                                                        .station
                                                        .as_ref()
                                                        .unwrap()
                                                        .new_networks_state
                                                        .selected()
                                                    {
                                                        let (net, _) = app
                                                            .adapter
                                                            .device
                                                            .station
                                                            .as_ref()
                                                            .unwrap()
                                                            .new_networks[net_index]
                                                            .clone();

                                                        let mode = app.current_mode.clone();
                                                        tokio::spawn(async move {
                                                            net.connect(sender.clone())
                                                                .await
                                                                .unwrap();

                                                            sender
                                                                .clone()
                                                                .send(Event::Reset(mode))
                                                                .unwrap();
                                                        });
                                                    }
                                                }
                                                FocusedBlock::KnownNetworks => {
                                                    match &app
                                                        .adapter
                                                        .device
                                                        .station
                                                        .as_ref()
                                                        .unwrap()
                                                        .connected_network
                                                    {
                                                        Some(connected_net) => {
                                                            if let Some(selected_net_index) = app
                                                                .adapter
                                                                .device
                                                                .station
                                                                .as_ref()
                                                                .unwrap()
                                                                .known_networks_state
                                                                .selected()
                                                            {
                                                                let (selected_net, _signal) = &app
                                                                    .adapter
                                                                    .device
                                                                    .station
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .known_networks
                                                                    [selected_net_index];

                                                                if selected_net.name
                                                                    == connected_net.name
                                                                {
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
                                                                        .position(|(n, _s)| {
                                                                            n.name
                                                                                == selected_net.name
                                                                        });

                                                                    if let Some(index) = net_index {
                                                                        let (net, _) = app
                                                                            .adapter
                                                                            .device
                                                                            .station
                                                                            .as_ref()
                                                                            .unwrap()
                                                                            .known_networks[index]
                                                                            .clone();
                                                                        app.adapter
                                                                            .device
                                                                            .station
                                                                            .as_ref()
                                                                            .unwrap()
                                                                            .disconnect(
                                                                                sender.clone(),
                                                                            )
                                                                            .await?;
                                                                        tokio::spawn(async move {
                                                                            net.connect(
                                                                                sender.clone(),
                                                                            )
                                                                            .await
                                                                            .unwrap();
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        None => {
                                                            if let Some(selected_net_index) = app
                                                                .adapter
                                                                .device
                                                                .station
                                                                .as_ref()
                                                                .unwrap()
                                                                .known_networks_state
                                                                .selected()
                                                            {
                                                                let (selected_net, _signal) = &app
                                                                    .adapter
                                                                    .device
                                                                    .station
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .known_networks
                                                                    [selected_net_index];
                                                                let net_index = app
                                                                    .adapter
                                                                    .device
                                                                    .station
                                                                    .as_ref()
                                                                    .unwrap()
                                                                    .known_networks
                                                                    .iter()
                                                                    .position(|(n, _s)| {
                                                                        n.name == selected_net.name
                                                                    });

                                                                if let Some(index) = net_index {
                                                                    let (net, _) = app
                                                                        .adapter
                                                                        .device
                                                                        .station
                                                                        .as_ref()
                                                                        .unwrap()
                                                                        .known_networks[index]
                                                                        .clone();
                                                                    tokio::spawn(async move {
                                                                        net.connect(sender.clone())
                                                                            .await
                                                                            .unwrap();
                                                                    });
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }

                                        // Scroll down
                                        KeyCode::Char('j') | KeyCode::Down => {
                                            match app.focused_block {
                                                FocusedBlock::KnownNetworks => {
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
                                                FocusedBlock::NewNetworks => {
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

                                                FocusedBlock::Help => {
                                                    app.help.scroll_down();
                                                }
                                                _ => {}
                                            }
                                        }

                                        KeyCode::Char('k') | KeyCode::Up => match app.focused_block
                                        {
                                            FocusedBlock::KnownNetworks => {
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
                                            FocusedBlock::NewNetworks => {
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
                                            FocusedBlock::Help => {
                                                app.help.scroll_up();
                                            }
                                            _ => {}
                                        },
                                        _ => {}
                                    }
                                }
                                Mode::Ap => match key_event.code {
                                    KeyCode::Char(c) if c == config.ap.start => {
                                        if let Some(ap) = &app.adapter.device.access_point {
                                            // Start AP
                                            ap.ap_start
                                                .store(true, std::sync::atomic::Ordering::Relaxed);
                                        }
                                    }
                                    KeyCode::Char(c) if c == config.ap.stop => {
                                        if let Some(ap) = &mut app.adapter.device.access_point {
                                            ap.stop(sender).await?;
                                            ap.connected_devices = Vec::new();
                                        }
                                    }

                                    // Scroll down
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        if app.focused_block == FocusedBlock::Help {
                                            app.help.scroll_down();
                                        }
                                    }

                                    KeyCode::Char('k') | KeyCode::Up => {
                                        if app.focused_block == FocusedBlock::Help {
                                            app.help.scroll_up();
                                        }
                                    }

                                    _ => {}
                                },
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
