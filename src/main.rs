use anyhow::Result;
use log::info;
use sonos_scrobbler::sonos::{SonosDiscovery, EventSubscriber};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting Sonos Scrobbler...");

    // Initialize Sonos discovery
    let discovery = SonosDiscovery::new().await?;
    
    // Discover devices
    let devices = discovery.discover_devices().await?;
    
    for device_ip in devices {
        // Subscribe to events for each device
        let subscriber = EventSubscriber::new(&device_ip).await?;
        subscriber.subscribe().await?;
        
        subscriber.handle_events(|event| {
            info!("Received event: {:?}", event);
            Ok(())
        }).await?;
    }

    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}
