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
    
    if devices.is_empty() {
        info!("No Sonos devices found!");
        return Ok(());
    }

    // Create subscribers for all devices
    let mut handles = Vec::new();
    
    for device_name in devices {
        info!("Setting up subscriber for device: {}", device_name);
        let subscriber = EventSubscriber::new(&device_name).await?;
        subscriber.subscribe().await?;
        
        // Spawn a task for each device's event handling
        let handle = tokio::spawn(async move {
            if let Err(e) = subscriber.handle_events(move |event| {
                info!("Received event from {}: {:?}", device_name, event);
                Ok(())
            }).await {
                info!("Error handling events for {}: {}", device_name, e);
            }
        });
        
        handles.push(handle);
    }

    // Wait for ctrl-c while handling events
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    
    Ok(())
}
