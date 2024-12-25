use anyhow::Result;
use log::info;
use rusty_sonos::discovery::Sonos;

pub struct SonosDiscovery {
    sonos: Sonos,
}

impl SonosDiscovery {
    pub async fn new() -> Result<Self> {
        let sonos = Sonos::discover().await.map_err(anyhow::Error::from)?;
        Ok(Self { sonos })
    }

    pub async fn discover_devices(&self) -> Result<Vec<String>> {
        info!("Discovering Sonos devices...");
        let devices = self.sonos.devices();
        
        let device_ips: Vec<String> = devices
            .iter()
            .map(|device| device.ip().to_string())
            .collect();

        info!("Found {} Sonos devices", device_ips.len());
        Ok(device_ips)
    }
}
