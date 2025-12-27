//! Database and external service connectors

mod clickhouse_client;
mod gorse_client;
mod mysql_client;
mod redis_client;

pub use clickhouse_client::ClickHouseClient;
pub use gorse_client::GorseClient;
pub use mysql_client::MySqlClient;
pub use redis_client::RedisClient;

use crate::Config;
use anyhow::Result;
use std::sync::Arc;

/// Shared application state containing all connectors
#[derive(Clone)]
pub struct AppState {
    pub clickhouse: Arc<ClickHouseClient>,
    pub mysql: Arc<MySqlClient>,
    pub redis: Arc<RedisClient>,
    pub gorse: Arc<GorseClient>,
}

impl AppState {
    pub async fn new(config: &Config) -> Result<Self> {
        let clickhouse = Arc::new(ClickHouseClient::new(&config.database.clickhouse_url)?);
        let mysql = Arc::new(MySqlClient::new(&config.database.mysql_url).await?);
        let redis = Arc::new(RedisClient::new(&config.redis.url).await?);
        let gorse = Arc::new(GorseClient::new(&config.gorse.url, &config.gorse.api_key));

        Ok(Self {
            clickhouse,
            mysql,
            redis,
            gorse,
        })
    }
}
