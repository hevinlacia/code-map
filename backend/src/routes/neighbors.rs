use axum::{Json, extract::State};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    features::projects::{
        IndexFreshness, NeighborHit, NeighborsRequest, NeighborsResponse, ProjectKind,
        materialize_project, neighbors_project, resolve_branch_view, scan_branch_overlay,
        scan_project,
    },
    state::AppState,
};

pub async fn neighbors(
    State(state): State<AppState>,
    Json(payload): Json<NeighborsRequest>,
) -> AppResult<Json<NeighborsResponse>> {
    if let Some(branch_name) = payload
        .branch_name
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        return neighbors_branch_view(state, payload, &branch_name).map(Json);
    }

    neighbors_single_project(state, payload).map(Json)
}

fn neighbors_branch_view(
    state: AppState,
    payload: NeighborsRequest,
    branch_name: &str,
) -> AppResult<NeighborsResponse> {
    let runtime = state.runtime();
    if !runtime.indexing_enabled {
        return Err(AppError::bad_request(
            "indexing is disabled; enable indexing before resolving a branch view",
        ));
    }

    let resolution = resolve_branch_view(
        state.projects(),
        &state.config.data_dir,
        branch_name,
        state.config.enable_write_actions,
    )?;
    state.replace_projects(resolution.projects)?;

    let mut definitions = Vec::new();
    let mut producers = Vec::new();
    let mut consumers = Vec::new();
    let mut readers = Vec::new();
    let mut writers = Vec::new();
    let mut callers = Vec::new();

    for project in &resolution.materialized_projects {
        let response = neighbors_project(project, &payload.entity)?;
        definitions.extend(response.definitions);
        producers.extend(response.producers);
        consumers.extend(response.consumers);
        readers.extend(response.readers);
        writers.extend(response.writers);
        callers.extend(response.callers);
    }

    for list in [
        &mut definitions,
        &mut producers,
        &mut consumers,
        &mut readers,
        &mut writers,
        &mut callers,
    ] {
        dedupe_hits(list);
        list.truncate(25);
    }

    Ok(NeighborsResponse {
        project_id: Uuid::nil(),
        project_name: format!("branch:{branch_name}"),
        entity: payload.entity,
        definitions,
        producers,
        consumers,
        readers,
        writers,
        callers,
        index_freshness: IndexFreshness {
            current_git: None,
            last_scanned_git: None,
            index_stale: false,
            auto_refreshed: resolution.auto_refreshed,
            warning: None,
        },
    })
}

fn neighbors_single_project(
    state: AppState,
    payload: NeighborsRequest,
) -> AppResult<NeighborsResponse> {
    let runtime = state.runtime();
    let mut projects = state.projects();
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

    let project_index = projects
        .iter()
        .position(|project| project.id == project_id)
        .ok_or_else(|| AppError::not_found(format!("project not found: {project_id}")))?;

    let mut auto_refreshed = false;
    let project = if runtime.auto_refresh_enabled && runtime.indexing_enabled {
        let current = projects[project_index].clone();
        let scanned = if current.kind == ProjectKind::BranchOverlay {
            let base_project = current
                .base_project_id
                .and_then(|base_id| {
                    projects
                        .iter()
                        .find(|project| project.id == base_id)
                        .cloned()
                })
                .ok_or_else(|| {
                    AppError::bad_request("branch overlay baseline project not found")
                })?;
            scan_branch_overlay(current, &base_project, &state.config.data_dir, false)?
        } else {
            scan_project(current, &state.config.data_dir, false)?
        };
        projects[project_index] = scanned.clone();
        state.replace_projects(projects.clone())?;
        auto_refreshed = true;
        scanned
    } else {
        let mut current = projects[project_index].clone();
        if current.kind == ProjectKind::BranchOverlay {
            current.last_used_at = Some(crate::features::projects::now_unix_string());
            projects[project_index] = current.clone();
            state.replace_projects(projects.clone())?;
        }
        current
    };
    let base_project = project
        .base_project_id
        .and_then(|base_id| projects.iter().find(|candidate| candidate.id == base_id));
    let materialized = materialize_project(&project, base_project);

    let mut response = neighbors_project(&materialized, &payload.entity)?;
    response.index_freshness.auto_refreshed = auto_refreshed;

    Ok(response)
}

fn dedupe_hits(hits: &mut Vec<NeighborHit>) {
    let mut seen = Vec::<(Option<String>, String, u64, String, String)>::new();
    hits.retain(|hit| {
        let key = (
            hit.repo.clone(),
            hit.repo_relative_path.clone(),
            hit.line,
            hit.kind.clone(),
            hit.name.clone(),
        );
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}
