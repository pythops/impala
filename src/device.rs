use std::sync::Arc;

use anyhow::{Context, Result};
use iwdrs::{device::Device as iwdDevice, session::Session};

use tracing::error;

use crate::{access_point::AccessPoint, station::Station};

#[derive(Debug, Clone)]
pub struct Device {
    session: Arc<Session>,
    pub device: iwdDevice,
    pub name: String,
    pub address: String,
    pub mode: String,
    pub is_powered: bool,
    pub station: Option<Station>,
    pub access_point: Option<AccessPoint>,
}

impl Device {
    pub async fn new(session: Arc<Session>) -> Result<Self> {
        let device = session.device().context("No device found")?;

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

        let access_point = match session.access_point() {
            Some(iwdrs_access_point) => match AccessPoint::new(iwdrs_access_point).await {
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
            access_point,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.is_powered = self.device.is_powered().await?;
        let current_mode = self.device.get_mode().await?;

        match current_mode.as_str() {
            "station" => {
                match self.mode.as_str() {
                    "station" => {
                        // refresh exisiting station
                        if let Some(station) = &mut self.station {
                            station.refresh().await?;
                        }
                    }
                    "ap" => {
                        // Switch mode from ap to station
                        self.access_point = None;
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
                    _ => {}
                }
            }
            "ap" => {
                match self.mode.as_str() {
                    "station" => {
                        self.station = None;
                        self.access_point = match self.session.access_point() {
                            Some(iwdrs_access_point) => {
                                match AccessPoint::new(iwdrs_access_point).await {
                                    Ok(v) => Some(v),
                                    Err(e) => {
                                        error!("{}", e.to_string());
                                        None
                                    }
                                }
                            }
                            None => None,
                        };
                    }
                    "ap" => {
                        // Switch mode
                        if self.access_point.is_some() {
                            self.access_point.as_mut().unwrap().refresh().await?;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        self.mode = current_mode;
        Ok(())
    }
}
