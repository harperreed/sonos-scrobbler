use anyhow::Result;
use log::info;
use rusty_sonos::{
    discovery::discover_devices,
    responses::CurrentTrack,
};
use std::time::Duration;

#[derive(Clone)]
pub struct EventSubscriber {
    device_ip: String,
    friendly_name: String,
}

impl EventSubscriber {
    pub async fn new(device_name: &str) -> Result<Self> {
        let devices = discover_devices(2, 5)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // Extract the RINCON ID from the input string
        // Format: "IP - Model Name - RINCON_ID, Room Name"
        let rincon_id = device_name
            .split(" - ")
            .nth(2)
            .and_then(|s| s.split(',').next())
            .ok_or_else(|| anyhow::anyhow!("Invalid device name format: {}", device_name))?;

        info!("Looking for device with RINCON ID: {}", rincon_id);
        
        let device = devices
            .into_iter()
            .inspect(|d| info!("Checking device: {}", d.friendly_name))
            .find(|d| d.friendly_name.contains(rincon_id))
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_name))?;

        Ok(Self {
            device_ip: device.ip_addr.to_string(),
            friendly_name: device.friendly_name.clone(),
        })
    }

    pub async fn poll_current_track(&self) -> Result<()> {
        info!("Starting track polling for device {}...", self.friendly_name);
        
        let mut last_track: Option<String> = None;
        
        loop {
            let current = CurrentTrack::get(&self.device_ip).await?;
            let track_info = match (current.artist, current.title) {
                (Some(artist), Some(title)) => format!("{} - {}", artist, title),
                (Some(artist), None) => artist,
                (None, Some(title)) => title,
                (None, None) => "Unknown Track".to_string(),
            };
            
            if let Some(last) = &last_track {
                if last != &track_info {
                    info!("Track changed on {}: {}", self.friendly_name, track_info);
                    last_track = Some(track_info);
                }
            } else {
                info!("Initial track on {}: {}", self.friendly_name, track_info);
                last_track = Some(track_info);
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

}
