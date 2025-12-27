//! Orchestrator - coordinates all personalization components

use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use crate::connectors::{AppState, GorseClient, RedisClient};
use crate::engine::{ContentRulesEngine, LeadScoringEngine};
use crate::models::{Recommendation, RuleAction};

/// Combined personalization response
#[derive(Debug, serde::Serialize)]
pub struct PersonalizationResult {
    pub user_id: String,
    pub lead_score: f64,
    pub segment: String,
    pub recommendations: Vec<Recommendation>,
    pub content_actions: Vec<RuleAction>,
}

pub struct Orchestrator {
    lead_scoring: LeadScoringEngine,
    content_rules: ContentRulesEngine,
    gorse: Arc<GorseClient>,
    redis: Arc<RedisClient>,
}

impl Orchestrator {
    pub fn new(state: &AppState) -> Self {
        Self {
            lead_scoring: LeadScoringEngine::new(
                state.clickhouse.clone(),
                state.mysql.clone(),
                state.redis.clone(),
            ),
            content_rules: ContentRulesEngine::new(state.mysql.clone()),
            gorse: state.gorse.clone(),
            redis: state.redis.clone(),
        }
    }

    /// Get full personalization for a user
    pub async fn personalize(
        &self,
        user_id: &str,
        page_url: Option<&str>,
        num_recommendations: u32,
    ) -> Result<PersonalizationResult> {
        // 1. Calculate lead score
        let lead_score = self.lead_scoring.calculate_score(user_id).await?;

        // 2. Get recommendations from Gorse
        let recommendations = self.get_recommendations_cached(user_id, num_recommendations).await?;

        // 3. Build context and evaluate content rules
        let context = ContentRulesEngine::build_context(
            user_id,
            Some(&serde_json::to_string(&lead_score.segment)?),
            Some(lead_score.score),
            page_url,
            None,
        );
        let content_actions = self.content_rules.evaluate(&context).await?;

        info!(
            user_id,
            score = lead_score.score,
            segment = ?lead_score.segment,
            num_recs = recommendations.len(),
            num_actions = content_actions.len(),
            "Personalization complete"
        );

        Ok(PersonalizationResult {
            user_id: user_id.to_string(),
            lead_score: lead_score.score,
            segment: serde_json::to_string(&lead_score.segment)?,
            recommendations,
            content_actions,
        })
    }

    /// Get recommendations with caching
    async fn get_recommendations_cached(
        &self,
        user_id: &str,
        n: u32,
    ) -> Result<Vec<Recommendation>> {
        let cache_key = RedisClient::recommendations_key(user_id);

        // Check cache
        if let Some(cached) = self.redis.get::<Vec<Recommendation>>(&cache_key).await? {
            return Ok(cached);
        }

        // Fetch from Gorse
        let recs = self.gorse.get_recommendations(user_id, n).await?;

        // Cache for 10 minutes
        self.redis.set(&cache_key, &recs, 600).await?;

        Ok(recs)
    }
}
