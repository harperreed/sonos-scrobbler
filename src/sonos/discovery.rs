use anyhow::Result;
use log::info;
use rusty_sonos::discovery::{discover_devices, BasicSpeakerInfo};
use std::time::Duration;

pub struct SonosDiscovery {
    devices: Vec<BasicSpeakerInfo>,
}

impl SonosDiscovery {
    pub async fn new() -> Result<Self> {
        let devices = discover_devices(2, 5)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        Ok(Self { devices })
    }

    pub async fn discover_devices(&self) -> Result<Vec<String>> {
        info!("Discovering Sonos devices...");
        
        let device_info: Vec<String> = self.devices
            .iter()
            .map(|device| format!("{}, {}", device.friendly_name, device.room_name))
            .collect();

        info!("Found {} Sonos devices", device_info.len());
        Ok(device_info)
    }
}
