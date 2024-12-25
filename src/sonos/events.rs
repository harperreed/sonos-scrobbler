use anyhow::Result;
use log::{info, error};
use rusty_sonos::discovery::Discoverer;
use rusty_sonos::events::{EventSubscriber as SonosEventSubscriber, Event};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct EventSubscriber {
    subscriber: Arc<Mutex<SonosEventSubscriber>>,
}

impl EventSubscriber {
    pub async fn new(device_ip: &str) -> Result<Self> {
        let subscriber = SonosEventSubscriber::new(device_ip).await?;
        Ok(Self {
            subscriber: Arc::new(Mutex::new(subscriber))
        })
    }

    pub async fn subscribe(&self) -> Result<()> {
        info!("Subscribing to Sonos events...");
        let mut subscriber = self.subscriber.lock().await;
        subscriber.subscribe().await?;
        Ok(())
    }

    pub async fn handle_events<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(Event) -> Result<()> + Send + 'static,
    {
        let subscriber = Arc::clone(&self.subscriber);
        
        tokio::spawn(async move {
            loop {
                let mut sub = subscriber.lock().await;
                match sub.next_event().await {
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
        });

        Ok(())
    }
}
