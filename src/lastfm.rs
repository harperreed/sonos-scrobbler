use anyhow::{Context, Result};
use log::{error, info};
use rustfm_scrobble::{Scrobble, Scrobbler};
use std::env;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LastFmError {
    #[error("Missing Last.fm credentials in environment")]
    MissingCredentials(String),
    #[error("Authentication failed")]
    AuthenticationError(String),
}

/// Represents authenticated Last.fm credentials
pub struct LastFmAuth {
    username: String,
    password: String,
    api_key: String,
    api_secret: String,
}

impl LastFmAuth {
    /// Create a new LastFmAuth instance from environment variables
    pub fn from_env() -> Result<Self> {
        let username = env::var("LASTFM_USERNAME")
            .context("LASTFM_USERNAME not found in environment")?;
        let password = env::var("LASTFM_PASSWORD")
            .context("LASTFM_PASSWORD not found in environment")?;
        let api_key = env::var("LASTFM_API_KEY")
            .context("LASTFM_API_KEY not found in environment")?;
        let api_secret = env::var("LASTFM_API_SECRET")
            .context("LASTFM_API_SECRET not found in environment")?;

        Ok(Self {
            username,
            password,
            api_key,
            api_secret,
        })
    }
}

pub struct LastFm {
    scrobbler: Scrobbler,
}

impl LastFm {
    /// Create and authenticate a new Last.fm session
    pub async fn new() -> Result<Self> {
        info!("Initializing Last.fm connection...");
        
        // Load credentials
        let auth = LastFmAuth::from_env()?;
        
        // Create scrobbler
        let mut scrobbler = Scrobbler::new(&auth.api_key, &auth.api_secret);
        
        // Authenticate
        match scrobbler.authenticate_with_password(&auth.username, &auth.password) {
            Ok(_) => {
                info!("Successfully authenticated with Last.fm as {}", auth.username);
                Ok(Self { scrobbler })
            }
            Err(e) => {
                error!("Failed to authenticate with Last.fm: {}", e);
                Err(LastFmError::AuthenticationError(e.to_string()).into())
            }
        }
    }

    /// Scrobble a track to Last.fm
    pub async fn scrobble(&self, artist: &str, title: &str, album: Option<&str>) -> Result<()> {
        let scrobble = if let Some(album_name) = album {
            Scrobble::new(artist, title, album_name)
        } else {
            Scrobble::new(artist, title, "")
        };

        match self.scrobbler.scrobble(&scrobble) {
            Ok(_) => {
                info!("Scrobbled: {} - {}", artist, title);
                Ok(())
            }
            Err(e) => {
                error!("Failed to scrobble track: {}", e);
                Err(anyhow::anyhow!("Scrobbling failed: {}", e))
            }
        }
    }
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
