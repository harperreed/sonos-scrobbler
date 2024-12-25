use rusty_sonos::discovery::Discoverer;
use anyhow::Result;
use log::{info, error};

pub struct SonosDiscovery {
    discoverer: Discoverer,
}

impl SonosDiscovery {
    pub async fn new() -> Result<Self> {
        let discoverer = Discoverer::new().await?;
        Ok(Self { discoverer })
    }

    pub async fn discover_devices(&self) -> Result<Vec<String>> {
        info!("Discovering Sonos devices...");
        let devices = self.discoverer.discover().await?;
        
        let device_ips: Vec<String> = devices
            .iter()
            .map(|device| device.ip.to_string())
            .collect();

        info!("Found {} Sonos devices", device_ips.len());
        Ok(device_ips)
    }
}
