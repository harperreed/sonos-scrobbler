use anyhow::{Context, Result};
use log::{error, info, warn};
use rusty_sonos::{discovery, speaker::Speaker};
use std::time::Duration;
use tokio::time;

const CONNECTION_TIMEOUT_SECS: u64 = 5;
const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_SECS: u64 = 5;

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
    speaker: Option<Speaker>,
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
            max_retries: MAX_RETRIES,
            speaker: None,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!(
            "Connecting to device {} at {}",
            self.room_name, self.ip_addr
        );

        // Try to establish initial connection
        match self.ping().await {
            Ok(_) => {
                info!("Successfully connected to device {}", self.room_name);
                self.state = ConnectionState::Connected;
                self.retry_count = 0;
                // Initialize speaker
                self.speaker = Some(Speaker::new(&self.ip_addr));
                Ok(())
            }
            Err(e) => {
                error!(
                    "Failed to establish initial connection to {}: {}",
                    self.room_name, e
                );
                self.state = ConnectionState::Disconnected;
                Err(e)
            }
        }
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

        client
            .get(url)
            .timeout(Duration::from_secs(CONNECTION_TIMEOUT_SECS))
            .send()
            .await
            .context("Failed to ping device")?;

        Ok(())
    }

    async fn handle_connection_failure(&mut self) -> bool {
        self.state = ConnectionState::Reconnecting;
        self.retry_count += 1;

        if self.retry_count > self.max_retries {
            error!(
                "Max reconnection attempts reached for device {}",
                self.room_name
            );
            error!("Please check:");
            error!("  1. Is the device powered on and connected to the network?");
            error!(
                "  2. Can you access the device's web interface at http://{}:1400?",
                self.ip_addr
            );
            error!("  3. Are there any network connectivity issues?");
            self.state = ConnectionState::Disconnected;
            return false;
        }

        warn!(
            "Attempting to reconnect to {} (attempt {}/{})",
            self.room_name, self.retry_count, self.max_retries
        );

        // Try to rediscover devices with increased timeouts
        match discovery::discover_devices(30000, 10000).await {
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
                error!("Error details: {}", e);
            }
        }

        info!("Waiting {} seconds before next retry...", RETRY_DELAY_SECS);
        time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
        false
    }

    pub fn get_ip(&self) -> &str {
        &self.ip_addr
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    pub async fn cleanup(&mut self) {
        info!("Cleaning up device manager for {}", self.room_name);
        // Ensure we're marked as disconnected
        self.state = ConnectionState::Disconnected;
        // Add a small delay to allow any pending operations to complete
        time::sleep(Duration::from_millis(100)).await;
    }

    pub fn get_room_name(&self) -> &str {
        &self.room_name
    }

    pub fn get_state(&self) -> &ConnectionState {
        &self.state
    }

    pub async fn get_current_track(&self) -> Result<()> {
        if let Some(speaker) = &self.speaker {
            match speaker.get_current_track().await {
                Ok(track) => {
                    if !track.title.is_empty() {
                        info!(
                            "[{}] Now playing: {} - {} (from {})",
                            self.room_name, track.artist, track.title, track.album
                        );
                    }
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to get track info: {}", e);
                    Ok(())
                }
            }
        } else {
            warn!("Speaker not initialized");
            Ok(())
        }
    }
}
