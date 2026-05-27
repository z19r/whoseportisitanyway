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

#[cfg(test)]
mod tests {
    use super::defaults::*;
    use super::*;

    #[test]
    fn default_config_values() {
        let cfg = Config::default();
        assert_eq!(cfg.refresh_interval_secs, DEFAULT_REFRESH_INTERVAL_SECS);
        assert_eq!(cfg.default_sort, DEFAULT_SORT);
        assert_eq!(cfg.show_established, DEFAULT_SHOW_ESTABLISHED);
        assert!(cfg.watched_ports.is_empty());
    }

    #[test]
    fn default_project_markers_not_empty() {
        let cfg = Config::default();
        assert!(!cfg.project_markers.is_empty());
        assert!(cfg.project_markers.contains(&"package.json".to_string()));
        assert!(cfg.project_markers.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn deserialize_partial_toml() {
        let toml_str = r#"
            refresh_interval_secs = 5
            watched_ports = [3000, 8080]
        "#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.refresh_interval_secs, 5);
        assert_eq!(cfg.watched_ports, vec![3000, 8080]);
        assert_eq!(cfg.default_sort, DEFAULT_SORT);
        assert_eq!(cfg.show_established, DEFAULT_SHOW_ESTABLISHED);
    }

    #[test]
    fn deserialize_full_toml() {
        let toml_str = r#"
            refresh_interval_secs = 10
            default_sort = "pid"
            show_established = true
            project_markers = ["go.mod"]
            watched_ports = [443]
        "#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.refresh_interval_secs, 10);
        assert_eq!(cfg.default_sort, "pid");
        assert!(cfg.show_established);
        assert_eq!(cfg.project_markers, vec!["go.mod"]);
        assert_eq!(cfg.watched_ports, vec![443]);
    }

    #[test]
    fn deserialize_empty_toml() {
        let cfg: Config = toml::from_str("").unwrap();
        assert_eq!(cfg.refresh_interval_secs, DEFAULT_REFRESH_INTERVAL_SECS);
    }

    #[test]
    fn load_returns_default_when_no_file() {
        let cfg = Config::load().unwrap();
        assert_eq!(cfg.refresh_interval_secs, DEFAULT_REFRESH_INTERVAL_SECS);
    }

    #[test]
    fn config_path_ends_with_config_toml() {
        let path = Config::config_path();
        assert!(path.ends_with("whoseportisitanyway/config.toml"));
    }
}
