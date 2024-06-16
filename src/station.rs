use anyhow::Result;
use futures::future::join_all;
use iwdrs::station::Station as iwdStation;
use ratatui::widgets::TableState;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    app::AppResult,
    event::Event,
    network::Network,
    notification::{Notification, NotificationLevel},
};

#[derive(Debug, Clone)]
pub struct Station {
    pub s: iwdStation,
    pub state: String,
    pub is_scanning: bool,
    pub connected_network: Option<Network>,
    pub new_networks: Vec<(Network, i16)>,
    pub known_networks: Vec<(Network, i16)>,
    pub known_networks_state: TableState,
    pub new_networks_state: TableState,
}

impl Station {
    pub async fn new(s: iwdStation) -> Result<Self> {
        let state = s.state().await?;
        let connected_network = {
            if let Some(n) = s.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network)
            } else {
                None
            }
        };

        let is_scanning = s.is_scanning().await?;
        let discovered_networks = s.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        let mut new_networks_state = TableState::default();
        if new_networks.is_empty() {
            new_networks_state.select(None);
        } else {
            new_networks_state.select(Some(0));
        }

        let mut known_networks_state = TableState::default();

        if known_networks.is_empty() {
            known_networks_state.select(None);
        } else {
            known_networks_state.select(Some(0));
        }

        Ok(Self {
            s,
            state,
            is_scanning,
            connected_network,
            new_networks,
            known_networks,
            known_networks_state,
            new_networks_state,
        })
    }
    pub async fn refresh(&mut self) -> Result<()> {
        let state = self.s.state().await?;
        let is_scanning = self.s.is_scanning().await?;
        let connected_network = {
            if let Some(n) = self.s.connected_network().await? {
                let network = Network::new(n.clone()).await?;
                Some(network.to_owned())
            } else {
                None
            }
        };
        let discovered_networks = self.s.discovered_networks().await?;
        let networks = {
            let collected_futures = discovered_networks
                .iter()
                .map(|(n, signal)| async {
                    match Network::new(n.clone()).await {
                        Ok(network) => Ok((network, signal.to_owned())),
                        Err(e) => Err(e),
                    }
                })
                .collect::<Vec<_>>();
            let results = join_all(collected_futures).await;
            results
                .into_iter()
                .filter_map(Result::ok)
                .collect::<Vec<(Network, i16)>>()
        };

        let new_networks: Vec<(Network, i16)> = networks
            .clone()
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_none())
            .collect();

        let known_networks: Vec<(Network, i16)> = networks
            .into_iter()
            .filter(|(net, _signal)| net.known_network.is_some())
            .collect();

        self.state = state;
        self.is_scanning = is_scanning;

        if self.new_networks.len() != new_networks.len() {
            let mut new_networks_state = TableState::default();
            if new_networks.is_empty() {
                new_networks_state.select(None);
            } else {
                new_networks_state.select(Some(0));
            }

            self.new_networks_state = new_networks_state;
            self.new_networks = new_networks;
        } else {
            self.new_networks.iter_mut().for_each(|(net, signal)| {
                let n = new_networks
                    .iter()
                    .find(|(refreshed_net, _signal)| refreshed_net.name == net.name);

                if let Some((_, refreshed_signal)) = n {
                    *signal = *refreshed_signal;
                }
            })
        }

        if self.known_networks.len() != known_networks.len() {
            let mut known_networks_state = TableState::default();
            if known_networks.is_empty() {
                known_networks_state.select(None);
            } else {
                known_networks_state.select(Some(0));
            }
            self.known_networks_state = known_networks_state;
            self.known_networks = known_networks;
        } else {
            self.known_networks.iter_mut().for_each(|(net, signal)| {
                let n = known_networks
                    .iter()
                    .find(|(refreshed_net, _signal)| refreshed_net.name == net.name);

                if let Some((refreshed_net, refreshed_signal)) = n {
                    net.known_network.as_mut().unwrap().is_autoconnect =
                        refreshed_net.known_network.as_ref().unwrap().is_autoconnect;
                    *signal = *refreshed_signal;
                }
            })
        }

        self.connected_network = connected_network;

        Ok(())
    }

    pub async fn scan(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.s.scan().await {
            Ok(_) => Notification::send(
                "Start Scanning".to_string(),
                NotificationLevel::Info,
                sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?,
        }

        Ok(())
    }

    pub async fn disconnect(&self, sender: UnboundedSender<Event>) -> AppResult<()> {
        match self.s.disconnect().await {
            Ok(_) => Notification::send(
                format!(
                    "Disconnected from {}",
                    self.connected_network.as_ref().unwrap().name
                ),
                NotificationLevel::Info,
                sender,
            )?,
            Err(e) => Notification::send(e.to_string(), NotificationLevel::Error, sender.clone())?,
        }
        Ok(())
    }
}
