use anyhow::Result;
use axum_rest_example::{config::AppConfig, telemetry};
use tracing::info;

fn main() -> Result<()> {
    let config = AppConfig::new()?;
    telemetry::init(&config)?;
    info!("Hello, world!");
    Ok(())
}
