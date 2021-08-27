use crate::config::AppConfig;
use anyhow::Result;
use axum::{handler::get, AddExtensionLayer, Router, Server};
use std::net::{IpAddr, SocketAddr};
use tower_http::trace::TraceLayer;
use tracing::{info, span};

async fn health_endpoint() -> &'static str {
    "OK"
}

pub async fn launch(config: &AppConfig) -> Result<()> {
    let root_span = span!(tracing::Level::TRACE, "app_start");
    let _enter = root_span.enter();

    let app = Router::new()
        .route("/health", get(health_endpoint))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port,
    );

    info!(port = ?addr.port(), address = ?addr.ip(), "Listening on http://{}/", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("could not launch HTTP server on port 8080");

    Ok(())
}
