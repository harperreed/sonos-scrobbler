use dotenv::dotenv;
use log::{info, debug, error};
use std::env;
use std::time::Duration;
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
        URN::new("schemas-upnp-org:device:ZonePlayer:1").unwrap()
    );
    
    // Configure search options
    let timeout = Duration::from_secs(3);
    
    // Perform the SSDP search
    let responses = ssdp_client::search(&search_target, timeout, 2)
        .await
        .context("Failed to perform SSDP search")?;
    
    // Extract IP addresses from responses
    let devices: Vec<String> = responses
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .filter_map(|response| {
            response.location()
                .and_then(|url| url.host_str())
                .map(|host| host.to_string())
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
    
    info!("Application completed");
}
