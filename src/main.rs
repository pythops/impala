use impala::app::{App, AppResult};
use impala::cli;
use impala::config::Config;
use impala::event::{Event, EventHandler};
use impala::handler::handle_key_events;
use impala::help::Help;
use impala::tracing::Tracing;
use impala::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    Tracing::init().unwrap();

    let args = cli::cli().get_matches();

    let config = Arc::new(Config::new());

    let mode = args.get_one::<String>("mode").cloned();

    let help = Help::new(config.clone());

    let mode = mode.unwrap_or_else(|| config.mode.clone());

    App::reset(mode.clone()).await?;
    let mut app = App::new(help.clone(), mode).await?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(50000);
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
            Event::Reset(mode) => {
                App::reset(mode.clone()).await?;
                app = App::new(help.clone(), mode).await?;
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
