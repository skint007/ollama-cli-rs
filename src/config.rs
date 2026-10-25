use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

const DEFAULT_URL: &str = "https://ollama.local.skint007.dev";

/// Which API a profile's URL speaks
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    /// Native Ollama API (/api/*)
    #[default]
    Ollama,
    /// OpenAI-compatible API (/v1/*), e.g. LiteLLM
    #[value(name = "openai")]
    #[serde(rename = "openai")]
    OpenAi,
}

impl BackendKind {
    pub fn as_str(self) -> &'static str {
        match self {
            BackendKind::Ollama => "ollama",
            BackendKind::OpenAi => "openai",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub url: String,
    #[serde(default)]
    pub kind: BackendKind,
    /// API key stored in the config file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Name of an environment variable holding the API key (takes precedence over api_key)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_env: Option<String>,
}

impl Profile {
    pub fn ollama(url: &str) -> Self {
        Profile {
            url: url.to_string(),
            kind: BackendKind::Ollama,
            api_key: None,
            api_key_env: None,
        }
    }

    /// Short description of where the API key comes from, without revealing it
    pub fn key_source(&self) -> Option<String> {
        if let Some(var) = &self.api_key_env {
            Some(format!("${}", var))
        } else {
            self.api_key.as_ref().map(|_| "config file".to_string())
        }
    }

    pub fn endpoint(&self) -> Result<Endpoint> {
        let api_key = match &self.api_key_env {
            Some(var) => Some(std::env::var(var).with_context(|| {
                format!("API key environment variable ${} is not set", var)
            })?),
            None => self.api_key.clone(),
        };
        Ok(Endpoint {
            api_key,
            ..self.endpoint_without_key()
        })
    }

    pub fn endpoint_without_key(&self) -> Endpoint {
        Endpoint {
            url: self.url.clone(),
            kind: self.kind,
            api_key: None,
        }
    }
}

/// Everything needed to talk to a backend
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub url: String,
    pub kind: BackendKind,
    pub api_key: Option<String>,
}

impl Endpoint {
    pub fn ollama(url: &str) -> Self {
        Endpoint {
            url: url.to_string(),
            kind: BackendKind::Ollama,
            api_key: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_url")]
    pub default_url: String,
    pub active_profile: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
    /// Legacy `[urls]` table (name = url); folded into `profiles` on load
    #[serde(default, skip_serializing)]
    urls: BTreeMap<String, String>,
}

fn default_url() -> String {
    DEFAULT_URL.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_url: DEFAULT_URL.to_string(),
            active_profile: None,
            profiles: BTreeMap::new(),
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
            let mut config: Config =
                toml::from_str(&contents).context("Failed to parse config file")?;
            config.fold_legacy_urls();
            config.warn_if_key_exposed(&config_path);
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

    /// Save config to TOML file (owner-only permissions, since it may hold API keys)
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create config directory")?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;

        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
            options.mode(0o600);
            // mode() only applies on creation; tighten existing files too
            if config_path.exists() {
                std::fs::set_permissions(&config_path, std::fs::Permissions::from_mode(0o600))
                    .context("Failed to set config file permissions")?;
            }
        }
        use std::io::Write;
        options
            .open(&config_path)
            .and_then(|mut f| f.write_all(contents.as_bytes()))
            .context("Failed to write config file")?;
        Ok(())
    }

    /// Warn when a hand-edited config holds an API key but is readable by others
    fn warn_if_key_exposed(&self, path: &std::path::Path) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let has_key = self.profiles.values().any(|p| p.api_key.is_some());
            let mode = std::fs::metadata(path).map(|m| m.permissions().mode());
            if has_key && mode.is_ok_and(|m| m & 0o077 != 0) {
                eprintln!(
                    "Warning: {} contains an API key but is readable by other users.\n         Run: chmod 600 {}",
                    path.display(),
                    path.display()
                );
            }
        }
        #[cfg(not(unix))]
        let _ = path;
    }

    /// The active profile, if one is set and exists
    pub fn active(&self) -> Option<&Profile> {
        self.active_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Resolve the effective URL: CLI override > active profile > default
    pub fn resolve_url(&self, cli_override: Option<&str>) -> String {
        if let Some(url) = cli_override {
            return url.to_string();
        }
        match self.active() {
            Some(profile) => profile.url.clone(),
            None => self.default_url.clone(),
        }
    }

    /// Resolve the effective endpoint. A `--url` override is always treated as a
    /// plain Ollama URL with no API key.
    pub fn resolve_endpoint(&self, cli_override: Option<&str>) -> Result<Endpoint> {
        if let Some(url) = cli_override {
            return Ok(Endpoint::ollama(url));
        }
        match self.active() {
            Some(profile) => profile.endpoint(),
            None => Ok(Endpoint::ollama(&self.default_url)),
        }
    }

    /// Like `resolve_endpoint`, but never fails: the API key is left out
    pub fn resolve_endpoint_without_key(&self, cli_override: Option<&str>) -> Endpoint {
        if let Some(url) = cli_override {
            return Endpoint::ollama(url);
        }
        match self.active() {
            Some(profile) => profile.endpoint_without_key(),
            None => Endpoint::ollama(&self.default_url),
        }
    }

    fn fold_legacy_urls(&mut self) {
        for (name, url) in std::mem::take(&mut self.urls) {
            self.profiles
                .entry(name)
                .or_insert_with(|| Profile::ollama(&url));
        }
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
                    config
                        .profiles
                        .insert(name.to_string(), Profile::ollama(val));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_urls_become_ollama_profiles() {
        let mut config: Config = toml::from_str(
            r#"
            default_url = "http://a:11434"
            active_profile = "local"

            [urls]
            local = "http://localhost:11434"
            "#,
        )
        .unwrap();
        config.fold_legacy_urls();

        let profile = config.active().unwrap();
        assert_eq!(profile.url, "http://localhost:11434");
        assert_eq!(profile.kind, BackendKind::Ollama);

        let saved = toml::to_string_pretty(&config).unwrap();
        assert!(!saved.contains("[urls]"));
        assert!(saved.contains("[profiles.local]"));
    }

    #[test]
    fn openai_profile_resolves_key_from_config() {
        let config: Config = toml::from_str(
            r#"
            active_profile = "litellm"

            [profiles.litellm]
            url = "http://host:4000"
            kind = "openai"
            api_key = "sk-test"
            "#,
        )
        .unwrap();

        let endpoint = config.resolve_endpoint(None).unwrap();
        assert_eq!(endpoint.kind, BackendKind::OpenAi);
        assert_eq!(endpoint.api_key.as_deref(), Some("sk-test"));

        let overridden = config.resolve_endpoint(Some("http://other:11434")).unwrap();
        assert_eq!(overridden.kind, BackendKind::Ollama);
        assert!(overridden.api_key.is_none());
    }

    #[test]
    fn missing_key_env_var_is_an_error() {
        let profile = Profile {
            url: "http://host:4000".to_string(),
            kind: BackendKind::OpenAi,
            api_key: None,
            api_key_env: Some("OLLAMA_CLI_TEST_SURELY_UNSET_VAR".to_string()),
        };
        assert!(profile.endpoint().is_err());
    }
}
