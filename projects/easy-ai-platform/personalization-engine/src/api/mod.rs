//! REST API routes for Personalization Engine

mod personalize;
mod admin;
mod webhooks;

use axum::{routing::{get, post}, Router};
use tower_http::trace::TraceLayer;
use anyhow::Result;

use crate::Config;
use crate::connectors::AppState;

/// Create the main API router
pub async fn create_router(config: &Config) -> Result<Router> {
    let state = AppState::new(config).await?;

    let app = Router::new()
        // Health check
        .route("/health", get(health))
        // Personalization API
        .route("/api/v1/personalize/:user_id", get(personalize::get_personalization))
        .route("/api/v1/recommend/:user_id", get(personalize::get_recommendations))
        .route("/api/v1/lead-score/:user_id", get(personalize::get_lead_score))
        .route("/api/v1/lead-score/:user_id", post(personalize::calculate_lead_score))
        // Admin API
        .route("/api/v1/admin/rules", get(admin::list_rules))
        .route("/api/v1/admin/rules", post(admin::create_rule))
        // Webhooks
        .route("/webhook/rybbit", post(webhooks::handle_rybbit))
        .route("/webhook/cdp", post(webhooks::handle_cdp))
        // State and middleware
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    Ok(app)
}

async fn health() -> &'static str {
    "ok"
}
