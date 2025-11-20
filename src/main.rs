use anyhow::Result;
use anyhow::anyhow;
use env_logger::Target;
use impala::{
    app::App,
    cli,
    config::Config,
    event::{Event, EventHandler},
    handler::{handle_key_events, toggle_connect},
    notification::{Notification, NotificationLevel},
    rfkill,
    tui::Tui,
};
use iwdrs::modes::Mode;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::sync::Arc;
use std::{io, process::exit};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .format_timestamp(None)
        .target(Target::Stderr)
        .init();

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

    let mut app = match App::new(tui.events.sender.clone(), config.clone(), mode).await {
        Ok(app) => app,
        Err(e) => {
            tui.exit()?;

            if e.to_string()
                .contains("org.freedesktop.DBus.Error.AccessDenied")
            {
                eprintln!("Permission Denied");
            } else {
                eprintln!("{}", e);
            }
            exit(1);
        }
    };

    let mut exit_error_message = None;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next().await? {
            Event::Tick => {
                if let Err(e) = app.tick(tui.events.sender.clone()).await {
                    exit_error_message = Some(e);
                    break;
                }
            }
            Event::Key(key_event) => {
                if let Err(e) = handle_key_events(
                    key_event,
                    &mut app,
                    tui.events.sender.clone(),
                    config.clone(),
                )
                .await
                {
                    exit_error_message = Some(e);
                    break;
                }
            }
            Event::Notification(notification) => {
                app.notifications.push(notification);
            }

            Event::Reset(mode) => {
                if let Err(e) = App::reset(mode).await {
                    exit_error_message = Some(e);
                    break;
                };

                match App::new(tui.events.sender.clone(), config.clone(), mode).await {
                    Ok(v) => app = v,
                    Err(e) => {
                        if e.to_string()
                            .contains("org.freedesktop.DBus.Error.AccessDenied")
                        {
                            exit_error_message = Some(anyhow!("Permission Denied"));
                        } else {
                            exit_error_message = Some(e);
                        }
                        break;
                    }
                };
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

                if let Some(station) = &mut app.device.station
                    && let Some(index) = station
                        .known_networks
                        .iter()
                        .position(|(net, _)| net.name == network_name)
                {
                    station.known_networks_state.select(Some(index));
                    if let Err(e) = toggle_connect(&mut app, tui.events.sender.clone()).await {
                        exit_error_message = Some(e);
                        break;
                    }
                }
            }

            Event::UsernameAndPasswordSubmit => {
                if let Some(req) = &mut app.auth.request_username_and_password {
                    if let Err(e) = req.submit(&app.agent).await {
                        exit_error_message = Some(e);
                        break;
                    }
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

    if let Some(error) = exit_error_message {
        eprintln!("{}", error);
        exit(1);
    }

    Ok(())
}
