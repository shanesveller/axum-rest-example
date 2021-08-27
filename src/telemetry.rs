#[cfg(feature = "otel")]
use std::time::Duration;

use anyhow::Result;
#[cfg(feature = "otel")]
use opentelemetry::sdk::Resource;
#[cfg(feature = "otel")]
use opentelemetry::KeyValue;
#[cfg(feature = "otel")]
use opentelemetry_otlp::{Protocol, WithExportConfig};
use tracing_subscriber::{fmt::format::FmtSpan, layer::SubscriberExt, EnvFilter, Registry};

use crate::config::{AppConfig, LogFormat};

pub fn init(config: &AppConfig) -> Result<()> {
    #[cfg(feature = "otel")]
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(
                    config
                        .telemetry
                        .opentelemetry_endpoint
                        .as_ref()
                        .expect("opentelemetry_endpoint not configured"),
                )
                .with_timeout(Duration::from_secs(3))
                .with_protocol(Protocol::Grpc),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "axum_rest_example",
            )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    #[cfg(feature = "otel")]
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    #[cfg(not(feature = "otel"))]
    let telemetry = tracing_opentelemetry::layer();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap()
        .add_directive("hyper=info".parse()?);

    let subscriber = Registry::default()
        .with(config.telemetry.opentelemetry.then(|| telemetry))
        .with(
            (config.telemetry.log_format == LogFormat::Full)
                .then(|| tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE)),
        )
        .with((config.telemetry.log_format == LogFormat::Pretty).then(|| {
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
        }))
        .with((config.telemetry.log_format == LogFormat::Json).then(|| {
            tracing_subscriber::fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
        }))
        .with(filter_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
