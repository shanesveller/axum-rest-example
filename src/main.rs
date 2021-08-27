use anyhow::Result;
use axum_rest_example::{config::AppConfig, server, telemetry};
use tracing::debug;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::new()?;
    telemetry::init(&config)?;
    debug!(?config);
    server::launch(&config).await?;
    Ok(())
}
