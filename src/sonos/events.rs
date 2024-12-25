use anyhow::Result;
use futures::StreamExt;
use hyper::{Body, Request, Response, Server};
use log::{info, warn};
use rusty_sonos::discovery::{discover_devices, BasicSpeakerInfo};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct EventSubscriber {
    device: BasicSpeakerInfo,
}

impl EventSubscriber {
    pub async fn new(device_name: &str) -> Result<Self> {
        let devices = discover_devices(2, 5)
            .await
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        
        // Extract the RINCON ID from the input string
        // Format: "IP - Model Name - RINCON_ID, Room Name"
        let rincon_id = device_name
            .split(" - ")
            .nth(2)
            .and_then(|s| s.split(',').next())
            .ok_or_else(|| anyhow::anyhow!("Invalid device name format: {}", device_name))?;

        info!("Looking for device with RINCON ID: {}", rincon_id);
        
        let device = devices
            .into_iter()
            .inspect(|d| info!("Checking device: {}", d.friendly_name))
            .find(|d| d.friendly_name.contains(rincon_id))
            .ok_or_else(|| anyhow::anyhow!("Device not found: {}", device_name))?;

        Ok(Self { device })
    }

    pub async fn subscribe(&self) -> Result<()> {
        info!("Subscribing to Sonos events for device {}...", self.device.friendly_name);
        
        // Start local HTTP server to receive events
        let addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let (tx, mut rx) = mpsc::channel(100);
        let tx = Arc::new(tx);

        let make_service = hyper::service::make_service_fn(move |_| {
            let tx = tx.clone();
            async move {
                Ok::<_, hyper::Error>(hyper::service::service_fn(move |req: Request<Body>| {
                    let tx = tx.clone();
                    async move {
                        let body_bytes = hyper::body::to_bytes(req.into_body()).await?;
                        if let Ok(body_str) = String::from_utf8(body_bytes.to_vec()) {
                            info!("Received event: {}", body_str);
                            let _ = tx.send(body_str).await;
                        }
                        Ok::<_, hyper::Error>(Response::new(Body::empty()))
                    }
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_service);
        let addr = server.local_addr();
        info!("Event listener started on {}", addr);

        // Subscribe to Sonos events
        let callback_url = format!("http://{}/notify", addr);
        let client = reqwest::Client::new();
        
        // Subscribe to AVTransport events
        let sub_url = format!("http://{}/MediaRenderer/AVTransport/Event", self.device.ip_addr);
        let resp = client
            .subscribe(&sub_url)
            .header("CALLBACK", format!("<{}>", callback_url))
            .header("NT", "upnp:event")
            .header("TIMEOUT", "Second-300")
            .send()
            .await?;

        if !resp.status().is_success() {
            warn!("Failed to subscribe to events: {}", resp.status());
        }

        // Process events
        while let Some(event) = rx.recv().await {
            info!("Processing event: {}", event);
            // TODO: Parse XML event and extract relevant information
        }

        Ok(())
    }

    pub async fn handle_events<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(String) -> Result<()> + Send + 'static,
    {
        self.subscribe().await
    }
}
