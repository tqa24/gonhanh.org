//! Personalization Engine - Entry Point
//! High-performance personalization service for Easy AI Platform

use anyhow::Result;
use personalization_engine::{api, config::Config};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "personalization_engine=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;
    info!("Loaded configuration for environment: {}", config.environment);

    // Build and run the API server
    let app = api::create_router(&config).await?;

    let listener = tokio::net::TcpListener::bind(&config.server_addr()).await?;
    info!("Starting server on {}", config.server_addr());

    axum::serve(listener, app).await?;

    Ok(())
}
