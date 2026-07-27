use axum::{Json, extract::State};

use crate::{
    error::AppResult,
    state::{AppState, RuntimeState},
};

pub async fn get_settings(State(state): State<AppState>) -> Json<RuntimeState> {
    Json(state.runtime())
}

pub async fn put_settings(
    State(state): State<AppState>,
    Json(mut payload): Json<RuntimeState>,
) -> AppResult<Json<RuntimeState>> {
    if payload.query_token_budget < 200 {
        return Err(crate::error::AppError::bad_request(
            "query_token_budget must be at least 200",
        ));
    }

    if payload.max_summary_lines == 0 || payload.max_summary_lines > 500 {
        return Err(crate::error::AppError::bad_request(
            "max_summary_lines must be between 1 and 500",
        ));
    }

    if !state.config.enable_write_actions {
        payload.auto_refresh_enabled = false;
    }

    Ok(Json(state.replace_runtime(payload)))
}
