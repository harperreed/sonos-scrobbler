mod sonos;
mod device_manager;
mod discovery;

use dotenv::dotenv;
use log::{info, error, warn};
use crate::device_manager::DeviceManager;
use crate::discovery::discover_sonos_devices;
use std::time::Duration;
use tokio::time;
use anyhow::Result;

// Discovery timeouts
const DISCOVERY_TIMEOUT_SECS: u64 = 2;  // Search timeout
const RESPONSE_TIMEOUT_SECS: u64 = 5;   // Read timeout
const MAX_RETRIES: u32 = 3;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize environment and logging
    dotenv().ok();
    env_logger::init();
    
    info!("Starting Sonos Scrobbler");
    
    // Discover Sonos devices using SSDP
    info!("Starting Sonos device discovery...");
    let mut retry_count = 0;
    let mut devices = Vec::new();

    while retry_count < MAX_RETRIES {
        info!("Discovery attempt {}/{}", retry_count + 1, MAX_RETRIES);
        match discover_sonos_devices(
            Duration::from_secs(DISCOVERY_TIMEOUT_SECS),
            Duration::from_secs(RESPONSE_TIMEOUT_SECS)
        ).await {
            Ok(found_devices) => {
                if !found_devices.is_empty() {
                    info!("Successfully found {} devices!", found_devices.len());
                    devices = found_devices;
                    break;
                }
                warn!("No devices found on attempt {}/{}", retry_count + 1, MAX_RETRIES);
            }
            Err(e) => {
                error!("Discovery attempt failed: {}", e);
                if let Some(source) = e.source() {
                    error!("Error source: {}", source);
                }
            }
        }
        retry_count += 1;
        if retry_count < MAX_RETRIES {
            info!("Waiting before next attempt...");
            time::sleep(Duration::from_secs(2)).await;
        }
    }
    
    if devices.is_empty() {
        error!("No Sonos devices found. Please check:");
        error!("  1. Are your Sonos devices powered on?");
        error!("  2. Are you on the same network as your Sonos devices?");
        error!("  3. Is multicast traffic (UDP port 1900) allowed?");
        error!("  4. Try running 'nmap -p 1400 192.168.1.0/24' to scan for Sonos devices");
        return Ok(());
    }
    
    // Log discovered devices
    info!("Found {} Sonos device(s):", devices.len());
    for device in &devices {
        info!("  - {} ({}) at {}", device.room_name, device.model_name, device.ip_addr);
    }
    
    // Rest of the code remains the same...
    if let Some(device) = devices.first() {
        info!("Monitoring speaker: {} at {}", device.room_name, device.ip_addr);
        let mut device_manager = DeviceManager::new(
            device.ip_addr.clone(),
            device.room_name.clone(),
        );

        device_manager.connect().await?;

        loop {
            if device_manager.check_connection().await {
                device_manager.get_current_track().await?;
            }
            time::sleep(Duration::from_secs(5)).await;
        }
    }
    
    Ok(())
}
