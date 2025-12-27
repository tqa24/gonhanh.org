//! Redis connector for caching

use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands};
use serde::{de::DeserializeOwned, Serialize};

pub struct RedisClient {
    conn: ConnectionManager,
}

impl RedisClient {
    pub async fn new(url: &str) -> Result<Self> {
        let client = redis::Client::open(url)?;
        let conn = ConnectionManager::new(client).await?;
        Ok(Self { conn })
    }

    /// Get cached value
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.conn.clone();
        let value: Option<String> = conn.get(key).await?;
        match value {
            Some(v) => Ok(Some(serde_json::from_str(&v)?)),
            None => Ok(None),
        }
    }

    /// Set cached value with TTL in seconds
    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(value)?;
        conn.set_ex(key, json, ttl_seconds).await?;
        Ok(())
    }

    /// Delete cached value
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        conn.del(key).await?;
        Ok(())
    }

    /// Cache key for lead score
    pub fn lead_score_key(user_id: &str) -> String {
        format!("lead_score:{}", user_id)
    }

    /// Cache key for recommendations
    pub fn recommendations_key(user_id: &str) -> String {
        format!("recs:{}", user_id)
    }
}
