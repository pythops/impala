use clap::{crate_version, Command};
use impala::app::{App, AppResult};
use impala::config::Config;
use impala::event::{Event, EventHandler};
use impala::handler::handle_key_events;
use impala::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    Command::new("impala")
        .version(crate_version!())
        .get_matches();

    let config = Arc::new(Config::new());

    let mut app = App::new(config.clone()).await?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                handle_key_events(
                    key_event,
                    &mut app,
                    tui.events.sender.clone(),
                    config.clone(),
                )
                .await?
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
