use std::sync::Arc;

use anyhow::{Context, Result};
use iwdrs::{device::Device as iwdDevice, session::Session};

use tracing::error;

use crate::station::Station;

#[derive(Debug, Clone)]
pub struct Device {
    session: Arc<Session>,
    pub device: iwdDevice,
    pub name: String,
    pub address: String,
    pub mode: String,
    pub is_powered: bool,
    pub station: Option<Station>,
}

impl Device {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let device = session.device().context("Not device found")?;

        let name = device.name().await?;
        let address = device.address().await?;
        let mode = device.get_mode().await?;
        let is_powered = device.is_powered().await?;

        let station = match session.station() {
            Some(iwdrs_station) => match Station::new(iwdrs_station).await {
                Ok(v) => Some(v),
                Err(e) => {
                    error!("{}", e.to_string());
                    None
                }
            },
            None => None,
        };

        Ok(Self {
            session,
            device,
            name,
            address,
            mode,
            is_powered,
            station,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        let mode = self.device.get_mode().await?;
        let is_powered = self.device.is_powered().await?;
        self.mode = mode;

        if self.station.is_none() {
            self.station = match self.session.station() {
                Some(iwdrs_station) => match Station::new(iwdrs_station).await {
                    Ok(v) => Some(v),
                    Err(e) => {
                        error!("{}", e.to_string());
                        None
                    }
                },
                None => None,
            };
        }

        match self.mode.as_str() {
            "station" => {
                if self.station.is_some() {
                    self.station.as_mut().unwrap().refresh().await?;
                }
            }
            "access_point" => {}
            _ => {}
        }

        self.is_powered = is_powered;
        Ok(())
    }
}
