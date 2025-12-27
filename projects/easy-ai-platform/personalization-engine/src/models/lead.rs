//! Lead scoring models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Lead segment classification based on score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LeadSegment {
    Cold,       // score < 20
    Warm,       // 20 <= score < 50
    Hot,        // 50 <= score < 80
    SalesReady, // score >= 80
}

impl LeadSegment {
    pub fn from_score(score: f64) -> Self {
        match score {
            s if s >= 80.0 => Self::SalesReady,
            s if s >= 50.0 => Self::Hot,
            s if s >= 20.0 => Self::Warm,
            _ => Self::Cold,
        }
    }
}

/// Factors used to calculate lead score
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoringFactors {
    pub page_views_last_7d: i32,
    pub time_on_site_avg_seconds: i32,
    pub form_submissions: i32,
    pub pricing_page_visits: i32,
    pub demo_requests: i32,
    pub email_opens: i32,
    pub email_clicks: i32,
    pub return_visits: i32,
}

impl ScoringFactors {
    /// Calculate total lead score based on weighted factors
    pub fn calculate_score(&self) -> f64 {
        let score = (self.page_views_last_7d as f64 * 0.5)
            + (self.time_on_site_avg_seconds as f64 / 60.0 * 1.0) // convert to minutes
            + (self.form_submissions as f64 * 5.0)
            + (self.pricing_page_visits as f64 * 3.0)
            + (self.demo_requests as f64 * 10.0)
            + (self.email_opens as f64 * 0.5)
            + (self.email_clicks as f64 * 1.0)
            + (self.return_visits as f64 * 2.0);

        score.min(100.0) // cap at 100
    }
}

/// Complete lead score record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeadScore {
    pub id: i64,
    pub user_id: String,
    pub score: f64,
    pub segment: LeadSegment,
    pub factors: ScoringFactors,
    pub calculated_at: DateTime<Utc>,
}

impl LeadScore {
    pub fn new(user_id: String, factors: ScoringFactors) -> Self {
        let score = factors.calculate_score();
        Self {
            id: 0,
            user_id,
            score,
            segment: LeadSegment::from_score(score),
            factors,
            calculated_at: Utc::now(),
        }
    }
}
