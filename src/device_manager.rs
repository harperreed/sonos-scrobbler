use anyhow::{Result, Context};
use log::{error, info, warn};
use std::time::Duration;
use tokio::time;
use rusty_sonos::discovery;

#[derive(Debug)]
pub enum ConnectionState {
    Connected,
    Disconnected,
    Reconnecting,
}

pub struct DeviceManager {
    ip_addr: String,
    room_name: String,
    state: ConnectionState,
    retry_count: u32,
    max_retries: u32,
}

impl DeviceManager {
    pub fn new(ip_addr: String, room_name: String) -> Self {
        Self {
            ip_addr,
            room_name,
            state: ConnectionState::Disconnected,
            retry_count: 0,
            max_retries: 5,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to device {} at {}", self.room_name, self.ip_addr);
        self.state = ConnectionState::Connected;
        self.retry_count = 0;
        Ok(())
    }

    pub async fn check_connection(&mut self) -> bool {
        match self.ping().await {
            Ok(_) => {
                if !matches!(self.state, ConnectionState::Connected) {
                    info!("Device {} is now connected", self.room_name);
                    self.state = ConnectionState::Connected;
                    self.retry_count = 0;
                }
                true
            }
            Err(e) => {
                warn!("Connection check failed for {}: {}", self.room_name, e);
                self.handle_connection_failure().await
            }
        }
    }

    async fn ping(&self) -> Result<()> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:1400/status/info", self.ip_addr);
        
        client.get(url)
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .context("Failed to ping device")?;
        
        Ok(())
    }

    async fn handle_connection_failure(&mut self) -> bool {
        self.state = ConnectionState::Reconnecting;
        self.retry_count += 1;

        if self.retry_count > self.max_retries {
            error!("Max reconnection attempts reached for device {}", self.room_name);
            self.state = ConnectionState::Disconnected;
            return false;
        }

        warn!(
            "Attempting to reconnect to {} (attempt {}/{})",
            self.room_name, self.retry_count, self.max_retries
        );

        // Try to rediscover devices
        match discovery::discover_devices(5000, 5000).await {
            Ok(devices) => {
                if let Some(device) = devices.iter().find(|d| d.room_name == self.room_name) {
                    if device.ip_addr.to_string() != self.ip_addr {
                        info!(
                            "Device {} found at new IP: {} (old: {})",
                            self.room_name, device.ip_addr, self.ip_addr
                        );
                        self.ip_addr = device.ip_addr.to_string();
                    }
                    return true;
                }
            }
            Err(e) => {
                error!("Failed to rediscover devices: {}", e);
            }
        }

        time::sleep(Duration::from_secs(5)).await;
        false
    }

    pub fn get_ip(&self) -> &str {
        &self.ip_addr
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }
}
