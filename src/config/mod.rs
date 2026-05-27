pub mod defaults;

use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

use self::defaults::*;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub refresh_interval_secs: u64,
    pub default_sort: String,
    pub show_established: bool,
    pub project_markers: Vec<String>,
    pub watched_ports: Vec<u16>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval_secs: DEFAULT_REFRESH_INTERVAL_SECS,
            default_sort: DEFAULT_SORT.to_string(),
            show_established: DEFAULT_SHOW_ESTABLISHED,
            project_markers: DEFAULT_PROJECT_MARKERS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            watched_ports: Vec::new(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read config from {}", path.display()))?;
            let config: Config = toml::from_str(&content)
                .with_context(|| format!("failed to parse config from {}", path.display()))?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("whoseportisitanyway")
            .join("config.toml")
    }
}
