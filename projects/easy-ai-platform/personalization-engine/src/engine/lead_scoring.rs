//! Lead scoring engine

use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use crate::connectors::{ClickHouseClient, MySqlClient, RedisClient};
use crate::models::{LeadScore, ScoringFactors};

pub struct LeadScoringEngine {
    clickhouse: Arc<ClickHouseClient>,
    mysql: Arc<MySqlClient>,
    redis: Arc<RedisClient>,
}

impl LeadScoringEngine {
    pub fn new(
        clickhouse: Arc<ClickHouseClient>,
        mysql: Arc<MySqlClient>,
        redis: Arc<RedisClient>,
    ) -> Self {
        Self { clickhouse, mysql, redis }
    }

    /// Calculate and store lead score for a user
    pub async fn calculate_score(&self, user_id: &str) -> Result<LeadScore> {
        // Check cache first
        let cache_key = RedisClient::lead_score_key(user_id);
        if let Some(cached) = self.redis.get::<LeadScore>(&cache_key).await? {
            info!(user_id, "Lead score cache hit");
            return Ok(cached);
        }

        // Get behavior data from ClickHouse
        let behavior = self.clickhouse.get_user_behavior_stats(user_id).await?;

        // Get customer data from MySQL (CDP)
        let customer_data = self.mysql.get_customer_data(user_id).await?;

        // Build scoring factors
        let factors = self.build_factors(behavior, customer_data);
        let score = LeadScore::new(user_id.to_string(), factors);

        // Save to MySQL
        self.mysql.upsert_lead_score(&score).await?;

        // Cache for 5 minutes
        self.redis.set(&cache_key, &score, 300).await?;

        info!(user_id, score = score.score, segment = ?score.segment, "Lead score calculated");
        Ok(score)
    }

    /// Build scoring factors from various data sources
    fn build_factors(
        &self,
        behavior: Option<crate::connectors::clickhouse_client::UserBehaviorStats>,
        customer_data: Option<serde_json::Value>,
    ) -> ScoringFactors {
        let mut factors = ScoringFactors::default();

        // From ClickHouse behavior data
        if let Some(b) = behavior {
            factors.page_views_last_7d = b.page_views_7d;
            factors.time_on_site_avg_seconds = b.avg_session_duration;
            factors.pricing_page_visits = b.pricing_page_visits;
            factors.return_visits = b.return_visits;
        }

        // From CDP customer data
        if let Some(data) = customer_data {
            if let Some(form_count) = data.get("form_submissions").and_then(|v| v.as_i64()) {
                factors.form_submissions = form_count as i32;
            }
            if let Some(demo) = data.get("demo_requests").and_then(|v| v.as_i64()) {
                factors.demo_requests = demo as i32;
            }
            if let Some(opens) = data.get("email_opens").and_then(|v| v.as_i64()) {
                factors.email_opens = opens as i32;
            }
            if let Some(clicks) = data.get("email_clicks").and_then(|v| v.as_i64()) {
                factors.email_clicks = clicks as i32;
            }
        }

        factors
    }

    /// Invalidate cached lead score (call when user data changes)
    pub async fn invalidate_cache(&self, user_id: &str) -> Result<()> {
        let cache_key = RedisClient::lead_score_key(user_id);
        self.redis.delete(&cache_key).await
    }
}
