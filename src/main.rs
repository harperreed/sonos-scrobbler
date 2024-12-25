use anyhow::Result;
use log::info;
use sonos_scrobbler::sonos::{SonosDiscovery, EventSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Sonos Scrobbler...");

    // Initialize Sonos discovery
    let discovery = SonosDiscovery::new().await?;
    
    // Discover and list devices
    let devices = discovery.discover_devices().await?;
    info!("Available devices:");
    for (i, device) in devices.iter().enumerate() {
        info!("  {}: {}", i + 1, device);
    }
    
    // Pick first device for now
    if let Some(device_name) = devices.first() {
        info!("Selecting first device: {}", device_name);
        
        // Subscribe to events for selected device
        let subscriber = EventSubscriber::new(device_name).await?;
        subscriber.subscribe().await?;
        
        subscriber.handle_events(|event| {
            info!("Received event: {:?}", event);
            Ok(())
        }).await?;
    } else {
        info!("No Sonos devices found!");
    }

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}
