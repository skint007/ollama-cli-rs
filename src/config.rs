use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const DEFAULT_URL: &str = "https://ollama.local.skint007.dev";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub default_url: String,
    pub active_profile: Option<String>,
    #[serde(default)]
    pub urls: BTreeMap<String, String>,
}

fn default_url() -> String {
    DEFAULT_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_url: DEFAULT_URL.to_string(),
            active_profile: None,
            urls: BTreeMap::new(),
        }
    }
}

impl Config {
    /// Get the config file path (~/.config/ollama-cli/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "ollama-cli") {
            let config_dir = proj_dirs.config_dir();
            Ok(config_dir.join("config.toml"))
        } else {
            let home = dirs_home().context("Cannot determine home directory")?;
            Ok(home.join(".config").join("ollama-cli").join("config.toml"))
        }
    }

    /// Load config from TOML file, migrating from bash format if needed
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let contents =
                std::fs::read_to_string(&config_path).context("Failed to read config file")?;
            let config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            return Ok(config);
        }

        // Check for legacy bash config and migrate
        let legacy_path = dirs_home()
            .map(|h| h.join(".ollama-cli.conf"))
            .unwrap_or_default();

        if legacy_path.exists() {
            let config = Self::migrate_from_bash(&legacy_path)?;
            config.save()?;
            eprintln!(
                "Migrated config from {} to {}",
                legacy_path.display(),
                config_path.display()
            );
            return Ok(config);
        }

        Ok(Config::default())
    }

    /// Save config to TOML file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&config_path, contents).context("Failed to write config file")?;
        Ok(())
    }

    /// Resolve the effective URL: CLI override > active profile > default
    pub fn resolve_url(&self, cli_override: Option<&str>) -> String {
        if let Some(url) = cli_override {
            return url.to_string();
        }
        if let Some(profile) = &self.active_profile {
            if let Some(url) = self.urls.get(profile) {
                return url.clone();
            }
        }
        self.default_url.clone()
    }

    /// Parse legacy bash config format (KEY="VALUE" lines)
    fn migrate_from_bash(path: &std::path::Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read legacy config")?;
        let mut config = Config::default();

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, val)) = line.split_once('=') {
                let val = val.trim_matches('"');
                if let Some(name) = key.strip_prefix("OLLAMA_URL_") {
                    config.urls.insert(name.to_string(), val.to_string());
                } else if key == "OLLAMA_CURRENT" {
                    config.active_profile = Some(val.to_string());
                } else if key == "OLLAMA_URL" {
                    config.default_url = val.to_string();
                }
            }
        }

        Ok(config)
    }
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
