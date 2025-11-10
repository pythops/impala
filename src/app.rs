use anyhow::anyhow;
use std::{
    error::Error,
    process::{self, exit},
    sync::Arc,
};
use tokio::sync::mpsc::UnboundedSender;

use iwdrs::{modes::Mode, session::Session};

use crate::{
    adapter::Adapter, agent::AuthAgent, config::Config, device::Device, event::Event,
    mode::station::auth::Auth, notification::Notification, reset::Reset,
};

pub type AppResult<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedBlock {
    Device,
    AccessPoint,
    KnownNetworks,
    NewNetworks,
    PskAuthKey,
    WpaEntrepriseAuth,
    AdapterInfos,
    AccessPointInput,
    AccessPointConnectedDevices,
    RequestKeyPasshphrase,
    RequestPassword,
    RequestUsernameAndPassword,
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub focused_block: FocusedBlock,
    pub notifications: Vec<Notification>,
    pub session: Arc<Session>,
    pub adapter: Adapter,
    pub device: Device,
    pub agent: AuthAgent,
    pub reset: Reset,
    pub config: Arc<Config>,
    pub auth: Auth,
    pub network_name_requiring_auth: Option<String>,
}

impl App {
    pub async fn new(
        sender: UnboundedSender<Event>,
        config: Arc<Config>,
        mode: Mode,
    ) -> AppResult<Self> {
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => {
                    eprintln!("Can not access the iwd service {e}");
                    exit(1);
                }
            }
        };

        let adapter = match Adapter::new(session.clone(), config.clone()).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                eprintln!("Make sure iwd daemon is up and running");
                process::exit(1);
            }
        };

        let device = Device::new(session.clone()).await?;
        device.set_mode(mode).await?;

        let agent = AuthAgent::new(sender);
        let _ = session.register_agent(agent.clone()).await?;

        let focused_block = if device.is_powered {
            match device.mode {
                Mode::Station => FocusedBlock::KnownNetworks,
                _ => FocusedBlock::AccessPoint,
            }
        } else {
            FocusedBlock::Device
        };

        let reset = Reset::new(mode);

        Ok(Self {
            running: true,
            focused_block,
            notifications: Vec::new(),
            session,
            adapter,
            agent,
            reset,
            device,
            config,
            auth: Auth::default(),
            network_name_requiring_auth: None,
        })
    }

    pub async fn reset(mode: Mode) -> AppResult<()> {
        let session = {
            match iwdrs::session::Session::new().await {
                Ok(session) => Arc::new(session),
                Err(e) => return Err(anyhow!("Can not access the iwd service: {}", e).into()),
            }
        };

        let device = match Device::new(session.clone()).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{e}");
                eprintln!("Make sure iwd daemon is up and running");
                process::exit(1);
            }
        };

        device.set_mode(mode).await?;
        Ok(())
    }

    pub async fn tick(&mut self, sender: UnboundedSender<Event>) -> AppResult<()> {
        self.notifications.retain(|n| n.ttl > 0);
        self.notifications.iter_mut().for_each(|n| n.ttl -= 1);

        self.device.refresh(sender).await.unwrap();
        self.adapter.refresh().await.unwrap();

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
