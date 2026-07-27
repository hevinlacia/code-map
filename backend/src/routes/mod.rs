use axum::{Router, routing::get};

use crate::state::AppState;

pub mod health;
pub mod projects;
pub mod settings;
pub mod status;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/status", get(status::get_status))
        .route(
            "/settings",
            get(settings::get_settings).put(settings::put_settings),
        )
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
}
