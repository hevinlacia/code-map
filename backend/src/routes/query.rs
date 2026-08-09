use axum::{Json, extract::State};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    features::{
        notes::{NoteRecord, matching_notes},
        projects::{
            IndexFreshness, ProjectKind, QueryRequest, QueryResponse, QueryResult,
            materialize_project, query_project, resolve_branch_view, scan_branch_overlay,
            scan_project,
        },
    },
    state::AppState,
};

pub async fn query(
    State(state): State<AppState>,
    Json(payload): Json<QueryRequest>,
) -> AppResult<Json<QueryResponse>> {
    if let Some(branch_name) = payload
        .branch_name
        .clone()
        .filter(|value| !value.trim().is_empty())
    {
        return query_branch_view(state, payload, &branch_name).map(Json);
    }

    query_single_project(state, payload).map(Json)
}

fn query_branch_view(
    state: AppState,
    payload: QueryRequest,
    branch_name: &str,
) -> AppResult<QueryResponse> {
    let runtime = state.runtime();
    if !runtime.indexing_enabled {
        return Err(AppError::bad_request(
            "indexing is disabled; enable indexing before querying a branch view",
        ));
    }

    let resolution = resolve_branch_view(
        state.projects(),
        &state.config.data_dir,
        branch_name,
        state.config.enable_write_actions,
    )?;
    state.replace_projects(resolution.projects.clone())?;

    let notes_source = state.notes();
    let mut terms = Vec::new();
    let mut notes = Vec::<NoteRecord>::new();
    let mut results = Vec::<QueryResult>::new();

    for project in &resolution.materialized_projects {
        let project_notes = matching_notes(&notes_source, project.id, &payload.query);
        notes.extend(project_notes.clone());
        let response = query_project(
            project,
            &payload.query,
            Some(50),
            runtime.max_summary_lines,
            project_notes,
        )?;
        if terms.is_empty() {
            terms = response.terms;
        }
        results.extend(response.results);
    }

    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| result_path(left).cmp(&result_path(right)))
    });
    let limit = payload
        .max_results
        .unwrap_or(12)
        .clamp(1, 50)
        .min(results.len());
    results.truncate(limit);

    let query = payload.query.trim().to_string();
    let project_name = format!("branch:{branch_name}");
    let summary_lines = branch_summary_lines(
        branch_name,
        &query,
        &results,
        resolution.materialized_projects.len(),
        runtime.max_summary_lines,
    );

    Ok(QueryResponse {
        project_id: Uuid::nil(),
        project_name,
        query,
        terms,
        result_count: results.len(),
        summary_lines,
        results,
        notes,
        index_freshness: IndexFreshness {
            current_git: None,
            last_scanned_git: None,
            index_stale: false,
            auto_refreshed: resolution.auto_refreshed,
            warning: None,
        },
    })
}

fn query_single_project(state: AppState, payload: QueryRequest) -> AppResult<QueryResponse> {
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

    let notes: Vec<NoteRecord> = matching_notes(&state.notes(), project_id, &payload.query);

    let mut response = query_project(
        &materialized,
        &payload.query,
        payload.max_results,
        runtime.max_summary_lines,
        notes,
    )?;
    response.index_freshness.auto_refreshed = auto_refreshed;

    Ok(response)
}

fn branch_summary_lines(
    branch_name: &str,
    query: &str,
    results: &[QueryResult],
    project_count: usize,
    max_summary_lines: u32,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "Branch view '{}' query '{}' returned {} candidate file(s) across {} materialized baseline/overlay index(es).",
        branch_name,
        query,
        results.len(),
        project_count
    ));

    let detail_limit = (max_summary_lines as usize).saturating_sub(2).min(8);
    for (index, result) in results.iter().take(detail_limit).enumerate() {
        let line_hint = result
            .snippets
            .first()
            .map(|snippet| format!(":{}", snippet.line))
            .or_else(|| {
                result
                    .symbols
                    .first()
                    .map(|symbol| format!(":{}", symbol.line))
            })
            .or_else(|| {
                result
                    .relationships
                    .first()
                    .map(|relationship| format!(":{}", relationship.line))
            })
            .unwrap_or_default();
        lines.push(format!(
            "{}. {}{} score={} reason={}",
            index + 1,
            result_path(result),
            line_hint,
            result.score,
            result
                .reasons
                .first()
                .cloned()
                .unwrap_or_else(|| "content match".to_string())
        ));
    }

    if results.is_empty() {
        lines.push("No matches found in this branch view.".to_string());
    } else {
        lines
            .push("Suggested agent action: read the top 3 repo/path line hints first.".to_string());
    }

    lines
}

fn result_path(result: &QueryResult) -> String {
    match &result.repo {
        Some(repo) if !repo.is_empty() => format!("{}:{}", repo, result.repo_relative_path),
        _ => result.relative_path.clone(),
    }
}
