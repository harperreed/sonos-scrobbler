use anyhow::{Result, Context};
use rusty_sonos::discovery::discover_devices;
use std::time::Duration;

pub struct SonosDevice {
    pub ip_addr: String,
    pub room_name: String,
    pub model_name: String,
}

pub async fn discover_sonos_devices(
    discovery_timeout: Duration,
    response_timeout: Duration
) -> Result<Vec<SonosDevice>> {
    let devices = discover_devices(
        discovery_timeout.as_secs(),
        response_timeout.as_secs()
    ).await.context("Failed to discover devices")?;
    
    Ok(devices.into_iter()
        .map(|d| SonosDevice {
            ip_addr: d.ip_addr,
            room_name: d.room_name,
            model_name: d.model,
        })
        .collect())
}
