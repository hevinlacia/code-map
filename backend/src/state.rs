use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use serde::{Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    features::{notes::NoteRecord, projects::ProjectRecord},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    runtime: Arc<RwLock<RuntimeState>>,
    projects: Arc<RwLock<Vec<ProjectRecord>>>,
    notes: Arc<RwLock<Vec<NoteRecord>>>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
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
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        fs::create_dir_all(&config.data_dir)?;
        let runtime = read_json_or_default(&settings_file(&config.data_dir))?;
        let mut projects: Vec<ProjectRecord> =
            read_json_or_default(&projects_file(&config.data_dir))?;
        if projects.is_empty()
            && let Some(default_workspace) = &config.default_workspace
            && default_workspace.is_dir()
        {
            let root_path = fs::canonicalize(default_workspace)?;
            projects.push(ProjectRecord::new(
                "Default workspace",
                root_path.display().to_string(),
            ));
            write_json_atomic(&projects_file(&config.data_dir), &projects)?;
        }
        let notes: Vec<NoteRecord> = read_json_or_default(&notes_file(&config.data_dir))?;

        Ok(Self {
            config,
            runtime: Arc::new(RwLock::new(runtime)),
            projects: Arc::new(RwLock::new(projects)),
            notes: Arc::new(RwLock::new(notes)),
        })
    }

    pub fn runtime(&self) -> RuntimeState {
        self.runtime.read().expect("runtime state poisoned").clone()
    }

    pub fn replace_runtime(&self, runtime: RuntimeState) -> AppResult<RuntimeState> {
        {
            let mut guard = self.runtime.write().expect("runtime state poisoned");
            *guard = runtime.clone();
        }
        self.persist_runtime()?;
        Ok(runtime)
    }

    pub fn projects(&self) -> Vec<ProjectRecord> {
        self.projects
            .read()
            .expect("projects state poisoned")
            .clone()
    }

    pub fn replace_projects(&self, projects: Vec<ProjectRecord>) -> AppResult<Vec<ProjectRecord>> {
        {
            let mut guard = self.projects.write().expect("projects state poisoned");
            *guard = projects.clone();
        }
        self.persist_projects()?;
        Ok(projects)
    }

    pub fn notes(&self) -> Vec<NoteRecord> {
        self.notes.read().expect("notes state poisoned").clone()
    }

    pub fn replace_notes(&self, notes: Vec<NoteRecord>) -> AppResult<Vec<NoteRecord>> {
        {
            let mut guard = self.notes.write().expect("notes state poisoned");
            *guard = notes.clone();
        }
        write_json_atomic(&notes_file(&self.config.data_dir), &notes)?;
        Ok(notes)
    }

    pub fn persist_runtime(&self) -> AppResult<()> {
        write_json_atomic(&settings_file(&self.config.data_dir), &self.runtime())
    }

    pub fn persist_projects(&self) -> AppResult<()> {
        write_json_atomic(&projects_file(&self.config.data_dir), &self.projects())
    }
}

fn settings_file(data_dir: &Path) -> PathBuf {
    data_dir.join("settings.json")
}

fn projects_file(data_dir: &Path) -> PathBuf {
    data_dir.join("projects.json")
}

fn notes_file(data_dir: &Path) -> PathBuf {
    data_dir.join("notes.json")
}

fn read_json_or_default<T>(path: &Path) -> anyhow::Result<T>
where
    T: DeserializeOwned + Default,
{
    match fs::read_to_string(path) {
        Ok(contents) => Ok(serde_json::from_str(&contents)?),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(T::default()),
        Err(error) => Err(error.into()),
    }
}

fn write_json_atomic<T>(path: &Path, value: &T) -> AppResult<()>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| AppError::internal(error.to_string()))?;
    }

    let tmp_path = path.with_extension("json.tmp");
    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| AppError::internal(error.to_string()))?;
    fs::write(&tmp_path, contents).map_err(|error| AppError::internal(error.to_string()))?;
    fs::rename(&tmp_path, path).map_err(|error| AppError::internal(error.to_string()))?;
    Ok(())
}
