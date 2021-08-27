use crate::{
    config::AppConfig,
    db,
    links::{Link, NewLink},
};
use anyhow::Result;
use axum::{
    extract::{Extension, Json},
    handler::{get, post},
    response::Redirect,
    AddExtensionLayer, Router, Server,
};
use sqlx::PgPool;
use std::{
    convert::TryInto,
    net::{IpAddr, SocketAddr},
};
use tower_http::trace::TraceLayer;
use tracing::{info, span};

#[allow(clippy::unused_async)]
async fn health_endpoint() -> &'static str {
    "OK"
}

async fn create_link(db: Extension<PgPool>, Json(payload): Json<NewLink>) -> Json<Link> {
    let link = payload
        .try_into()
        .expect("could not parse link payload as valid URL");

    let mut conn = db.acquire().await.expect("could not acquire DB conn");
    let inserted = Link::insert(&mut conn, link)
        .await
        .expect("could not insert link");

    Json(inserted)
}

async fn list_links(db: Extension<PgPool>) -> Json<Vec<Link>> {
    let mut conn = db.acquire().await.expect("could not acquire DB conn");
    if let Ok(links) = Link::list(&mut conn).await {
        Json(links)
    } else {
        Json(vec![])
    }
}

#[cfg_attr(debug_assertions, allow(clippy::unused_async))]
async fn visit_link(_db: Extension<PgPool>) -> Redirect {
    Redirect::temporary("https://www.google.com/".parse().unwrap())
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
