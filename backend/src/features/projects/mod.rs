use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: Uuid,
    pub name: String,
    pub root_path: String,
    pub indexed: bool,
    pub last_indexed_at: Option<String>,
    pub symbol_count: u64,
    pub relationship_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub root_path: String,
}

impl ProjectSummary {
    pub fn sample(name: impl Into<String>, root_path: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            root_path: root_path.into(),
            indexed: false,
            last_indexed_at: None,
            symbol_count: 0,
            relationship_count: 0,
        }
    }
}
