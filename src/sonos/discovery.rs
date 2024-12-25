use anyhow::Result;
use log::info;
use rusty_sonos::discovery::{discover_devices, BasicSpeakerInfo};

pub struct SonosDiscovery {
    devices: Vec<BasicSpeakerInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_discovery_new() {
        let discovery = SonosDiscovery::new().await;
        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_discover_devices_formats_correctly() {
        let mut devices = Vec::new();
        devices.push(BasicSpeakerInfo {
            ip_addr: "192.168.1.100".parse().unwrap(),
            friendly_name: "Living Room".to_string(),
            room_name: "Living Room".to_string(),
            device_type: "ZPS1".to_string(),
        });

        let discovery = SonosDiscovery { devices };
        let result = discovery.discover_devices().await.unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "Living Room, Living Room");
    }
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
