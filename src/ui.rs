use std::sync::atomic::Ordering;

use ratatui::Frame;

use crate::app::{App, FocusedBlock};

use crate::auth::Auth;

pub fn render(app: &mut App, frame: &mut Frame) {
    // App
    app.adapter.render(frame, app.color_mode, app.focused_block);

    if app.focused_block == FocusedBlock::AdapterInfos {
        app.adapter.render_adapter(frame, app.color_mode);
    }

    // Popup
    if app.authentication_required.load(Ordering::Relaxed) {
        app.focused_block = FocusedBlock::AuthKey;
        Auth.render(frame, app.passkey_input.value());
    }

    // Help
    if let FocusedBlock::Help = app.focused_block {
        app.help.render(frame, app.color_mode);
    }

    // Notifications
    for (index, notification) in app.notifications.iter().enumerate() {
        notification.render(index, frame);
    }
}
