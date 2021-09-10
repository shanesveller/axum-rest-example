#![allow(clippy::module_name_repetitions)]

use std::net::Ipv4Addr;

use config::{Config, ConfigError, Environment, File};
use secrecy::Secret;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub http: HttpConfig,
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
    #[warn(dead_code)]
    pub fn new() -> Result<Self, ConfigError> {
        let mut config = Config::default();

        config.merge(File::with_name("config/default"))?;

        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        config.merge(File::with_name(&format!("config/{}", env)).required(false))?;

        config.merge(File::with_name("config/local").required(false))?;

        config.merge(Environment::with_prefix("APP").separator("__"))?;

        config.try_into()
    }
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    #[serde(default = "default_database_url")]
    pub url: Secret<String>,
}

fn default_database_url() -> Secret<String> {
    Secret::new("postgresql://postgres:postgres@localhost/axum_rest_example_dev".to_owned())
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: 128,
            // TODO: number of CPU cores/worker threads
            min_connections: 8,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 900,
            max_lifetime_seconds: 3600,
            url: default_database_url(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "default_listen_address")]
    pub listen_address: Ipv4Addr,
    #[serde(default = "default_port")]
    pub listen_port: u16,
}

fn default_listen_address() -> Ipv4Addr {
    Ipv4Addr::new(0, 0, 0, 0)
}

fn default_port() -> u16 {
    8080
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0".parse().unwrap(),
            listen_port: 8080,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Full,
    Pretty,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Full
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub opentelemetry: bool,
    #[serde(default)]
    pub opentelemetry_endpoint: Option<String>,
    #[serde(default)]
    pub log_format: LogFormat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_prefix() {
        std::env::set_var("APP_TELEMETRY__OPENTELEMETRY", "true");
        std::env::set_var("APP_TELEMETRY__LOG_FORMAT", "json");
        let config = AppConfig::new().unwrap();

        assert!(config.telemetry.opentelemetry);
        assert_eq!(config.telemetry.log_format, LogFormat::Json);

        std::env::remove_var("APP_TELEMETRY__OPENTELEMETRY");
        std::env::remove_var("APP_TELEMETRY__LOG_FORMAT");
    }
}
