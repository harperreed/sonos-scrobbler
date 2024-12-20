use anyhow::{Result, Context, anyhow};
use log::{error, info, debug};
use serde::Deserialize;
use std::time::Duration;
use std::io::BufReader;

/// Represents detailed track information from a Sonos device
#[derive(Debug, Deserialize)]
pub struct TrackInfo {
    #[serde(rename = "Track")]
    pub title: String,
    #[serde(rename = "Artist")]
    pub artist: String,
    #[serde(rename = "Album")]
    pub album: String,
    #[serde(rename = "TrackDuration")]
    pub duration: String,
    #[serde(rename = "RelTime")]
    pub position: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SoapEnvelope {
    #[serde(rename = "s:Body")]
    body: SoapBody,
}

#[derive(Debug, Deserialize)]
struct SoapBody {
    #[serde(rename = "u:GetPositionInfoResponse")]
    position_info_response: PositionInfo,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PositionInfo {
    track_meta_data: String,
    rel_time: String,
    track_duration: String,
}

/// Represents the currently playing track information
#[derive(Debug, Deserialize)]
pub struct PlaybackState {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub position: Option<String>,
    pub duration: Option<String>,
}

/// Subscribe to playback events from a Sonos device
/// 
/// This function sets up a polling mechanism to periodically check
/// the playback state of a Sonos device at the specified IP address.
/// 
/// # Arguments
/// * `device_ip` - IP address of the Sonos device
/// 
/// # Returns
/// A Result containing a stream of PlaybackState updates
pub async fn subscribe_to_playback_events(device_ip: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:1400", device_ip);
    
    // Poll the device every 5 seconds for updates
    let mut interval = tokio::time::interval(Duration::from_secs(5));
    
    loop {
        interval.tick().await;
        
        match get_current_playback_state(&client, &base_url).await {
            Ok(state) => {
                if let Some(title) = &state.title {
                    info!("Now playing: {}", title);
                    if let Some(artist) = &state.artist {
                        info!("Artist: {}", artist);
                    }
                }
            }
            Err(e) => {
                error!("Failed to get playback state: {}", e);
            }
        }
    }
}

/// Get the current playback state from a Sonos device
/// Get current track information from a Sonos device
///
/// # Arguments
/// * `device_ip` - IP address of the Sonos device
///
/// # Returns
/// Result containing track information including title, artist, album and timing
pub async fn get_current_track_info(device_ip: &str) -> Result<TrackInfo> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:1400", device_ip);
    
    // SOAP request to get current track info
    let soap_body = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
            <s:Body>
                <u:GetPositionInfo xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                    <InstanceID>0</InstanceID>
                </u:GetPositionInfo>
            </s:Body>
        </s:Envelope>"#;

    let response = client
        .post(format!("{}/MediaRenderer/AVTransport/Control", base_url))
        .header("SOAPAction", "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"")
        .header("Content-Type", "text/xml")
        .body(soap_body)
        .send()
        .await
        .context("Failed to send request to Sonos device")?;

    let response_text = response.text().await?;
    
    // Parse the full response into our response structure
    // Parse the SOAP response with namespace awareness
    let reader = BufReader::new(response_text.as_bytes());
    let envelope: SoapEnvelope = quick_xml::de::from_reader(reader)
        .context("Failed to parse SOAP envelope")?;

    // The track metadata is embedded as an escaped XML string, we need to unescape it
    let track_metadata = envelope.body.position_info_response.track_meta_data;
    
    // Parse the track metadata XML
    let track_info = TrackInfo {
        title: extract_didl_value(&track_metadata, "dc:title")
            .unwrap_or_else(|_| "Unknown Title".to_string()),
        artist: extract_didl_value(&track_metadata, "dc:creator")
            .unwrap_or_else(|_| "Unknown Artist".to_string()),
        album: extract_didl_value(&track_metadata, "upnp:album")
            .unwrap_or_else(|_| "Unknown Album".to_string()),
        duration: envelope.body.position_info_response.track_duration,
        position: envelope.body.position_info_response.rel_time,
    };

    Ok(track_info)
}

/// Helper function to extract values from DIDL-Lite XML
fn extract_didl_value(xml: &str, tag: &str) -> Result<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    let start = xml.find(&start_tag)
        .ok_or_else(|| anyhow!("Could not find start tag: {}", tag))?;
    let end = xml.find(&end_tag)
        .ok_or_else(|| anyhow!("Could not find end tag: {}", tag))?;
    
    Ok(xml[start + start_tag.len()..end].to_string())
}

async fn get_current_playback_state(client: &reqwest::Client, base_url: &str) -> Result<PlaybackState> {
    // SOAP request to get current track info
    let soap_body = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
            <s:Body>
                <u:GetPositionInfo xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                    <InstanceID>0</InstanceID>
                </u:GetPositionInfo>
            </s:Body>
        </s:Envelope>"#;

    let response = client
        .post(format!("{}/MediaRenderer/AVTransport/Control", base_url))
        .header("SOAPAction", "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"")
        .header("Content-Type", "text/xml")
        .body(soap_body)
        .send()
        .await
        .context("Failed to send request to Sonos device")?;

    let response_text = response.text().await?;
    
    // Log the raw response for debugging
    debug!("Raw SOAP response: {}", response_text);
    
    // Parse the SOAP response
    let reader = BufReader::new(response_text.as_bytes());
    let envelope: SoapEnvelope = quick_xml::de::from_reader(reader)
        .context(format!("Failed to parse SOAP envelope: {}", response_text))?;

    // Log parsed envelope for debugging
    debug!("Parsed envelope: {:?}", envelope);

    // Extract track metadata
    let track_metadata = envelope.body.position_info_response.track_meta_data;
    
    // Handle empty metadata case
    if track_metadata.trim().is_empty() {
        debug!("Empty track metadata received");
        return Ok(PlaybackState {
            title: None,
            artist: None,
            album: None,
            position: Some(envelope.body.position_info_response.rel_time),
            duration: Some(envelope.body.position_info_response.track_duration),
        });
    }
    
    // Create PlaybackState from parsed data
    let state = PlaybackState {
        title: extract_didl_value(&track_metadata, "dc:title").ok(),
        artist: extract_didl_value(&track_metadata, "dc:creator").ok(),
        album: extract_didl_value(&track_metadata, "upnp:album").ok(),
        position: Some(envelope.body.position_info_response.rel_time),
        duration: Some(envelope.body.position_info_response.track_duration),
    };
    
    debug!("Parsed playback state: {:?}", state);
    Ok(state)
}
