//! Webhook handlers for external data sources

use axum::{http::StatusCode, Json};
use serde::Deserialize;
use tracing::info;

/// Rybbit event webhook payload
#[derive(Debug, Deserialize)]
pub struct RybbitEvent {
    pub user_id: String,
    pub event_type: String,
    pub page_url: String,
    pub timestamp: String,
    #[serde(default)]
    pub properties: serde_json::Value,
}

/// POST /webhook/rybbit
/// Handle Rybbit analytics events
pub async fn handle_rybbit(
    Json(events): Json<Vec<RybbitEvent>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(count = events.len(), "Received Rybbit events");

    // TODO: Process events
    // 1. Sync to Gorse as feedback
    // 2. Invalidate lead score cache for affected users

    Ok(Json(serde_json::json!({
        "status": "accepted",
        "processed": events.len()
    })))
}

/// CDP event webhook payload
#[derive(Debug, Deserialize)]
pub struct CdpEvent {
    pub user_id: String,
    pub event_type: String,
    #[serde(default)]
    pub data: serde_json::Value,
}

/// POST /webhook/cdp
/// Handle CDP customer events
pub async fn handle_cdp(
    Json(events): Json<Vec<CdpEvent>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    info!(count = events.len(), "Received CDP events");

    // TODO: Process events
    // 1. Update customer data
    // 2. Recalculate lead scores

    Ok(Json(serde_json::json!({
        "status": "accepted",
        "processed": events.len()
    })))
}
