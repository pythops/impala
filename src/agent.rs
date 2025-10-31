use async_channel::{Receiver, Sender};
use std::sync::{Arc, atomic::AtomicBool};

use iwdrs::error::agent::Canceled;
use iwdrs::{agent::Agent, network::Network};

#[derive(Debug, Clone)]
pub struct AuthAgent {
    pub tx_cancel: Sender<()>,
    pub rx_cancel: Receiver<()>,
    pub tx_passphrase: Sender<String>,
    pub rx_passphrase: Receiver<String>,
    pub required: Arc<AtomicBool>,
}

impl AuthAgent {
    pub fn new() -> Self {
        let (tx_passphrase, rx_passphrase) = async_channel::unbounded();
        let (tx_cancel, rx_cancel) = async_channel::unbounded();

        Self {
            tx_cancel,
            rx_cancel,
            tx_passphrase,
            rx_passphrase,
            required: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for AuthAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Agent for AuthAgent {
    async fn request_passphrase(&self, _: &Network) -> Result<String, Canceled> {
        self.required
            .store(true, std::sync::atomic::Ordering::Relaxed);

        //TODO: add sender for notification

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

    fn request_private_key_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<String, iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(Canceled()))
    }

    fn request_user_name_and_passphrase(
        &self,
        _network: &Network,
    ) -> impl Future<Output = Result<(String, String), iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(Canceled()))
    }

    fn request_user_password(
        &self,
        _network: &Network,
        _user_name: Option<&String>,
    ) -> impl Future<Output = Result<(String, String), iwdrs::error::agent::Canceled>> + Send {
        std::future::ready(Err(Canceled()))
    }
}
