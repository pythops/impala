use anyhow::Result;
use iwdrs::{
    error::{IWDError, network::ConnectError},
    network::{Network as iwdNetwork, NetworkType},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    event::Event,
    mode::station::known_network::KnownNetwork,
    notification::{Notification, NotificationLevel},
};

#[derive(Debug, Clone)]
pub struct Network {
    pub n: iwdNetwork,
    pub name: String,
    pub network_type: NetworkType,
    pub is_connected: bool,
    pub known_network: Option<KnownNetwork>,
}

impl Network {
    pub async fn new(n: iwdNetwork) -> Result<Self> {
        let name = n.name().await?;
        let network_type = n.network_type().await?;
        let is_connected = n.connected().await?;
        let known_network = {
            match n.known_network().await {
                Ok(v) => match v {
                    Some(net) => Some(KnownNetwork::new(net).await.unwrap()),
                    None => None,
                },
                Err(_) => None,
            }
        };

        Ok(Self {
            n,
            name,
            network_type,
            is_connected,
            known_network,
        })
    }

    pub async fn connect(&self, sender: UnboundedSender<Event>) -> Result<()> {
        match self.n.connect().await {
            Ok(()) => Notification::send(
                format!("Connected to {}", self.name),
                NotificationLevel::Info,
                &sender,
            )?,
            Err(e) => match e {
                IWDError::OperationError(e) => match e {
                    ConnectError::Aborted => {
                        Notification::send(e.to_string(), NotificationLevel::Info, &sender)?;
                    }
                    _ => {
                        Notification::send(e.to_string(), NotificationLevel::Error, &sender)?;
                    }
                },
                _ => {
                    Notification::send(e.to_string(), NotificationLevel::Error, &sender)?;
                }
            },
        }
        Ok(())
    }
}
