use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub query: String,
    pub summary: String,
    pub pointers: Vec<NotePointer>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotePointer {
    pub repo: Option<String>,
    pub path: String,
    pub line: u64,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateNoteRequest {
    pub project_id: Option<Uuid>,
    pub query: String,
    pub summary: String,
    #[serde(default)]
    pub pointers: Vec<NotePointer>,
}

#[derive(Debug, Deserialize)]
pub struct ListNotesRequest {
    pub project_id: Option<Uuid>,
    pub query: Option<String>,
}

/// Notes whose query fuzzy-matches the search term (case-insensitive substring
/// either direction). Returns at most 5 to keep agent context small.
pub fn matching_notes(notes: &[NoteRecord], project_id: Uuid, query: &str) -> Vec<NoteRecord> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    notes
        .iter()
        .filter(|note| note.project_id == project_id)
        .filter(|note| {
            let hay = note.query.to_lowercase();
            hay.contains(&needle) || needle.contains(&hay)
        })
        .take(5)
        .cloned()
        .collect()
}

pub fn validate_note(request: &CreateNoteRequest) -> AppResult<()> {
    if request.query.trim().is_empty() {
        return Err(AppError::bad_request("query is required"));
    }
    if request.summary.trim().is_empty() {
        return Err(AppError::bad_request("summary is required"));
    }
    Ok(())
}
