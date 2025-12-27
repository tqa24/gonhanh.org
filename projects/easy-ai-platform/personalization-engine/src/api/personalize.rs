//! Personalization API endpoints

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::connectors::AppState;
use crate::engine::Orchestrator;
use crate::models::Recommendation;

#[derive(Debug, Deserialize)]
pub struct PersonalizeQuery {
    page_url: Option<String>,
    #[serde(default = "default_n")]
    n: u32,
}

fn default_n() -> u32 { 10 }

/// GET /api/v1/personalize/:user_id
/// Get full personalization for a user
pub async fn get_personalization(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<PersonalizeQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let orchestrator = Orchestrator::new(&state);

    let result = orchestrator
        .personalize(&user_id, query.page_url.as_deref(), query.n)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::to_value(result).unwrap()))
}

/// GET /api/v1/recommend/:user_id
/// Get only recommendations for a user
pub async fn get_recommendations(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<PersonalizeQuery>,
) -> Result<Json<Vec<Recommendation>>, (StatusCode, String)> {
    let recs = state.gorse
        .get_recommendations(&user_id, query.n)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(recs))
}

/// GET /api/v1/lead-score/:user_id
/// Get cached lead score for a user
pub async fn get_lead_score(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let score = state.mysql
        .get_lead_score(&user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match score {
        Some(s) => Ok(Json(serde_json::to_value(s).unwrap())),
        None => Err((StatusCode::NOT_FOUND, "Lead score not found".to_string())),
    }
}

/// POST /api/v1/lead-score/:user_id
/// Recalculate lead score for a user
pub async fn calculate_lead_score(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use crate::engine::LeadScoringEngine;

    let engine = LeadScoringEngine::new(
        state.clickhouse.clone(),
        state.mysql.clone(),
        state.redis.clone(),
    );

    let score = engine
        .calculate_score(&user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::to_value(score).unwrap()))
}
