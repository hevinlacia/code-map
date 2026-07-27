use axum::{Json, extract::State};

use crate::{
    error::{AppError, AppResult},
    features::{
        notes::{NoteRecord, matching_notes},
        projects::{QueryRequest, QueryResponse, query_project},
    },
    state::AppState,
};

pub async fn query(
    State(state): State<AppState>,
    Json(payload): Json<QueryRequest>,
) -> AppResult<Json<QueryResponse>> {
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

    let notes: Vec<NoteRecord> = matching_notes(&state.notes(), project_id, &payload.query);

    Ok(Json(query_project(
        project,
        &payload.query,
        payload.max_results,
        runtime.max_summary_lines,
        notes,
    )?))
}
