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
        CreateProjectRequest, ProjectKind, ProjectRecord, ProjectSummary, canonical_project_path,
        expected_baseline_branch, infer_branch_name, infer_overlay_base_project_id,
        is_branch_missing, is_overlay_expired, is_worktree_project_path, project_constants_file,
        scan_branch_overlay, scan_project,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct ScanQueryParams {
    pub force: Option<bool>,
}

pub async fn list_projects(State(state): State<AppState>) -> AppResult<Json<Vec<ProjectSummary>>> {
    prune_branch_overlay_projects(&state)?;
    Ok(Json(
        state.projects().iter().map(ProjectSummary::from).collect(),
    ))
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

    let mut project = ProjectRecord::new(name.to_string(), root_path.clone());
    project.kind = payload
        .kind
        .clone()
        .unwrap_or_else(|| infer_project_kind(&root_path));
    project.baseline_branch = payload
        .baseline_branch
        .clone()
        .or_else(|| expected_baseline_branch(&root_path).map(str::to_string));
    project.branch_name = payload
        .branch_name
        .clone()
        .or_else(|| infer_branch_name(&root_path));

    if project.kind == ProjectKind::Baseline {
        validate_baseline_branch(&project)?;
    } else {
        project.base_project_id = payload
            .base_project_id
            .or_else(|| infer_overlay_base_project_id(&project, &projects));
        if project.base_project_id.is_none() {
            return Err(AppError::bad_request(
                "branch overlay requires an existing baseline project; add and scan the production branch first",
            ));
        }
    }

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
    let project = projects[index].clone();
    let scanned = if project.kind == ProjectKind::BranchOverlay {
        let base_project = project
            .base_project_id
            .and_then(|base_id| {
                projects
                    .iter()
                    .find(|candidate| candidate.id == base_id)
                    .cloned()
            })
            .ok_or_else(|| AppError::bad_request("branch overlay baseline project not found"))?;
        scan_branch_overlay(project, &base_project, &state.config.data_dir, force)?
    } else {
        validate_baseline_branch(&project)?;
        scan_project(project, &state.config.data_dir, force)?
    };
    let summary = ProjectSummary::from(&scanned);
    projects[index] = scanned;
    state.replace_projects(projects)?;

    Ok(Json(summary))
}

pub async fn delete_project(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    delete_project_by_id(&state, id)?;
    Ok(StatusCode::NO_CONTENT)
}

fn prune_branch_overlay_projects(state: &AppState) -> AppResult<()> {
    let projects = state.projects();
    let stale_overlay_ids = projects
        .iter()
        .filter(|project| project.kind == ProjectKind::BranchOverlay)
        .filter(|project| {
            !std::path::Path::new(&project.root_path).is_dir()
                || is_overlay_expired(project)
                || is_branch_missing(project)
        })
        .map(|project| project.id)
        .collect::<Vec<_>>();

    for id in stale_overlay_ids {
        delete_project_by_id(state, id)?;
    }
    Ok(())
}

fn infer_project_kind(root_path: &str) -> ProjectKind {
    if is_worktree_project_path(root_path) {
        ProjectKind::BranchOverlay
    } else {
        ProjectKind::Baseline
    }
}

fn validate_baseline_branch(project: &ProjectRecord) -> AppResult<()> {
    let Some(expected_branch) = project.baseline_branch.as_deref() else {
        return Ok(());
    };
    let current_branch = infer_branch_name(&project.root_path);
    if current_branch.as_deref() != Some(expected_branch) {
        return Err(AppError::bad_request(format!(
            "baseline project must be on production branch '{expected_branch}', current branch is '{}'",
            current_branch.unwrap_or_else(|| "unknown".to_string())
        )));
    }
    Ok(())
}

fn delete_project_by_id(state: &AppState, id: Uuid) -> AppResult<()> {
    let mut projects = state.projects();
    let project_index = projects
        .iter()
        .position(|project| project.id == id)
        .ok_or_else(|| AppError::not_found(format!("project not found: {id}")))?;

    projects.remove(project_index);
    state.replace_projects(projects)?;

    let mut runtime = state.runtime();
    if runtime.active_project_id == Some(id) {
        runtime.active_project_id = state.projects().first().map(|project| project.id);
        state.replace_runtime(runtime)?;
    }

    let notes = state
        .notes()
        .into_iter()
        .filter(|note| note.project_id != id)
        .collect::<Vec<_>>();
    state.replace_notes(notes)?;

    let constants_file = project_constants_file(&state.config.data_dir, id);
    match std::fs::remove_file(&constants_file) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(AppError::internal(error.to_string())),
    }

    Ok(())
}
