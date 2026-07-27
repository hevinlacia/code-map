use tracing_subscriber::{EnvFilter, fmt};

pub fn init() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("code_map_backend=info,tower_http=info"));

    fmt().with_env_filter(filter).compact().init();
}
