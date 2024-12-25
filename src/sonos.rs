use anyhow::{anyhow, Context, Result};
use log::debug;
use serde::Deserialize;
use std::io::BufReader;

/// Common response structure for Sonos track information
#[derive(Debug)]
pub struct SonosResponse {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub position: String,
    pub duration: String,
}

/// Represents detailed track information from a Sonos device

impl From<SonosResponse> for PlaybackState {
    fn from(response: SonosResponse) -> Self {
        Self {
            title: response.title,
            artist: response.artist,
            album: response.album,
            position: Some(response.position),
            duration: Some(response.duration),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE_SOAP_RESPONSE: &str = r#"<?xml version="1.0"?>
        <s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/">
            <s:Body>
                <u:GetPositionInfoResponse xmlns:u="urn:schemas-upnp-org:service:AVTransport:1">
                    <TrackMetaData>
                        <DIDL-Lite>
                            <dc:title>Test Song</dc:title>
                            <dc:creator>Test Artist</dc:creator>
                            <upnp:album>Test Album</upnp:album>
                        </DIDL-Lite>
                    </TrackMetaData>
                    <RelTime>0:01:23</RelTime>
                    <TrackDuration>0:04:56</TrackDuration>
                </u:GetPositionInfoResponse>
            </s:Body>
        </s:Envelope>"#;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_get_current_track_info() {
        let mock_server = mockito::Server::new();

        // Setup mock response
        let _m = mock_server
            .mock("POST", "/MediaRenderer/AVTransport/Control")
            .with_status(200)
            .with_header("content-type", "text/xml")
            .with_body(SAMPLE_SOAP_RESPONSE)
            .create();

        let result = get_current_track_info(&mock_server.url()[7..])
            .await
            .unwrap();

        assert_eq!(result.title, Some("Test Song".to_string()));
        assert_eq!(result.artist, Some("Test Artist".to_string()));
        assert_eq!(result.album, Some("Test Album".to_string()));
        assert_eq!(result.position, "0:01:23".to_string());
        assert_eq!(result.duration, "0:04:56".to_string());
    }

    #[test]
    fn test_extract_didl_value() {
        let xml = r#"<DIDL-Lite xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test Song</dc:title></DIDL-Lite>"#;
        let result = extract_didl_value(xml, "dc:title").unwrap();
        assert_eq!(result, "Test Song");
    }

    #[test]
    fn test_extract_didl_value_with_entities() {
        let xml = r#"<DIDL-Lite><dc:title>Rock &amp; Roll</dc:title></DIDL-Lite>"#;
        let result = extract_didl_value(xml, "dc:title").unwrap();
        assert_eq!(result, "Rock & Roll");
    }

    #[test]
    fn test_extract_didl_value_with_namespace() {
        let xml = r#"<DIDL-Lite xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>Test</dc:title></DIDL-Lite>"#;
        let result = extract_didl_value(xml, "dc:title").unwrap();
        assert_eq!(result, "Test");
    }

    #[test]
    fn test_extract_didl_value_missing_tag() {
        let xml = r#"<DIDL-Lite><dc:other>Test</dc:other></DIDL-Lite>"#;
        assert!(extract_didl_value(xml, "dc:title").is_err());
    }

    #[test]
    fn test_extract_didl_value_malformed_xml() {
        let xml = r#"<DIDL-Lite><dc:title>Test</dc:title"#; // More severely malformed XML
        assert!(extract_didl_value(xml, "dc:title").is_err());
    }
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

/// Get the current playback state from a Sonos device
/// Get current track information from a Sonos device
///
/// # Arguments
/// * `device_ip` - IP address of the Sonos device
///
/// # Returns
/// Result containing track information including title, artist, album and timing
pub async fn get_current_track_info(device_ip: &str) -> Result<PlaybackState> {
    let client = reqwest::Client::new();
    let base_url = format!("http://{}:1400", device_ip);

    let sonos_response = get_sonos_info(&client, &base_url).await?;
    Ok(PlaybackState::from(sonos_response))
}

/// Helper function to extract values from DIDL-Lite XML with namespace support
fn extract_didl_value(xml: &str, tag: &str) -> Result<String> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut buf = Vec::new();
    let mut inside_target_tag = false;
    let mut value = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                // Handle both prefixed and unprefixed tags
                if name.local_name().as_ref() == tag.split(':').last().unwrap_or(tag).as_bytes() {
                    inside_target_tag = true;
                }
            }
            Ok(Event::Text(e)) if inside_target_tag => {
                value = e
                    .unescape()
                    .context("Failed to unescape XML text")?
                    .to_string();
                break;
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.local_name().as_ref() == tag.split(':').last().unwrap_or(tag).as_bytes() {
                    inside_target_tag = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error parsing XML: {}", e)),
            _ => (),
        }
        buf.clear();
    }

    if value.is_empty() {
        Err(anyhow!("Tag {} not found or empty", tag))
    } else {
        Ok(value)
    }
}

/// Make a SOAP request to get current track information
async fn get_sonos_info(client: &reqwest::Client, base_url: &str) -> Result<SonosResponse> {
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
        .header(
            "SOAPAction",
            "\"urn:schemas-upnp-org:service:AVTransport:1#GetPositionInfo\"",
        )
        .header("Content-Type", "text/xml")
        .body(soap_body)
        .send()
        .await
        .context("Failed to send request to Sonos device")?;

    let response_text = response.text().await?;
    debug!("Raw SOAP response: {}", response_text);

    let reader = BufReader::new(response_text.as_bytes());
    let envelope: SoapEnvelope =
        quick_xml::de::from_reader(reader).context("Failed to parse SOAP envelope")?;

    let track_metadata = envelope.body.position_info_response.track_meta_data;

    let (title, artist, album) = if track_metadata.trim().is_empty() {
        debug!("Empty track metadata received");
        (None, None, None)
    } else {
        (
            extract_didl_value(&track_metadata, "dc:title").ok(),
            extract_didl_value(&track_metadata, "dc:creator").ok(),
            extract_didl_value(&track_metadata, "upnp:album").ok(),
        )
    };

    Ok(SonosResponse {
        title,
        artist,
        album,
        position: envelope.body.position_info_response.rel_time,
        duration: envelope.body.position_info_response.track_duration,
    })
}

async fn get_current_playback_state(
    client: &reqwest::Client,
    base_url: &str,
) -> Result<PlaybackState> {
    let sonos_response = get_sonos_info(client, base_url).await?;
    Ok(PlaybackState::from(sonos_response))
}
