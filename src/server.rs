use crate::{
    config::AppConfig,
    db,
    links::{Link, NewLink, NewLinkError},
};
use anyhow::Result;
use axum::{
    body::{Bytes, Full},
    extract::{self, Extension, Json},
    handler::{get, post},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    AddExtensionLayer, Router, Server,
};
use serde_json::json;
use sqlx::PgPool;
use std::{
    convert::{Infallible, TryInto},
    net::{IpAddr, SocketAddr},
};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, span};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("error creating link")]
    NewLinkError(#[from] NewLinkError),
    #[error("database error")]
    SqlError(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    type Body = Full<Bytes>;

    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
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

#[allow(clippy::unused_async)]
async fn health_endpoint() -> &'static str {
    "OK"
}

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

#[instrument(skip(db))]
async fn list_links(db: Extension<PgPool>) -> Result<Json<Vec<Link>>, AppError> {
    let mut conn = db.acquire().await?;
    if let Ok(links) = Link::list(&mut conn).await {
        Ok(links.into())
    } else {
        Ok(Json(vec![]))
    }
}

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

pub async fn launch(config: &AppConfig) -> Result<()> {
    let root_span = span!(tracing::Level::TRACE, "app_start");
    let _enter = root_span.enter();

    let pool = db::new_pool(config).await?;

    let app = Router::new()
        .route("/:slug", get(visit_link))
        .route("/health", get(health_endpoint))
        .route("/v1/link", post(create_link))
        .route("/v1/links", get(list_links))
        // TODO: Omit health check from request logging
        .layer(AddExtensionLayer::new(pool))
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
