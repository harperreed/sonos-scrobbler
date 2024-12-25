use anyhow::Result;
use log::info;
use rusty_sonos::discovery::{discover_devices, BasicSpeakerInfo};

pub struct EventSubscriber {
    device: BasicSpeakerInfo,
}

impl EventSubscriber {
    pub async fn new(device_name: &str) -> Result<Self> {
        let devices = discover_devices(2, 5)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // Extract just the model name from the combined string
        // Format: "IP - Model Name - RINCON_ID, Room Name"
        let model_name = device_name
            .split(" - ")
            .nth(1)
            .ok_or_else(|| anyhow::anyhow!("Invalid device name format: {}", device_name))?;

        let device = devices
            .into_iter()
            .find(|d| d.friendly_name == model_name)
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
