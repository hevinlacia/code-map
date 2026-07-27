use axum::{Json, extract::State};

use crate::{
    error::{AppError, AppResult},
    features::projects::{NeighborsRequest, NeighborsResponse, neighbors_project},
    state::AppState,
};

pub async fn neighbors(
    State(state): State<AppState>,
    Json(payload): Json<NeighborsRequest>,
) -> AppResult<Json<NeighborsResponse>> {
    let runtime = state.runtime();
    let projects = state.projects();
    let project_id = payload
        .project_id
        .or(runtime.active_project_id)
        .or_else(|| {
            if projects.len() == 1 {
                Some(projects[0].id)
            } else {
                None
            }
        });

    let project_id = project_id.ok_or_else(|| {
        AppError::bad_request("project_id is required when no active or single project exists")
    })?;

    let project = projects
        .iter()
        .find(|project| project.id == project_id)
        .ok_or_else(|| AppError::not_found(format!("project not found: {project_id}")))?;

    Ok(Json(neighbors_project(project, &payload.entity)?))
}
