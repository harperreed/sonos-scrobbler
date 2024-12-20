use anyhow::{Context, Result};
use log::{info, warn};
use rustfm_scrobble::Scrobbler;
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LastFmError {
    #[error("Missing Last.fm credentials in environment")]
    MissingCredentials(String),
}

/// Represents authenticated Last.fm credentials
pub struct LastFmAuth {
    api_key: String,
    api_secret: String,
    session_key: String,
}

impl LastFmAuth {
    /// Create a new LastFmAuth instance from environment variables
    pub fn from_env() -> Result<Self> {
        let api_key = env::var("LASTFM_API_KEY")
            .context("LASTFM_API_KEY not found in environment")?;
        let api_secret = env::var("LASTFM_API_SECRET")
            .context("LASTFM_API_SECRET not found in environment")?;
        let session_key = env::var("LASTFM_SESSION_KEY")
            .context("LASTFM_SESSION_KEY not found in environment")?;

        Ok(Self {
            api_key,
            api_secret,
            session_key,
        })
    }
}

/// Authenticate with Last.fm using credentials from environment
pub async fn authenticate() -> Result<Scrobbler> {
    info!("Authenticating with Last.fm...");
    
    // Load credentials
    let auth = LastFmAuth::from_env()?;
    
    // Create and authenticate scrobbler
    let scrobbler = Scrobbler::new(auth.api_key, auth.api_secret);
    
    // Verify we have a valid session key
    if auth.session_key.is_empty() {
        warn!("No Last.fm session key found. Please authenticate at:");
        warn!("https://www.last.fm/api/auth");
        return Err(LastFmError::MissingCredentials(
            "Session key required".to_string(),
        ))?;
    }

    info!("Successfully authenticated with Last.fm");
    Ok(scrobbler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_credentials() {
        // Clear any existing env vars
        env::remove_var("LASTFM_API_KEY");
        env::remove_var("LASTFM_API_SECRET");
        env::remove_var("LASTFM_SESSION_KEY");

        let result = LastFmAuth::from_env();
        assert!(result.is_err());
    }
}
