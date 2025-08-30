use impala::app::{App, AppResult};
use impala::config::Config;
use impala::event::{Event, EventHandler};
use impala::handler::handle_key_events;
use impala::tui::Tui;
use impala::{cli, rfkill};
use iwdrs::modes::Mode;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = cli::cli().get_matches();

    rfkill::check()?;

    let config = Arc::new(Config::new());

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(2_000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    let mode = args.get_one::<String>("mode").cloned();
    let mode = mode.unwrap_or_else(|| config.mode.clone());

    let mode = Mode::try_from(mode.as_str())?;

    if App::reset(mode.clone(), tui.events.sender.clone(), config.clone())
        .await
        .is_err()
    {
        tui.exit()?;
    }

    let mut app = App::new(config.clone(), mode, tui.events.sender.clone()).await?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => app.tick(tui.events.sender.clone()).await?,
            Event::Key(key_event) => {
                handle_key_events(
                    key_event,
                    &mut app,
                    tui.events.sender.clone(),
                    config.clone(),
                )
                .await?;
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
            Event::Reset(mode) => {
                if App::reset(mode.clone(), tui.events.sender.clone(), config.clone())
                    .await
                    .is_err()
                {
                    tui.exit()?;
                }
                app = App::new(config.clone(), mode, tui.events.sender.clone()).await?;
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
