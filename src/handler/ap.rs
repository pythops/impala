use std::sync::Arc;

use crate::access_point::APFocusedSection;
use crate::app::{App, AppResult, FocusedBlock};
use crate::config::Config;
use crate::event::Event;
use crate::notification::Notification;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use tui_input::backend::crossterm::EventHandler;

pub async fn handle_ap_key_events(
    key_event: KeyEvent,
    app: &mut App,
    sender: UnboundedSender<Event>,
    config: Arc<Config>,
) -> AppResult<()> {
    let access_point = &mut app.adapter.device.access_point;
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
            access_point.as_mut().unwrap().scan(sender).await?;
        }
        (KeyCode::Tab, FocusedBlock::Device) => app.focused_block = FocusedBlock::AccessPoint,
        (KeyCode::Tab, FocusedBlock::AccessPoint) => {
            if let Some(ap) = access_point.as_ref() {
                if ap.connected_devices.is_empty() {
                    app.focused_block = FocusedBlock::Device;
                } else {
                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                }
            }
        }
        (KeyCode::Tab, FocusedBlock::AccessPointConnectedDevices) => {
            app.focused_block = FocusedBlock::Device;
        }
        (KeyCode::BackTab, FocusedBlock::Device) => {
            if let Some(ap) = access_point.as_ref() {
                if ap.connected_devices.is_empty() {
                    app.focused_block = FocusedBlock::AccessPoint;
                } else {
                    app.focused_block = FocusedBlock::AccessPointConnectedDevices;
                }
            }
        }
        (KeyCode::BackTab, FocusedBlock::AccessPoint) => {
            app.focused_block = FocusedBlock::Device;
        }
        (KeyCode::BackTab, FocusedBlock::AccessPointConnectedDevices) => {
            app.focused_block = FocusedBlock::AccessPoint;
        }
        (KeyCode::Char(c), _) if c == config.ap.start => {
            if let Some(ap) = &access_point {
                // Start AP
                ap.ap_start
                    .store(true, std::sync::atomic::Ordering::Relaxed);
            }
        }
        (KeyCode::Char(c), _) if c == config.ap.stop => {
            if let Some(ap) = access_point {
                ap.stop(sender).await?;
                ap.connected_devices = Vec::new();
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
