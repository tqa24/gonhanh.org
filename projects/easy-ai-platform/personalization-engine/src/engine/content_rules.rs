//! Content rules engine for dynamic personalization

use std::sync::Arc;
use anyhow::Result;
use tracing::debug;

use crate::connectors::MySqlClient;
use crate::models::{ContentRule, RuleAction};

pub struct ContentRulesEngine {
    mysql: Arc<MySqlClient>,
}

impl ContentRulesEngine {
    pub fn new(mysql: Arc<MySqlClient>) -> Self {
        Self { mysql }
    }

    /// Evaluate all rules against user context and return matching actions
    pub async fn evaluate(&self, context: &serde_json::Value) -> Result<Vec<RuleAction>> {
        let rules = self.mysql.get_active_rules().await?;
        let mut actions = Vec::new();

        for rule in rules {
            if rule.matches(context) {
                debug!(rule_name = %rule.name, "Rule matched");
                actions.extend(rule.actions);
            }
        }

        Ok(actions)
    }

    /// Build user context from various data sources
    pub fn build_context(
        user_id: &str,
        segment: Option<&str>,
        lead_score: Option<f64>,
        page_url: Option<&str>,
        custom_data: Option<&serde_json::Value>,
    ) -> serde_json::Value {
        let mut ctx = serde_json::json!({
            "user_id": user_id,
        });

        if let Some(s) = segment {
            ctx["segment"] = serde_json::json!(s);
        }
        if let Some(score) = lead_score {
            ctx["lead_score"] = serde_json::json!(score);
        }
        if let Some(url) = page_url {
            ctx["page_url"] = serde_json::json!(url);
        }
        if let Some(data) = custom_data {
            if let Some(obj) = data.as_object() {
                for (k, v) in obj {
                    ctx[k] = v.clone();
                }
            }
        }

        ctx
    }
}
