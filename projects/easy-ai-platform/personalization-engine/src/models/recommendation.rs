//! Recommendation models from Gorse

use serde::{Deserialize, Serialize};

/// Type of recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationType {
    Personalized,  // based on user history
    Popular,       // trending/popular items
    Similar,       // similar to viewed items
    Latest,        // newest items
}

/// Single recommendation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub item_id: String,
    pub score: f64,
    pub recommendation_type: RecommendationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Response from personalization API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationResponse {
    pub user_id: String,
    pub recommendations: Vec<Recommendation>,
    pub lead_score: Option<f64>,
    pub segment: Option<String>,
    pub content_actions: Vec<serde_json::Value>,
}
