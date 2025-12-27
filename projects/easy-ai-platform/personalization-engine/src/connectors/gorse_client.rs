//! Gorse API client for recommendations

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::models::{Recommendation, RecommendationType};

pub struct GorseClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Deserialize)]
struct GorseItem {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Score")]
    score: f64,
}

#[derive(Debug, Serialize)]
struct GorseFeedback {
    #[serde(rename = "UserId")]
    user_id: String,
    #[serde(rename = "ItemId")]
    item_id: String,
    #[serde(rename = "FeedbackType")]
    feedback_type: String,
    #[serde(rename = "Timestamp")]
    timestamp: String,
}

impl GorseClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        }
    }

    /// Get personalized recommendations for a user
    pub async fn get_recommendations(&self, user_id: &str, n: u32) -> Result<Vec<Recommendation>> {
        let url = format!("{}/api/recommend/{}", self.base_url, user_id);

        let items: Vec<GorseItem> = self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&[("n", n.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(items.into_iter().map(|i| Recommendation {
            item_id: i.id,
            score: i.score,
            recommendation_type: RecommendationType::Personalized,
            metadata: None,
        }).collect())
    }

    /// Get popular items
    pub async fn get_popular(&self, n: u32) -> Result<Vec<Recommendation>> {
        let url = format!("{}/api/popular", self.base_url);

        let items: Vec<GorseItem> = self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&[("n", n.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(items.into_iter().map(|i| Recommendation {
            item_id: i.id,
            score: i.score,
            recommendation_type: RecommendationType::Popular,
            metadata: None,
        }).collect())
    }

    /// Get items similar to a given item
    pub async fn get_similar(&self, item_id: &str, n: u32) -> Result<Vec<Recommendation>> {
        let url = format!("{}/api/item/{}/neighbors", self.base_url, item_id);

        let items: Vec<GorseItem> = self.client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .query(&[("n", n.to_string())])
            .send()
            .await?
            .json()
            .await?;

        Ok(items.into_iter().map(|i| Recommendation {
            item_id: i.id,
            score: i.score,
            recommendation_type: RecommendationType::Similar,
            metadata: None,
        }).collect())
    }

    /// Submit user feedback (view, click, purchase, etc.)
    pub async fn insert_feedback(&self, user_id: &str, item_id: &str, feedback_type: &str) -> Result<()> {
        let url = format!("{}/api/feedback", self.base_url);

        let feedback = vec![GorseFeedback {
            user_id: user_id.to_string(),
            item_id: item_id.to_string(),
            feedback_type: feedback_type.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }];

        self.client
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&feedback)
            .send()
            .await?;

        Ok(())
    }
}
