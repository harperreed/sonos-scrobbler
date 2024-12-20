use dotenv::dotenv;
use log::{info, debug, error};
use std::env;
mod sonos;
use std::time::Duration;
use futures_util::StreamExt;
use ssdp_client::{SearchTarget, URN};
use anyhow::{Result, Context};

/// Initialize the logging system with environment-based configuration
fn setup_logging() {
    // Initialize env_logger with RUST_LOG environment variable
    env_logger::init();
}

/// Load and return configuration from environment variables
fn load_config() -> String {
    // Example of reading a configuration value
    env::var("APP_NAME").unwrap_or_else(|_| "default_name".to_string())
}

/// Discover Sonos devices on the local network using SSDP
/// Returns a vector of IP addresses of discovered Sonos devices
async fn discover_sonos_devices() -> Result<Vec<String>> {
    info!("Starting Sonos device discovery...");
    
    // Create SSDP search target for Sonos devices
    // Sonos devices use the urn:schemas-upnp-org:device:ZonePlayer:1 search target
    let search_target = SearchTarget::URN(
        URN::device("schemas-upnp-org", "ZonePlayer", 1)
    );
    
    // Configure search options
    let timeout = Duration::from_secs(3);
    
    // Perform the SSDP search
    let responses = ssdp_client::search(&search_target, timeout, 2)
        .await
        .context("Failed to perform SSDP search")?;
    
    // Extract IP addresses from responses
    let responses = responses.collect::<Vec<_>>().await;
    let devices: Vec<String> = responses
        .into_iter()
        .filter_map(|response| {
            match response {
                Ok(response) => {
                    let location = response.location();
                    if let Ok(url) = url::Url::parse(location) {
                        if let Some(host) = url.host_str() {
                            let host_string = host.to_string();
                            if !host_string.is_empty() {
                                Some(host_string)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                },
                Err(e) => {
                    debug!("Error processing SSDP response: {}", e);
                    None
                }
            }
        })
        .collect();
    
    info!("Found {} Sonos device(s)", devices.len());
    for (i, ip) in devices.iter().enumerate() {
        debug!("Device {}: {}", i + 1, ip);
    }
    
    Ok(devices)
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Setup logging system
    setup_logging();
    
    // Load configuration
    let app_name = load_config();
    
    // Log some messages at different levels
    debug!("Debug message - Configuration loaded");
    info!("Starting {} application", app_name);
    
    // Discover Sonos devices
    match discover_sonos_devices().await {
        Ok(devices) => {
            if devices.is_empty() {
                info!("No Sonos devices found on the network");
            } else {
                info!("Successfully discovered Sonos devices");
            }
        }
        Err(e) => {
            error!("Failed to discover Sonos devices: {}", e);
        }
    }
    
    // Subscribe to events for the first discovered device
    if let Ok(devices) = discover_sonos_devices().await {
        if let Some(first_device) = devices.first() {
            info!("Subscribing to events for device: {}", first_device);
            if let Err(e) = sonos::subscribe_to_playback_events(first_device).await {
                error!("Failed to subscribe to events: {}", e);
            }
        }
    }
}
