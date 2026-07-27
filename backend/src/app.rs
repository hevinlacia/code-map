use axum::{Router, routing::get};
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

use crate::{config::AppConfig, routes, state::AppState};

pub fn build_router(config: AppConfig) -> anyhow::Result<Router> {
    let frontend_dir = config.frontend_dir.clone();
    let state = AppState::new(config)?;

    let static_files = ServeDir::new(&frontend_dir)
        .not_found_service(ServeFile::new(frontend_dir.join("index.html")));

    Ok(Router::new()
        .route("/health", get(routes::health::health))
        .nest("/api", routes::api_router())
        .fallback_service(static_files)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state))
}
