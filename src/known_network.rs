use iwdrs::known_netowk::KnownNetwork as iwdKnownNetwork;

use anyhow::Result;
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
    pub netowrk_type: String,
    pub is_autoconnect: bool,
    pub is_hidden: bool,
    pub last_connected: String,
}

impl KnownNetwork {
    pub async fn new(n: iwdKnownNetwork) -> Result<Self> {
        let name = n.name().await?;
        let netowrk_type = n.network_type().await?;
        let is_autoconnect = n.get_autoconnect().await?;
        let is_hidden = n.hidden().await?;
        let last_connected = n.last_connected_time().await?;

        Ok(Self {
            n,
            name,
            netowrk_type,
            is_autoconnect,
            is_hidden,
            last_connected,
        })
    }

    pub async fn forget(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        if let Err(e) = self.n.forget().await {
            Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?;
            return Ok(());
        }

        Notification::send(
            "Network Removed".to_string(),
            NotificationLevel::Info,
            sender,
        )?;
        Ok(())
    }
}
