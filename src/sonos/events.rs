use anyhow::Result;
use log::{info, error};
use rusty_sonos::discovery::Sonos;
use rusty_sonos::events::Event;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct EventSubscriber {
    sonos: Arc<Mutex<Sonos>>,
    device_ip: String,
}

impl EventSubscriber {
    pub async fn new(device_ip: &str) -> Result<Self> {
        let sonos = Sonos::discover().await?;
        Ok(Self {
            sonos: Arc::new(Mutex::new(sonos)),
            device_ip: device_ip.to_string(),
        })
    }

    pub async fn subscribe(&self) -> Result<()> {
        info!("Subscribing to Sonos events for device {}...", self.device_ip);
        let sonos = self.sonos.lock().await;
        if let Some(device) = sonos.device(&self.device_ip) {
            device.subscribe_events().await?;
        }
        Ok(())
    }

    pub async fn handle_events<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Event) -> Result<()> + Send + 'static,
    {
        let sonos: Arc<Mutex<Sonos>> = Arc::clone(&self.sonos);
        let device_ip = self.device_ip.clone();
        
        tokio::spawn(async move {
            loop {
                let sonos_guard = sonos.lock().await;
                if let Some(device) = sonos_guard.device(&device_ip) {
                    match device.next_event().await {
                        Ok(event) => {
                            if let Err(e) = callback(event) {
                                error!("Error handling event: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Error getting next event: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
