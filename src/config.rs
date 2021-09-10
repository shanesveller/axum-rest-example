#![allow(clippy::module_name_repetitions)]

//! Strongly-typed configuration details for application behavior which may be
//! decided at runtime
//!
//! Runtime calculations are performed via the [`config`] crate. This includes
//! the ability to layer per-environment configuration files in TOML format as
//! well as just-in-time overrides via well-named environment variables.

use std::net::Ipv4Addr;

use config::{Config, ConfigError, Environment, File};
use secrecy::Secret;
use serde::Deserialize;

/// The root configuration object, holding all available configuration details
/// as inner public fields
#[derive(Debug, Default, Deserialize)]
pub struct AppConfig {
    /// Configuration pertaining specifically to database connections,
    /// interactions, and authentication
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Configuration pertaining specifically to the app's exposed REST API
    #[serde(default)]
    pub http: HttpConfig,
    /// Configuration pertaining specifically to observability
    #[serde(default)]
    pub telemetry: TelemetryConfig,
}

impl AppConfig {
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

/// Configuration pertaining specifically to database connections, interactions,
/// and authentication
///
/// Uses the [`secrecy`] crate to help protect sensitive values from being
/// leaked to log output, stacktraces, etc.
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

/// Configuration pertaining specifically to the app's exposed REST API
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct HttpConfig {
    /// The default IPv4 address to bind the application to, defaulting to `0.0.0.0`
    #[serde(default = "default_listen_address")]
    pub listen_address: Ipv4Addr,
    /// The default TCP port to bind the application to, defaulting to `8080`
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

/// Available, named presets for logging style, corresponding closely to
/// [`mod@tracing_subscriber::fmt`]'s available choices.
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

/// Configuration pertaining specifically to observability
#[derive(Clone, Debug, Default, Deserialize)]
pub struct TelemetryConfig {
    /// Should an [`opentelemetry`] stack be initialized with the application as an active [`tracing`] subscriber/layer?
    #[serde(default)]
    pub opentelemetry: bool,
    /// A remote endpoint to report to when `opentelemetry` field is set to true
    /// - assumes a gRPC listener compatible with OTLP protocol, see
    /// [`crate::telemetry`] for details
    #[serde(default)]
    pub opentelemetry_endpoint: Option<String>,
    /// Select a named logging preset from [`LogFormat`]
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
