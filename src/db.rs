use std::time::Duration;

use crate::config::AppConfig;
use secrecy::ExposeSecret;
use sqlx::{pool::PoolOptions, PgPool};

pub async fn new_pool(config: &AppConfig) -> sqlx::Result<PgPool> {
    PoolOptions::new()
        .connect_timeout(Duration::from_secs(config.database.connect_timeout_seconds))
        .max_connections(config.database.max_connections)
        // TODO: CPU core count and/or test thread count?
        .min_connections(config.database.min_connections)
        .max_lifetime(Duration::from_secs(config.database.max_lifetime_seconds))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout_seconds))
        .connect(config.database.url.expose_secret())
        .await
}
