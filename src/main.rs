mod sonos;
mod device_manager;

use dotenv::dotenv;
use log::{info, error};
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

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables and setup logging
    dotenv().ok();
    setup_logging();
    
    info!("Starting Sonos Scrobbler");
    
    // Discover Sonos devices
    info!("Discovering Sonos devices...");
    // Use 5 second timeout for both discovery and response
    let devices = discovery::discover_devices(5000, 5000).await.map_err(anyhow::Error::msg)?;
    
    if devices.is_empty() {
        info!("No Sonos devices found on the network");
        return Ok(());
    }
    
    info!("Found {} Sonos device(s)", devices.len());
    
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
