use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    runtime: Arc<RwLock<RuntimeState>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeState {
    pub indexing_enabled: bool,
    pub auto_refresh_enabled: bool,
    pub active_project_id: Option<Uuid>,
    pub query_token_budget: u32,
    pub max_summary_lines: u32,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            indexing_enabled: true,
            auto_refresh_enabled: false,
            active_project_id: None,
            query_token_budget: 2_000,
            max_summary_lines: 50,
        }
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            runtime: Arc::new(RwLock::new(RuntimeState::default())),
        }
    }

    pub fn runtime(&self) -> RuntimeState {
        self.runtime.read().expect("runtime state poisoned").clone()
    }

    pub fn replace_runtime(&self, runtime: RuntimeState) -> RuntimeState {
        let mut guard = self.runtime.write().expect("runtime state poisoned");
        *guard = runtime.clone();
        runtime
    }
}
