use anyhow::Result;
use axum_rest_example::config::AppConfig;

fn main() -> Result<()> {
    let _config = AppConfig::new()?;
    println!("Hello, world!");
    Ok(())
}
