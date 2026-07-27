use std::{env, net::SocketAddr, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub data_dir: PathBuf,
    pub frontend_dir: PathBuf,
    pub default_workspace: Option<PathBuf>,
    pub enable_write_actions: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("CODE_MAP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("CODE_MAP_PORT")
                .ok()
                .and_then(|value| value.parse().ok())
                .unwrap_or(18765),
            data_dir: env::var("CODE_MAP_DATA_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./data")),
            frontend_dir: env::var("CODE_MAP_FRONTEND_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./frontend/dist")),
            default_workspace: env::var("CODE_MAP_DEFAULT_WORKSPACE")
                .ok()
                .map(PathBuf::from),
            enable_write_actions: env::var("CODE_MAP_ENABLE_WRITE_ACTIONS")
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
                .unwrap_or(false),
        }
    }

    pub fn socket_addr(&self) -> anyhow::Result<SocketAddr> {
        format!("{}:{}", self.host, self.port)
            .parse()
            .map_err(Into::into)
    }
}
