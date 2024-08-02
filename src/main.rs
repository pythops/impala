use impala::app::{App, AppResult};
use impala::cli;
use impala::config::{ColorMode, Config};
use impala::event::{Event, EventHandler};
use impala::handler::handle_key_events;
use impala::help::Help;
use impala::tracing::Tracing;
use impala::tui::{Tui, generate_palette};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> AppResult<()> {
    Tracing::init().unwrap();

    let args = cli::cli().get_matches();

    let config = Arc::new({
        let mut config = Config::new();

        // Automatically detect color mode
        if config.color_mode == ColorMode::Auto {
            config.color_mode = match terminal_light::luma() {
                Ok(luma) if luma > 0.6 => ColorMode::Light,
                Ok(_) => ColorMode::Dark,
                Err(_) => ColorMode::Dark,
            };
        }

        config
    });

    let palette = generate_palette(config.color_mode == ColorMode::Light,
        config.monochrome);

    let mode = args.get_one::<String>("mode").cloned();

    let help = Help::new(config.clone(), &palette);

    let mode = mode.unwrap_or_else(|| config.mode.clone());

    App::reset(mode.clone()).await?;
    let mut app = App::new(help.clone(), config.clone(), mode).await?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(3000);
    let mut tui = Tui::new(terminal, events, palette);
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
                app = App::new(help.clone(), config.clone(), mode).await?;
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
