use chrono::{DateTime, FixedOffset};

use iwdrs::{known_network::KnownNetwork as iwdKnownNetwork, network::NetworkType};

use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::AppResult,
    event::Event,
    notification::{Notification, NotificationLevel},
};

#[derive(Debug, Clone)]
pub struct KnownNetwork {
    pub n: iwdKnownNetwork,
    pub name: String,
    pub network_type: NetworkType,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
    pub last_connected: Option<DateTime<FixedOffset>>,
}

impl KnownNetwork {
    pub async fn new(n: iwdKnownNetwork) -> AppResult<Self> {
        let name = n.name().await?;
        let network_type = n.network_type().await?;
        let is_autoconnect = n.get_autoconnect().await?;
        let is_hidden = n.hidden().await?;
        let last_connected = match n.last_connected_time().await {
            Ok(v) => DateTime::parse_from_rfc3339(&v).ok(),
            Err(_) => None,
        };

        Ok(Self {
            n,
            name,
            network_type,
            is_autoconnect,
            is_hidden,
            last_connected,
        })
    }

    pub async fn forget(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        if let Err(e) = self.n.forget().await {
            Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?;
            return Ok(());
        }

        Notification::send(
            "Network Removed".to_string(),
            NotificationLevel::Info,
            &sender,
        )?;
        Ok(())
    }

    pub async fn toggle_autoconnect(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        if self.is_autoconnect {
            match self.n.set_autoconnect(false).await {
                Ok(()) => {
                    Notification::send(
                        format!("Disable Autoconnect for: {}", self.name),
                        NotificationLevel::Info,
                        &sender.clone(),
                    )?;
                }
                Err(e) => {
                    Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?;
                }
            }
        } else {
            match self.n.set_autoconnect(true).await {
                Ok(()) => {
                    Notification::send(
                        format!("Enable Autoconnect for: {}", self.name),
                        NotificationLevel::Info,
                        &sender.clone(),
                    )?;
                }
                Err(e) => {
                    Notification::send(e.to_string(), NotificationLevel::Error, &sender.clone())?;
                }
            }
        }
        Ok(())
    }
}
