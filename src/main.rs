mod sonos;

use dotenv::dotenv;
use log::{debug, info, error};
use rusty_sonos::discovery;
use crate::sonos::get_current_track_info;
use std::time::Duration;
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
        let ip = device.ip_addr.to_string();
        debug!("Using device IP: {}", ip);
        
        let mut interval = time::interval(Duration::from_secs(5));
        
        loop {
            interval.tick().await;
            
            match get_current_track_info(&ip).await {
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
    }
    
    Ok(())
}
