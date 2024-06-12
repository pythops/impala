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
    KnownNetworks,
    NewNetworks,
    Help,
    AuthKey,
    AdapterInfos,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorMode {
    Dark,
    Light,
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
    pub passkey_input: Input,
}

pub async fn request_confirmation(
    authentication_required: Arc<AtomicBool>,
    rx: Receiver<String>,
) -> Result<String, Box<dyn std::error::Error>> {
    authentication_required.store(true, std::sync::atomic::Ordering::Relaxed);
    match rx.recv().await {
        Ok(passkey) => Ok(passkey),
        Err(e) => Err(e.into()),
    }
}

impl App {
    pub async fn new(config: Arc<Config>) -> AppResult<Self> {
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

        let (s, r) = async_channel::unbounded();

        let authentication_required = Arc::new(AtomicBool::new(false));
        let authentication_required_caller = authentication_required.clone();

        let agent = Agent {
            request_passphrase_fn: Box::new(move || {
                {
                    let auth_clone = authentication_required_caller.clone();
                    request_confirmation(auth_clone, r.clone())
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
            help: Help::new(config),
            color_mode,
            notifications: Vec::new(),
            session,
            adapter,
            agent_manager,
            authentication_required: authentication_required.clone(),
            passkey_sender: s,
            passkey_input: Input::default(),
        })
    }

    pub async fn send_passkey(&mut self) -> AppResult<()> {
        let passkey: String = self.passkey_input.value().into();
        self.passkey_sender.send(passkey).await?;
        self.authentication_required
            .store(false, std::sync::atomic::Ordering::Relaxed);
        self.passkey_input.reset();
        Ok(())
    }

    pub async fn tick(&mut self) -> AppResult<()> {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        self.adapter.device.refresh().await?;

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
