use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    features::projects::{
        CreateProjectRequest, ProjectRecord, ProjectSummary, canonical_project_path, scan_project,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ScanQueryParams {
    pub force: Option<bool>,
}

pub async fn list_projects(State(state): State<AppState>) -> Json<Vec<ProjectSummary>> {
    Json(state.projects().iter().map(ProjectSummary::from).collect())
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> AppResult<(StatusCode, Json<ProjectSummary>)> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("project name is required"));
    }

    let root_path = canonical_project_path(&payload.root_path)?;
    let mut projects = state.projects();

    if projects
        .iter()
        .any(|project| project.root_path == root_path)
    {
        return Err(AppError::bad_request(format!(
            "project already exists for root_path: {root_path}"
        )));
    }

    let project = ProjectRecord::new(name.to_string(), root_path);
    let summary = ProjectSummary::from(&project);
    projects.push(project.clone());
    state.replace_projects(projects)?;

    if state.runtime().active_project_id.is_none() {
        let mut runtime = state.runtime();
        runtime.active_project_id = Some(project.id);
        state.replace_runtime(runtime)?;
    }

    Ok((StatusCode::CREATED, Json(summary)))
}

pub async fn scan_project_by_id(
    Path(id): Path<Uuid>,
    Query(params): Query<ScanQueryParams>,
    State(state): State<AppState>,
) -> AppResult<Json<ProjectSummary>> {
    if !state.runtime().indexing_enabled {
        return Err(AppError::bad_request(
            "indexing is disabled; enable indexing before scanning",
        ));
    }

    let mut projects = state.projects();
    let index = projects
        .iter()
        .position(|project| project.id == id)
        .ok_or_else(|| AppError::not_found(format!("project not found: {id}")))?;

    let force = params.force.unwrap_or(false);
    let scanned = scan_project(projects[index].clone(), &state.config.data_dir, force)?;
    let summary = ProjectSummary::from(&scanned);
    projects[index] = scanned;
    state.replace_projects(projects)?;

    Ok(Json(summary))
}
