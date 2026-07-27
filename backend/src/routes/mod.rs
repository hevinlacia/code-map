use axum::{
    Router,
    routing::{get, post},
};

use crate::state::AppState;

pub mod health;
pub mod neighbors;
pub mod notes;
pub mod projects;
pub mod query;
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
        .route("/projects/{id}/scan", post(projects::scan_project_by_id))
        .route("/query", post(query::query))
        .route("/neighbors", post(neighbors::neighbors))
        .route("/notes", get(notes::list_notes).post(notes::create_note))
}
