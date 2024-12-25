use crate::sonos::TrackDatabase;
use anyhow::Result;
use log::info;
use rusty_sonos::{
    discovery::discover_devices,
    speaker::Speaker,
};
use std::net::IpAddr;
use std::time::Duration;

pub struct EventSubscriber {
    speaker: Speaker,
    friendly_name: String,
    db: TrackDatabase,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_event_subscriber_new_valid_device() {
        let device_name = "192.168.1.100 - Sonos Play:1 - RINCON_123456,Living Room";
        let result = EventSubscriber::new(device_name).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_subscriber_new_invalid_device_name() {
        let device_name = "Invalid Device Name";
        let result = EventSubscriber::new(device_name).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_rincon_id() {
        let device_name = "192.168.1.100 - Sonos Play:1 - RINCON_123456,Living Room";
        let rincon_id = device_name
            .split(" - ")
            .nth(2)
            .and_then(|s| s.split(',').next())
            .unwrap();
        assert_eq!(rincon_id, "RINCON_123456");
    }
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

        let ip_addr = IpAddr::V4(device.ip_addr);
        let speaker = Speaker::new(&ip_addr.to_string()).await
            .map_err(|e| anyhow::anyhow!("Failed to create speaker: {}", e))?;
        
        let db = TrackDatabase::new().await?;
        
        Ok(Self {
            speaker,
            friendly_name: device.friendly_name.clone(),
            db,
        })
    }

    pub async fn poll_current_track(&self) -> Result<()> {
        info!("Starting track polling for device {}...", self.friendly_name);
        
        let mut last_track: Option<String> = None;
        
        loop {
            let current = self.speaker.get_current_track().await
                .map_err(|e| anyhow::anyhow!("Failed to get current track: {}", e))?;
            let track_info = match (current.artist, current.title) {
                (Some(artist), Some(title)) => format!("{} - {}", artist, title),
                (Some(artist), None) => artist,
                (None, Some(title)) => title,
                (None, None) => "Unknown Track".to_string(),
            };
            
            if let Some(last) = &last_track {
                if last != &track_info {
                    if self.db.log_track(&self.friendly_name, &track_info).await? {
                        info!("Track changed and logged on {}: {}", self.friendly_name, track_info);
                        last_track = Some(track_info);
                    }
                }
            } else {
                if self.db.log_track(&self.friendly_name, &track_info).await? {
                    info!("Initial track logged on {}: {}", self.friendly_name, track_info);
                    last_track = Some(track_info);
                }
            }
            
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

}
