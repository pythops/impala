use impala::app::{App, AppResult};
use impala::cli;
use impala::config::Config;
use impala::event::{Event, EventHandler};
use impala::handler::handle_key_events;
use impala::help::Help;
use impala::tui::Tui;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    if unsafe { libc::geteuid() } != 0 {
        eprintln!("This program must be run as root");
        std::process::exit(1);
    }

    let args = cli::cli().get_matches();

    let config = Arc::new(Config::new());

    let mode = args.get_one::<String>("mode").cloned();

    let help = Help::new(config.clone());

    let mode = mode.unwrap_or_else(|| config.mode.clone());

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(500);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    App::reset(mode.clone(), tui.events.sender.clone()).await?;
    let mut app = App::new(help.clone(), mode, tui.events.sender.clone()).await?;

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
                .await?
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }
            Event::Reset(mode) => {
                App::reset(mode.clone(), tui.events.sender.clone()).await?;
                app = App::new(help.clone(), mode, tui.events.sender.clone()).await?;
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
