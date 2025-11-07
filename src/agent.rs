use async_channel::{Receiver, Sender};
use std::sync::{Arc, atomic::AtomicBool};
use tokio::sync::mpsc::UnboundedSender;

use iwdrs::error::agent::Canceled;
use iwdrs::{agent::Agent, network::Network};

use crate::event::Event;

#[derive(Debug, Clone)]
pub struct AuthAgent {
    pub tx_cancel: Sender<()>,
    pub rx_cancel: Receiver<()>,
    pub tx_passphrase: Sender<String>,
    pub rx_passphrase: Receiver<String>,
    pub tx_username_password: Sender<(String, String)>,
    pub rx_username_password: Receiver<(String, String)>,
    pub psk_required: Arc<AtomicBool>,
    pub private_key_passphrase_required: Arc<AtomicBool>,
    pub password_required: Arc<AtomicBool>,
    pub username_and_password_required: Arc<AtomicBool>,
    pub event_sender: UnboundedSender<Event>,
}

impl AuthAgent {
    pub fn new(sender: UnboundedSender<Event>) -> Self {
        let (tx_passphrase, rx_passphrase) = async_channel::unbounded();
        let (tx_username_password, rx_username_password) = async_channel::unbounded();
        let (tx_cancel, rx_cancel) = async_channel::unbounded();

        Self {
            tx_cancel,
            rx_cancel,
            tx_passphrase,
            rx_passphrase,
            tx_username_password,
            rx_username_password,
            psk_required: Arc::new(AtomicBool::new(false)),
            private_key_passphrase_required: Arc::new(AtomicBool::new(false)),
            password_required: Arc::new(AtomicBool::new(false)),
            username_and_password_required: Arc::new(AtomicBool::new(false)),
            event_sender: sender,
        }
    }
}

impl Agent for AuthAgent {
    async fn request_passphrase(&self, network: &Network) -> Result<String, Canceled> {
        self.psk_required
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let network_name = network.name().await.map_err(|_| Canceled())?;
        self.event_sender
            .send(Event::Auth(network_name))
            .map_err(|_| Canceled())?;

        tokio::select! {
        r = self.rx_passphrase.recv() =>  {
                match r {
                    Ok(key) => Ok(key),
                    Err(_) => Err(Canceled()),
                }
            }

        _ = self.rx_cancel.recv() => {
                    Err(Canceled())
            }

        }
    }

    async fn request_private_key_passphrase(
        &self,
        network: &Network,
    ) -> Result<String, iwdrs::error::agent::Canceled> {
        self.private_key_passphrase_required
            .store(true, std::sync::atomic::Ordering::Relaxed);

        let network_name = network.name().await.map_err(|_| Canceled())?;
        self.event_sender
            .send(Event::AuthReqKeyPassphrase(network_name))
            .map_err(|_| Canceled())?;

        tokio::select! {
        r = self.rx_passphrase.recv() =>  {
                match r {
                    Ok(key) => Ok(key),
                    Err(_) => Err(Canceled()),
                }
            }

        _ = self.rx_cancel.recv() => {
                    Err(Canceled())
            }

        }
    }

    async fn request_user_name_and_passphrase(
        &self,
        network: &Network,
    ) -> Result<(String, String), iwdrs::error::agent::Canceled> {
        self.username_and_password_required
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let network_name = network.name().await.map_err(|_| Canceled())?;
        self.event_sender
            .send(Event::AuthReqUsernameAndPassword(network_name))
            .map_err(|_| Canceled())?;

        tokio::select! {
        r = self.rx_username_password.recv() =>  {
                match r {
                    Ok((username, password)) => Ok((username, password)),
                    Err(_) => Err(Canceled()),
                }
            }

        _ = self.rx_cancel.recv() => {
                    Err(Canceled())
            }

        }
    }

    async fn request_user_password(
        &self,
        network: &Network,
        user_name: Option<&String>,
    ) -> Result<String, iwdrs::error::agent::Canceled> {
        self.password_required
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let network_name = network.name().await.map_err(|_| Canceled())?;
        self.event_sender
            .send(Event::AuthRequestPassword((
                network_name,
                user_name.cloned(),
            )))
            .map_err(|_| Canceled())?;

        tokio::select! {
        r = self.rx_passphrase.recv() =>  {
                match r {
                    Ok(password) => Ok(password),
                    Err(_) => Err(Canceled()),
                }
            }

        _ = self.rx_cancel.recv() => {
                    Err(Canceled())
            }

        }
    }
}
