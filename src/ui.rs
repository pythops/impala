use std::sync::atomic::Ordering;

use iwdrs::modes::Mode;
use ratatui::Frame;

use crate::app::{App, FocusedBlock};

pub fn render(app: &mut App, frame: &mut Frame) {
    if app.reset.enable {
        app.reset.render(frame);
    } else {
        if !app.device.is_powered {
            app.device
                .render(frame, app.focused_block, app.config.clone())
        } else {
            let device = app.device.clone();
            match app.device.mode {
                Mode::Station => {
                    if let Some(station) = &mut app.device.station {
                        station.render(frame, app.focused_block, &device, app.config.clone());
                    }
                }
                Mode::Ap => {
                    if let Some(ap) = &mut app.device.ap {
                        ap.render(frame, app.focused_block, &device, app.config.clone());
                    }
                }
            }
        };

        if app.focused_block == FocusedBlock::AdapterInfos {
            app.adapter.render(frame, app.device.address.clone());
        }

        // Auth Popup
        if app.agent.psk_required.load(Ordering::Relaxed) {
            app.focused_block = FocusedBlock::PskAuthKey;

            app.auth
                .psk
                .render(frame, app.network_name_requiring_auth.clone());
        }

        // Notifications
        for (index, notification) in app.notifications.iter().enumerate() {
            notification.render(index, frame);
        }
    }
}
