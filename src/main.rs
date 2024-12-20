mod sonos;
mod device_manager;

use dotenv::dotenv;
use log::{info, error, warn};
use rusty_sonos::discovery;
use crate::sonos::get_current_track_info;
use crate::device_manager::DeviceManager;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::time;
use anyhow::Result;

/// Initialize the logging system with environment-based configuration
fn setup_logging() {
    env_logger::init();
}

const DISCOVERY_TIMEOUT_MS: u64 = 30000;  // 30 seconds
const RESPONSE_TIMEOUT_MS: u64 = 10000;   // 10 seconds
const MAX_RETRIES: u32 = 5;               // Increased from 3
const RETRY_DELAY_SECS: u64 = 5;          // Increased from 2

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables and setup logging
    dotenv().ok();
    setup_logging();
    
    info!("Starting Sonos Scrobbler");
    
    // Discover Sonos devices
    info!("Discovering Sonos devices... (timeout: {}s)", DISCOVERY_TIMEOUT_MS / 1000);
    let mut retry_count = 0;
    let mut devices = Vec::new();

    while retry_count < MAX_RETRIES {
        info!("Discovery attempt {}/{}", retry_count + 1, MAX_RETRIES);
        match discovery::discover_devices(DISCOVERY_TIMEOUT_MS as i32, RESPONSE_TIMEOUT_MS as i32).await {
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
                // Log more detailed error information
                if let Some(source) = e.source() {
                    error!("Error source: {}", source);
                }
            }
        }
        retry_count += 1;
        if retry_count < MAX_RETRIES {
            info!("Waiting {} seconds before next retry...", RETRY_DELAY_SECS);
            time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
        }
    }
    
    if devices.is_empty() {
        error!("No Sonos devices found after {} attempts. Please check:", MAX_RETRIES);
        error!("  1. Are you on the same network as your Sonos devices?");
        error!("  2. Is UDP port 1900 open (required for SSDP)?");
        error!("  3. Is multicast traffic allowed on your network?");
        error!("  4. Are you using a VPN that might block discovery?");
        return Ok(());
    }
    
    info!("Found {} Sonos device(s)", devices.len());
    for device in &devices {
        info!("  - {} at {}", device.room_name, device.ip_addr);
    }
    
    // Get the first device and monitor its playback
    if let Some(device) = devices.first() {
        info!("Monitoring speaker: {} at {}", device.room_name, device.ip_addr);
        let mut device_manager = DeviceManager::new(
            device.ip_addr.to_string(),
            device.room_name.clone(),
        );

        // Setup signal handling for graceful shutdown
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        let tx_clone = tx.clone();
        
        ctrlc::set_handler(move || {
            if let Some(tx) = tx_clone.lock().unwrap().take() {
                if let Err(e) = tx.send(()) {
                    error!("Failed to send shutdown signal: {:?}", e);
                }
            }
        })?;

        if let Err(e) = device_manager.connect().await {
            error!("Failed to connect to device: {}", e);
            return Ok(());
        }

        let mut interval = time::interval(Duration::from_secs(5));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if !device_manager.check_connection().await {
                        error!("Lost connection to device {} and unable to reconnect", device_manager.get_room_name());
                        break;
                    }

                    info!("Device state: {:?}", device_manager.get_state());

                    match get_current_track_info(device_manager.get_ip()).await {
                        Ok(track) => {
                            info!("Now Playing: {}", track.title);
                            info!("Artist: {}", track.artist);
                            info!("Album: {}", track.album);
                            info!("Position: {} / {}", track.position, track.duration);
                        }
                        Err(e) => {
                            error!("Failed to get track info: {}", e);
                        }
                    }
                }
                _ = &mut rx => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        // Cleanup before exit
        device_manager.cleanup().await;
        info!("Shutdown complete");
    }
    
    Ok(())
}
