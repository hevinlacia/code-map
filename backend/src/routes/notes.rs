use axum::{Json, extract::State};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    features::notes::{CreateNoteRequest, ListNotesRequest, NoteRecord, validate_note},
    state::AppState,
};

pub async fn list_notes(
    State(state): State<AppState>,
    Json(payload): Json<ListNotesRequest>,
) -> AppResult<Json<Vec<NoteRecord>>> {
    let runtime = state.runtime();
    let project_id = payload
        .project_id
        .or(runtime.active_project_id)
        .or_else(|| {
            let projects = state.projects();
            if projects.len() == 1 {
                Some(projects[0].id)
            } else {
                None
            }
        });

    let notes = state.notes();
    let filtered: Vec<NoteRecord> = notes
        .into_iter()
        .filter(|note| project_id.is_none_or(|id| note.project_id == id))
        .filter(|note| {
            payload
                .query
                .as_deref()
                .map(|query| {
                    let needle = query.trim().to_lowercase();
                    if needle.is_empty() {
                        return true;
                    }
                    let hay = note.query.to_lowercase();
                    hay.contains(&needle) || needle.contains(&hay)
                })
                .unwrap_or(true)
        })
        .take(50)
        .collect();

    Ok(Json(filtered))
}

pub async fn create_note(
    State(state): State<AppState>,
    Json(payload): Json<CreateNoteRequest>,
) -> AppResult<Json<NoteRecord>> {
    validate_note(&payload)?;
    let runtime = state.runtime();
    let project_id = payload
        .project_id
        .or(runtime.active_project_id)
        .ok_or_else(|| {
            AppError::bad_request("project_id is required when no active project exists")
        })?;

    let projects = state.projects();
    if !projects.iter().any(|project| project.id == project_id) {
        return Err(AppError::not_found(format!(
            "project not found: {project_id}"
        )));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string();
    let note = NoteRecord {
        id: Uuid::new_v4(),
        project_id,
        query: payload.query.trim().to_string(),
        summary: payload.summary.trim().to_string(),
        pointers: payload.pointers,
        created_at: now,
    };

    let mut notes = state.notes();
    notes.push(note.clone());
    state.replace_notes(notes)?;

    Ok(Json(note))
}
