use anyhow::Result;
use log::info;
use rusty_sonos::discovery::{discover_devices, BasicSpeakerInfo};
use std::time::Duration;

pub struct EventSubscriber {
    device: BasicSpeakerInfo,
}

impl EventSubscriber {
    pub async fn new(device_name: &str) -> Result<Self> {
        let devices = discover_devices(2, 5)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        let device = devices
            .into_iter()
            .find(|d| d.friendly_name == device_name)
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_name))?;

        Ok(Self { device })
    }

    pub async fn subscribe(&self) -> Result<()> {
        info!("Subscribing to Sonos events for device {}...", self.device.friendly_name);
        // TODO: Implement event subscription once we understand the correct API
        Ok(())
    }

    pub async fn handle_events<F>(&self, _callback: F) -> Result<()>
    where
        F: Fn(String) -> Result<()> + Send + 'static,
    {
        // TODO: Implement event handling once we understand the correct API
        Ok(())
    }
}
