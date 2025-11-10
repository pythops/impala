use anyhow::Result;
use impala::{
    app::App,
    cli,
    config::Config,
    event::{Event, EventHandler},
    handler::handle_key_events,
    notification::{Notification, NotificationLevel},
    rfkill,
    tui::Tui,
};
use iwdrs::modes::Mode;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
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

    let mut app = App::new(tui.events.sender.clone(), config.clone(), mode).await?;

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
                if App::reset(mode).await.is_err() {
                    tui.exit()?;
                }
                app = App::new(tui.events.sender.clone(), config.clone(), mode).await?;
            }

            Event::Auth(network_name) => {
                app.network_name_requiring_auth = Some(network_name);
            }

            Event::EapNeworkConfigured(network_name) => {
                app.auth.reset();
                app.focused_block = impala::app::FocusedBlock::KnownNetworks;
                Notification::send(
                    format!("Network {} configured", network_name),
                    NotificationLevel::Info,
                    &tui.events.sender.clone(),
                )?;
            }

            Event::UsernameAndPasswordSubmit => {
                if let Some(req) = &mut app.auth.request_username_and_password {
                    req.submit(&app.agent).await?;
                    app.focused_block = impala::app::FocusedBlock::KnownNetworks;
                    app.auth.request_username_and_password = None;
                }
            }

            Event::ConfigureNewEapNetwork(network_name) => {
                app.auth.init_eap(network_name);
                app.focused_block = impala::app::FocusedBlock::WpaEntrepriseAuth;
            }

            Event::AuthReqKeyPassphrase(network_name) => {
                app.auth.init_request_key_passphrase(network_name.clone());
                app.focused_block = impala::app::FocusedBlock::RequestKeyPasshphrase;
            }

            Event::AuthRequestPassword((network_name, user_name)) => {
                app.auth.init_request_password(network_name, user_name);
                app.focused_block = impala::app::FocusedBlock::RequestPassword
            }

            Event::AuthReqUsernameAndPassword(network_name) => {
                app.auth.init_request_username_and_password(network_name);
                app.focused_block = impala::app::FocusedBlock::RequestUsernameAndPassword
            }
            _ => {}
        }
    }

    tui.exit()?;
    Ok(())
}
