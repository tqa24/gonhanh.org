//! ClickHouse connector for Rybbit analytics data

use anyhow::Result;
use clickhouse::Client;
use serde::Deserialize;

pub struct ClickHouseClient {
    client: Client,
}

/// User behavior event from Rybbit
#[derive(Debug, Clone, Deserialize, clickhouse::Row)]
pub struct UserEvent {
    pub user_id: String,
    pub event_type: String,
    pub page_url: String,
    pub timestamp: i64,
    pub session_id: String,
    pub duration_seconds: Option<i32>,
}

/// Aggregated user behavior stats
#[derive(Debug, Clone, Deserialize, clickhouse::Row)]
pub struct UserBehaviorStats {
    pub user_id: String,
    pub page_views_7d: i32,
    pub avg_session_duration: i32,
    pub pricing_page_visits: i32,
    pub return_visits: i32,
}

impl ClickHouseClient {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::default().with_url(url);
        Ok(Self { client })
    }

    /// Get aggregated behavior stats for a user (last 7 days)
    pub async fn get_user_behavior_stats(&self, user_id: &str) -> Result<Option<UserBehaviorStats>> {
        let query = r#"
            SELECT
                user_id,
                count(*) as page_views_7d,
                avg(duration_seconds) as avg_session_duration,
                countIf(page_url LIKE '%pricing%') as pricing_page_visits,
                uniq(session_id) - 1 as return_visits
            FROM events
            WHERE user_id = ?
              AND timestamp > now() - INTERVAL 7 DAY
            GROUP BY user_id
        "#;

        let result = self.client
            .query(query)
            .bind(user_id)
            .fetch_optional::<UserBehaviorStats>()
            .await?;

        Ok(result)
    }

    /// Get recent events for a user
    pub async fn get_user_events(&self, user_id: &str, limit: u32) -> Result<Vec<UserEvent>> {
        let query = r#"
            SELECT user_id, event_type, page_url, timestamp, session_id, duration_seconds
            FROM events
            WHERE user_id = ?
            ORDER BY timestamp DESC
            LIMIT ?
        "#;

        let events = self.client
            .query(query)
            .bind(user_id)
            .bind(limit)
            .fetch_all::<UserEvent>()
            .await?;

        Ok(events)
    }
}
