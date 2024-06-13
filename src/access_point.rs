use anyhow::Result;
use iwdrs::access_point::AccessPoint as iwdAccessPoint;

#[derive(Debug, Clone)]
pub struct AccessPoint {
    pub a: iwdAccessPoint,
    pub has_started: bool,
    pub name: Option<String>,
    pub frequency: Option<u32>,
    pub is_scanning: Option<bool>,
    pub supported_ciphers: Option<Vec<String>>,
    pub used_cipher: Option<String>,
}

impl AccessPoint {
    pub async fn new(a: iwdAccessPoint) -> Result<Self> {
        let has_started = a.has_started().await?;
        let name = a.name().await?;
        let frequency = a.frequency().await?;
        let is_scanning = a.is_scanning().await.ok();
        let supported_ciphers = a.pairwise_ciphers().await?;
        let used_cipher = a.group_cipher().await?;
        Ok(Self {
            a,
            has_started,
            name,
            frequency,
            is_scanning,
            supported_ciphers,
            used_cipher,
        })
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.has_started = self.a.has_started().await?;
        self.name = self.a.name().await?;
        self.frequency = self.a.frequency().await?;
        self.is_scanning = self.a.is_scanning().await.ok();
        self.supported_ciphers = self.a.pairwise_ciphers().await?;
        self.used_cipher = self.a.group_cipher().await?;

        Ok(())
    }
}
