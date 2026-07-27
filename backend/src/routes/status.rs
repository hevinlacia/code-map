use axum::{Json, extract::State};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub service: &'static str,
    pub indexing_enabled: bool,
    pub auto_refresh_enabled: bool,
    pub active_project_id: Option<String>,
    pub query_token_budget: u32,
    pub max_summary_lines: u32,
}

pub async fn get_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let runtime = state.runtime();

    Json(StatusResponse {
        service: "code-map",
        indexing_enabled: runtime.indexing_enabled,
        auto_refresh_enabled: runtime.auto_refresh_enabled,
        active_project_id: runtime.active_project_id.map(|id| id.to_string()),
        query_token_budget: runtime.query_token_budget,
        max_summary_lines: runtime.max_summary_lines,
    })
}
