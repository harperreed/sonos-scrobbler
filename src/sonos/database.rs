use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TrackDatabase {
    pool: SqlitePool,
}

impl TrackDatabase {
    pub async fn new() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite:tracks.db").await?;
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                device_name TEXT NOT NULL,
                track_info TEXT NOT NULL,
                played_at INTEGER NOT NULL,
                UNIQUE(device_name, track_info, played_at)
            )"
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn log_track(&self, device_name: &str, track_info: &str) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs() as i64;

        // Check if we've logged this track in the last hour
        let recent_play = sqlx::query(
            "SELECT 1 FROM tracks 
             WHERE device_name = ? 
             AND track_info = ? 
             AND played_at > ?"
        )
        .bind(device_name)
        .bind(track_info)
        .bind(now - 3600) // Last hour
        .fetch_optional(&self.pool)
        .await?;

        if recent_play.is_some() {
            return Ok(false);
        }

        sqlx::query(
            "INSERT INTO tracks (device_name, track_info, played_at) 
             VALUES (?, ?, ?)"
        )
        .bind(device_name)
        .bind(track_info)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(true)
    }

    pub async fn get_last_track(&self, device_name: &str) -> Result<Option<String>> {
        let record = sqlx::query(
            "SELECT track_info FROM tracks 
             WHERE device_name = ? 
             ORDER BY played_at DESC 
             LIMIT 1"
        )
        .bind(device_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record.map(|row| row.get(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_database_operations() {
        let db = TrackDatabase::new().await.unwrap();
        
        // Test logging a track
        let logged = db.log_track("Test Device", "Test Track").await.unwrap();
        assert!(logged);

        // Test getting last track
        let last_track = db.get_last_track("Test Device").await.unwrap();
        assert_eq!(last_track, Some("Test Track".to_string()));

        // Test duplicate prevention
        let logged_again = db.log_track("Test Device", "Test Track").await.unwrap();
        assert!(!logged_again);
    }
}
