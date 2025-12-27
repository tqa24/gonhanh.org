//! Admin API endpoints for managing rules and experiments

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::connectors::AppState;
use crate::models::{ContentRule, RuleAction, RuleCondition};

/// GET /api/v1/admin/rules
/// List all content rules
pub async fn list_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<ContentRule>>, (StatusCode, String)> {
    let rules = state.mysql
        .get_active_rules()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(rules))
}

#[derive(Debug, Deserialize)]
pub struct CreateRuleRequest {
    pub name: String,
    pub description: Option<String>,
    pub conditions: Vec<RuleCondition>,
    pub actions: Vec<RuleAction>,
    #[serde(default)]
    pub priority: i32,
}

/// POST /api/v1/admin/rules
/// Create a new content rule
pub async fn create_rule(
    State(_state): State<AppState>,
    Json(payload): Json<CreateRuleRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // TODO: Implement create rule in MySQL
    // For now, return the payload as confirmation
    Ok(Json(serde_json::json!({
        "status": "created",
        "rule": {
            "name": payload.name,
            "description": payload.description,
            "priority": payload.priority,
        }
    })))
}
