//! Content personalization rules models

use serde::{Deserialize, Serialize};

/// Condition operators for rule matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    In,
}

/// Single condition in a content rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub field: String,           // e.g., "segment", "page_views", "country"
    pub operator: ConditionOperator,
    pub value: serde_json::Value, // flexible value type
}

/// Action to take when rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    ShowContent { content_id: String },
    HideElement { selector: String },
    Redirect { url: String },
    SetVariable { key: String, value: serde_json::Value },
    TriggerWebhook { url: String, payload: serde_json::Value },
}

/// Content personalization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentRule {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub conditions: Vec<RuleCondition>,  // AND logic within rule
    pub actions: Vec<RuleAction>,
    pub priority: i32,                   // higher = evaluated first
    pub is_active: bool,
}

impl ContentRule {
    /// Check if all conditions match the given context
    pub fn matches(&self, context: &serde_json::Value) -> bool {
        if !self.is_active {
            return false;
        }

        self.conditions.iter().all(|cond| {
            let field_value = context.get(&cond.field);
            match (&cond.operator, field_value) {
                (ConditionOperator::Equals, Some(v)) => v == &cond.value,
                (ConditionOperator::NotEquals, Some(v)) => v != &cond.value,
                (ConditionOperator::GreaterThan, Some(v)) => {
                    v.as_f64().zip(cond.value.as_f64())
                        .map(|(a, b)| a > b)
                        .unwrap_or(false)
                }
                (ConditionOperator::LessThan, Some(v)) => {
                    v.as_f64().zip(cond.value.as_f64())
                        .map(|(a, b)| a < b)
                        .unwrap_or(false)
                }
                (ConditionOperator::Contains, Some(v)) => {
                    v.as_str().zip(cond.value.as_str())
                        .map(|(a, b)| a.contains(b))
                        .unwrap_or(false)
                }
                (ConditionOperator::In, Some(v)) => {
                    cond.value.as_array()
                        .map(|arr| arr.contains(v))
                        .unwrap_or(false)
                }
                _ => false,
            }
        })
    }
}
