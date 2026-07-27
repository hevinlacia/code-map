use axum::{Json, extract::State, http::StatusCode};

use crate::{
    error::AppResult,
    features::projects::{CreateProjectRequest, ProjectSummary},
    state::AppState,
};

pub async fn list_projects(State(state): State<AppState>) -> Json<Vec<ProjectSummary>> {
    let root = state
        .config
        .default_workspace
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "Configure CODE_MAP_DEFAULT_WORKSPACE".to_string());

    Json(vec![ProjectSummary::sample("Default workspace", root)])
}

pub async fn create_project(
    State(_state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<ProjectSummary>)> {
    let name = payload.name.trim();
    let root_path = payload.root_path.trim();

    if name.is_empty() {
        return Err(crate::error::AppError::bad_request(
            "project name is required",
        ));
    }

    if root_path.is_empty() {
        return Err(crate::error::AppError::bad_request("root_path is required"));
    }

    Ok((
        StatusCode::CREATED,
        Json(ProjectSummary::sample(
            name.to_string(),
            root_path.to_string(),
        )),
    ))
}
