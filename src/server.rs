//! [`axum`]-specific logic for offering a REST API

use crate::{
    config::AppConfig,
    db,
    links::{Link, NewLink, NewLinkError},
};
use anyhow::Result;
use axum::{
    body::BoxBody,
    extract::{self, Extension, Json},
    http::{Request, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    AddExtensionLayer, Router, Server,
};
use hyper::Body;
use serde_json::json;
use sqlx::PgPool;
use std::{
    convert::TryInto,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::signal::unix::{signal, SignalKind};
use tower_http::trace::TraceLayer;
use tracing::{debug_span, field, info, instrument, span, Span};

/// Wrapper Error enum used to provide a consistent [`IntoResponse`] target for
/// request handlers that return inner domain Error types.
#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("error creating link")]
    NewLinkError(#[from] NewLinkError),
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NewLinkError(_) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "could not create link")
            }
            AppError::SqlError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "database error"),
        };

        let body = Json(json!({ "error": message }));

        (status, body).into_response()
    }
}

/// GET handler for health requests by an application platform
///
/// Intended for use in environments such as Amazon ECS or Kubernetes which want
/// to validate that the HTTP service is available for traffic, by returning a
/// 200 OK response with any content.
#[allow(clippy::unused_async)]
async fn health_endpoint() -> &'static str {
    "OK"
}

/// POST handler for creating new [`Link`]s
///
/// Extracts a [`NewLink`] from the request body as a JSON payload, and if
/// valid, generates and inserts a [`Link`] into the database. Returns the
/// inserted `Link` as the response body.
#[instrument(skip(db))]
async fn create_link(
    db: Extension<PgPool>,
    Json(payload): Json<NewLink>,
) -> Result<(StatusCode, Json<Link>), AppError> {
    let link = payload.try_into()?;

    let mut conn = db.acquire().await?;
    let inserted = Link::insert(&mut conn, link).await?;

    Ok((StatusCode::CREATED, inserted.into()))
}

/// GET handler which lists all previously recorded [`Link`]s without any limits
///
/// Returns a static ordering as determined by [`Link::list`].
#[instrument(skip(db))]
async fn list_links(db: Extension<PgPool>) -> Result<Json<Vec<Link>>, AppError> {
    let mut conn = db.acquire().await?;
    if let Ok(links) = Link::list(&mut conn).await {
        Ok(links.into())
    } else {
        Ok(Json(vec![]))
    }
}

/// GET handler which fetches a [`Link`] and redirects to its `destination` URL
///
/// Redirects to own `/` if no matching `hash` is found.
#[instrument(skip(db))]
async fn visit_link(
    db: Extension<PgPool>,
    extract::Path(hash): extract::Path<String>,
) -> Result<Redirect, AppError> {
    let mut conn = db.acquire().await?;

    Link::get_by_hash(&mut conn, &hash).await?.map_or_else(
        || Ok(Redirect::temporary("/".parse().unwrap())),
        |link| Ok(Redirect::temporary(link.destination.parse().unwrap())),
    )
}

/// Internal helper for [`tower_http::trace::TraceLayer`] to create
/// [`tracing::Span`]s around a request.
fn make_span(_request: &Request<Body>) -> Span {
    #[cfg(feature = "otel")]
    {
        debug_span!(
            "http-request",
            request_duration = tracing::field::Empty,
            status_code = tracing::field::Empty,
            traceID = tracing::field::Empty,
        )
    }
    #[cfg(not(feature = "otel"))]
    {
        debug_span!(
            "http-request",
            request_duration = tracing::field::Empty,
            status_code = tracing::field::Empty,
        )
    }
}

/// Internal helper for [`tower_http::trace::TraceLayer`] to emit a structured [`tracing::Span`] with specific recorded fields.
///
/// Uses a `Loki`-friendly traceID that can correlate to `Tempo` distributed traces.
fn emit_response_trace_with_id(response: &Response<BoxBody>, latency: Duration, span: &Span) {
    #[cfg(feature = "otel")]
    {
        // https://github.com/kube-rs/controller-rs/blob/b99ad0bfbf4ae75f03323bff2796572d4257bd96/src/telemetry.rs#L4-L8
        use opentelemetry::trace::TraceContextExt;
        use tracing_opentelemetry::OpenTelemetrySpanExt;
        let trace_id = span.context().span().span_context().trace_id().to_hex();
        span.record("traceID", &field::display(&trace_id));
    }

    span.record("request_duration", &field::display(latency.as_micros()));
    span.record("status_code", &field::display(response.status().as_u16()));

    tracing::debug!("response generated");
}

/// Opens an HTTP server on the indicated address and port from an [`AppConfig`].
///
/// Relies on [`axum::Server`] for the primary behavior. Also launches a
/// [`tokio::signal`]-based task to listen for OS kill signals to allow
/// in-flight requests to finish first, via
/// [`axum::Server::with_graceful_shutdown`]. Currently also comprehensively
/// defines all HTTP routes.
pub async fn launch(config: &AppConfig) -> Result<()> {
    let root_span = span!(tracing::Level::TRACE, "app_start");
    let _enter = root_span.enter();

    let pool = db::new_pool(config).await?;

    let app = Router::new()
        .route("/:slug", get(visit_link))
        .route("/health", get(health_endpoint))
        .route("/v1/link", post(create_link))
        .route("/v1/links", get(list_links))
        .layer(AddExtensionLayer::new(pool))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(make_span)
                .on_response(emit_response_trace_with_id),
        );

    let addr = SocketAddr::new(
        IpAddr::V4(config.http.listen_address),
        config.http.listen_port,
    );

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    let signal_handler = tokio::spawn(async {
        tokio::pin! {
          let interrupt = signal(SignalKind::interrupt()).expect("could not open SIGINT channel");
          let quit = signal(SignalKind::quit()).expect("could not open SIGQUIT channel");
          let term = signal(SignalKind::terminate()).expect("could not open SIGTERM channel");
        };

        loop {
            tokio::select! {
              _ = (&mut interrupt).recv() => {
                  info!("SIGINT received");
                  break;
              }
              _ = (&mut quit).recv() => {
                  info!("SIGQUIT received");
                  break;
              }
              _ = (&mut term).recv() => {
                  info!("SIGTERM received");
                  break;
              }
            }
        }

        shutdown_tx
            .send(())
            .expect("could not send shutdown signal");
    });

    info!(port = ?addr.port(), address = ?addr.ip(), "Listening on http://{}/", addr);
    info!("Waiting for SIGTERM/SIGQUIT for graceful shutdown");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        })
        .await
        .expect("could not launch HTTP server on port 8080");

    signal_handler
        .await
        .expect("error with shutdown handler task");

    Ok(())
}
