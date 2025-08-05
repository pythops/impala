mod ap;
mod station;

use std::sync::Arc;

use crate::app::{App, AppResult};
use crate::config::Config;
use crate::event::Event;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use iwdrs::modes::Mode;
use tokio::sync::mpsc::UnboundedSender;

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

    if let Mode::Station = app.adapter.device.mode {
        station::handle_station_key_events(key_event, app, sender.clone(), config.clone()).await
    } else {
        ap::handle_ap_key_events(key_event, app, sender.clone(), config.clone()).await
    }
}
