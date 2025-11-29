use anyhow::{Result, anyhow};
use std::sync::Arc;

use crate::app::{App, FocusedBlock};
use crate::config::Config;
use crate::device::Device;
use crate::event::Event;
use crate::mode::ap::APFocusedSection;
use crate::mode::station::share::Share;
use crate::notification::{self, Notification, NotificationLevel};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use iwdrs::modes::Mode;
use iwdrs::network::NetworkType;
use tokio::sync::mpsc::UnboundedSender;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

pub async fn toggle_connect(app: &mut App, sender: UnboundedSender<Event>) -> Result<()> {
    if let Some(station) = &mut app.device.station {
        match app.focused_block {
            FocusedBlock::NewNetworks => {
                if let Some(net_index) = station.new_networks_state.selected() {
                    if net_index < station.new_networks.len() {
                        let (net, _) = station.new_networks[net_index].clone();

                        if net.network_type == NetworkType::Eap {
                            sender.send(Event::ConfigureNewEapNetwork(net.name.clone()))?;
                            return Ok(());
                        }
                        tokio::spawn(async move {
                            let _ = net.connect(sender.clone()).await;
                        });
                    } else {
                        let net = station.new_hidden_networks
                            [net_index.saturating_sub(station.new_networks.len())]
                        .clone();

                        if net.network_type == NetworkType::Eap {
                            sender.send(Event::ConfigureNewEapNetwork(net.address.clone()))?;
                            return Ok(());
                        }
                        tokio::spawn({
                            let iwd_station =
                                station.session.stations().await.unwrap().pop().unwrap();
                            let ssid = net.address.clone();
                            async move {
                                if let Err(e) = iwd_station.connect_hidden_network(ssid).await {
                                    let _ = Notification::send(
                                        e.to_string(),
                                        notification::NotificationLevel::Error,
                                        &sender,
                                    );
                                }
                            }
                        });
                    }
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
                    FocusedBlock::ShareNetwork => {
                        if key_event.code == KeyCode::Esc {
                            station.share = None;
                            app.focused_block = FocusedBlock::KnownNetworks;
                        }
                    }
                    FocusedBlock::AddHiddenNetworkSsid => match key_event.code {
                        KeyCode::Enter => {
                            let ssid = station.hidden_ssid.value().trim().to_string();
                            if ssid.is_empty() {
                                let _ = Notification::send(
                                    "SSID cannot be empty".to_string(),
                                    NotificationLevel::Info,
                                    &sender,
                                );
                                return Ok(());
                            }

                            // Set the network name for auth
                            app.network_name_requiring_auth = Some(ssid.clone());

                            // Connect to the hidden network
                            let stations = station.session.stations().await?;
                            let iwd_station = stations
                                .into_iter()
                                .next()
                                .ok_or_else(|| anyhow!("No stations available"))?;
                            tokio::spawn(async move {
                                match iwd_station.connect_hidden_network(ssid.clone()).await {
                                    Ok(()) => {
                                        let _ = Notification::send(
                                            format!("Connected to hidden network: {}", ssid),
                                            NotificationLevel::Info,
                                            &sender,
                                        );
                                    }
                                    Err(e) => {
                                        let _ = Notification::send(
                                            e.to_string(),
                                            NotificationLevel::Error,
                                            &sender,
                                        );
                                    }
                                }
                            });
                            app.focused_block = FocusedBlock::NewNetworks;
                        }
                        KeyCode::Esc => {
                            app.focused_block = FocusedBlock::NewNetworks;
                        }
                        _ => {
                            station
                                .hidden_ssid
                                .handle_event(&crossterm::event::Event::Key(key_event));
                        }
                    },
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
                                        // Share
                                        KeyCode::Char(c)
                                            if c == config.station.known_network.share =>
                                        {
                                            if unsafe { libc::geteuid() } != 0 {
                                                let _ = Notification::send(
                                                    "impala must be run as root to share networks"
                                                        .to_string(),
                                                    notification::NotificationLevel::Info,
                                                    &sender,
                                                );
                                                return Ok(());
                                            }

                                            if let Some(net_index) =
                                                station.known_networks_state.selected()
                                            {
                                                if net_index > station.known_networks.len() - 1 {
                                                    let index = net_index.saturating_sub(
                                                        station.known_networks.len(),
                                                    );
                                                    let network =
                                                        &station.unavailable_known_networks[index];
                                                    if network.network_type == NetworkType::Psk
                                                        && let Ok(share) =
                                                            Share::new(network.name.clone())
                                                    {
                                                        station.share = Some(share);
                                                        app.focused_block =
                                                            FocusedBlock::ShareNetwork;
                                                    }
                                                } else {
                                                    let (network, _) =
                                                        &station.known_networks[net_index];
                                                    if network.network_type == NetworkType::Psk
                                                        && let Ok(share) =
                                                            Share::new(network.name.clone())
                                                    {
                                                        station.share = Some(share);
                                                        app.focused_block =
                                                            FocusedBlock::ShareNetwork;
                                                    }
                                                }
                                            }
                                        }
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
                                        KeyCode::Enter | KeyCode::Char(' ') => {
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
                                    // Show / Hide unavailable networks
                                    KeyCode::Char(c)
                                        if c == config.station.new_network.show_all =>
                                    {
                                        station.show_hidden_networks =
                                            !station.show_hidden_networks;
                                    }
                                    KeyCode::Char(c)
                                        if c == config.station.new_network.add_hidden =>
                                    {
                                        app.focused_block = FocusedBlock::AddHiddenNetworkSsid;
                                        station.hidden_ssid = Input::default();
                                    }
                                    KeyCode::Enter | KeyCode::Char(' ') => {
                                        toggle_connect(app, sender).await?
                                    }
                                    KeyCode::Char('j') | KeyCode::Down => {
                                        if !station.new_networks.is_empty() {
                                            let i = match station.new_networks_state.selected() {
                                                Some(i) => {
                                                    let limit = if station.show_hidden_networks {
                                                        station.new_networks.len()
                                                            + station.new_hidden_networks.len()
                                                            - 1
                                                    } else {
                                                        station.new_networks.len() - 1
                                                    };
                                                    if i < limit { i + 1 } else { i }
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
