//! MySQL connector for CDP customer data

use anyhow::Result;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};

use crate::models::{ContentRule, LeadScore, ScoringFactors};

pub struct MySqlClient {
    pool: Pool<MySql>,
}

impl MySqlClient {
    pub async fn new(url: &str) -> Result<Self> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    /// Get lead score for a user
    pub async fn get_lead_score(&self, user_id: &str) -> Result<Option<LeadScore>> {
        let row = sqlx::query_as!(
            LeadScoreRow,
            r#"SELECT id, user_id, score, segment, factors, calculated_at
               FROM lead_scores WHERE user_id = ? ORDER BY calculated_at DESC LIMIT 1"#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    /// Save or update lead score
    pub async fn upsert_lead_score(&self, score: &LeadScore) -> Result<()> {
        let factors_json = serde_json::to_string(&score.factors)?;
        let segment_str = serde_json::to_string(&score.segment)?;

        sqlx::query!(
            r#"INSERT INTO lead_scores (user_id, score, segment, factors, calculated_at)
               VALUES (?, ?, ?, ?, ?)
               ON DUPLICATE KEY UPDATE
                 score = VALUES(score),
                 segment = VALUES(segment),
                 factors = VALUES(factors),
                 calculated_at = VALUES(calculated_at)"#,
            score.user_id,
            score.score,
            segment_str,
            factors_json,
            score.calculated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get active content rules ordered by priority
    pub async fn get_active_rules(&self) -> Result<Vec<ContentRule>> {
        let rows = sqlx::query_as!(
            ContentRuleRow,
            r#"SELECT id, name, description, conditions, actions, priority, is_active
               FROM content_rules WHERE is_active = true ORDER BY priority DESC"#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Get customer data from CDP
    pub async fn get_customer_data(&self, user_id: &str) -> Result<Option<serde_json::Value>> {
        let row = sqlx::query_scalar!(
            r#"SELECT data as "data: serde_json::Value" FROM customers WHERE user_id = ?"#,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.flatten())
    }
}

// Internal row types for sqlx mapping
#[derive(Debug)]
struct LeadScoreRow {
    id: i64,
    user_id: String,
    score: f64,
    segment: String,
    factors: String,
    calculated_at: chrono::DateTime<chrono::Utc>,
}

impl From<LeadScoreRow> for LeadScore {
    fn from(row: LeadScoreRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            score: row.score,
            segment: serde_json::from_str(&row.segment).unwrap_or_default(),
            factors: serde_json::from_str(&row.factors).unwrap_or_default(),
            calculated_at: row.calculated_at,
        }
    }
}

#[derive(Debug)]
struct ContentRuleRow {
    id: i64,
    name: String,
    description: Option<String>,
    conditions: String,
    actions: String,
    priority: i32,
    is_active: bool,
}

impl From<ContentRuleRow> for ContentRule {
    fn from(row: ContentRuleRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            conditions: serde_json::from_str(&row.conditions).unwrap_or_default(),
            actions: serde_json::from_str(&row.actions).unwrap_or_default(),
            priority: row.priority,
            is_active: row.is_active,
        }
    }
}
