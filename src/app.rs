use anyhow::anyhow;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};
use std::{
    error,
    process::exit,
    sync::{atomic::AtomicBool, Arc},
};
use tui_input::Input;

use tracing::error;

use async_channel::{Receiver, Sender};
use futures::FutureExt;
use iwdrs::{agent::Agent, session::Session};

use crate::{adapter::Adapter, config::Config, help::Help, notification::Notification};

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Device,
    Station,
    AccessPoint,
    KnownNetworks,
    NewNetworks,
    Help,
    AuthKey,
    AdapterInfos,
    AccessPointInput,
    AccessPointConnectedDevices,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CharSet {
    Unicode,
    Ascii
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub focused_block: FocusedBlock,
    pub help: Help,
    pub color_mode: ColorMode,
    pub notifications: Vec<Notification>,
    pub session: Arc<Session>,
    pub adapter: Adapter,
    pub agent_manager: iwdrs::agent::AgentManager,
    pub authentication_required: Arc<AtomicBool>,
    pub passkey_sender: Sender<String>,
    pub cancel_signal_sender: Sender<()>,
    pub passkey_input: Input,
    pub mode: String,
    pub selected_mode: String,
    pub current_mode: String,
    pub reset_mode: bool,
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx_key: Receiver<String>,
    rx_cancel: Receiver<()>,
) -> Result<String, Box<dyn std::error::Error>> {
    authentication_required.store(true, std::sync::atomic::Ordering::Relaxed);

    tokio::select! {
    r = rx_key.recv() =>  {
            match r {
                Ok(key) => Ok(key),
                Err(_) => Err(anyhow!("Failed to receive the key").into()),
            }
        }

    r = rx_cancel.recv() => {
            match r {
                Ok(_) => {
                        Err(anyhow!("Operation Canceled").into())},
                Err(_) => Err(anyhow!("Failed to receive cancel signal").into()),
            }

        }

    }
}

impl App {
    pub async fn new(help: Help, config: Arc<Config>, mode: String) -> AppResult<Self> {
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => {
                    error!("Can not access the iwd service");
                    error!("{}", e.to_string());
                    exit(1);
                }
            }
        };

        // Start scanning if enabled
        if config.station.auto_scan {
            let iwd_station = session.station().unwrap();
            match iwd_station.scan().await {
                Ok(_) => (),
                Err(e) => {
                    error!("Failed to start scan");
                    error!("{}", e.to_string());
                }
            }
        }

        let adapter = Adapter::new(session.clone()).await.unwrap();

        let current_mode = adapter.device.mode.clone();

        let selected_mode = String::from("station");

        let (passkey_sender, passkey_receiver) = async_channel::unbounded();
        let (cancel_signal_sender, cancel_signal_receiver) = async_channel::unbounded();

        let authentication_required = Arc::new(AtomicBool::new(false));
        let authentication_required_caller = authentication_required.clone();

        let agent = Agent {
            request_passphrase_fn: Box::new(move || {
                {
                    let auth_clone = authentication_required_caller.clone();
                    request_confirmation(
                        auth_clone,
                        passkey_receiver.clone(),
                        cancel_signal_receiver.clone(),
                    )
                }
                .boxed()
            }),
        };

        let agent_manager = session.register_agent(agent).await?;

        let color_mode = match terminal_light::luma() {
            Ok(luma) if luma > 0.6 => ColorMode::Light,
            Ok(_) => ColorMode::Dark,
            Err(_) => ColorMode::Dark,
        };

        Ok(Self {
            running: true,
            focused_block: FocusedBlock::Device,
            help,
            color_mode,
            notifications: Vec::new(),
            session,
            adapter,
            agent_manager,
            authentication_required: authentication_required.clone(),
            passkey_sender,
            cancel_signal_sender,
            passkey_input: Input::default(),
            mode,
            selected_mode,
            current_mode,
            reset_mode: false,
        })
    }

    pub async fn reset(mode: String) -> AppResult<()> {
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => {
                    error!("Can not access the iwd service");
                    error!("{}", e.to_string());
                    exit(1);
                }
            }
        };

        let adapter = Adapter::new(session.clone()).await.unwrap();
        adapter.device.set_mode(mode).await?;
        Ok(())
    }

    pub fn render(&self, frame: &mut Frame) {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Min(10),
                    Constraint::Percentage(45),
                ]
                .as_ref(),
            )
            .split(frame.size());

        let area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((frame.size().width - 50) / 2),
                    Constraint::Min(50),
                    Constraint::Length((frame.size().width - 50) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1];

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(area);

        let (message_area, station_choice_area, ap_choice_area, help_area) =
            (chunks[1], chunks[2], chunks[3], chunks[6]);

        let station_choice_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(station_choice_area)[1];

        let ap_choice_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(ap_choice_area)[1];

        let message_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(2),
                    Constraint::Fill(1),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(message_area)[1];

        let (ap_text, station_text) = match self.selected_mode.as_str() {
            "ap" => match self.current_mode.as_str() {
                "ap" => (
                    Text::from("  Access Point (current)"),
                    Text::from("   Station"),
                ),
                "station" => (
                    Text::from("  Access Point"),
                    Text::from("   Station (current)"),
                ),
                _ => (Text::from("  Access Point"), Text::from("   Station")),
            },
            "station" => match self.current_mode.as_str() {
                "ap" => (
                    Text::from("   Access Point (current)"),
                    Text::from("  Station"),
                ),
                "station" => (
                    Text::from("   Access Point"),
                    Text::from("  Station (current)"),
                ),
                _ => (Text::from("  Access Point"), Text::from("   Station")),
            },
            _ => panic!("unknwon mode"),
        };

        let message = Paragraph::new("Please select a mode:")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::uniform(1)));

        let station_choice = Paragraph::new(station_text)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::horizontal(10)));

        let ap_choice = Paragraph::new(ap_text)
            .style(Style::default().fg(Color::White))
            .block(Block::new().padding(Padding::horizontal(10)));

        let help = Paragraph::new(
            Text::from(" Scroll down: j | Scroll up: k | Enter: Confirm ")
                .style(Style::default().blue()),
        )
        .alignment(Alignment::Center)
        .style(Style::default())
        .block(Block::new().padding(Padding::horizontal(1)));

        frame.render_widget(Clear, area);

        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().green())
                .border_style(Style::default().fg(Color::Green)),
            area,
        );
        frame.render_widget(message, message_area);
        frame.render_widget(ap_choice, ap_choice_area);
        frame.render_widget(station_choice, station_choice_area);
        frame.render_widget(help, help_area);
    }

    pub async fn send_passkey(&mut self) -> AppResult<()> {
        let passkey: String = self.passkey_input.value().into();
        self.passkey_sender.send(passkey).await?;
        self.authentication_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passkey_input.reset();
        Ok(())
    }

    pub async fn cancel_auth(&mut self) -> AppResult<()> {
        self.cancel_signal_sender.send(()).await?;
        self.authentication_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passkey_input.reset();
        Ok(())
    }

    pub async fn tick(&mut self) -> AppResult<()> {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        self.adapter.refresh().await?;

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
