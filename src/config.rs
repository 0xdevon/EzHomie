use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;

pub const DEFAULT_CONFIG: &str = r#"[off]
confirm_seconds = 10
shutdown = true
quit_timeout_secs = 15

[[apps]]
name = "Cursor"
delay_ms = 500

[[apps]]
name = "Google Chrome"
delay_ms = 500

[[urls]]
url = "https://calendar.google.com"
delay_ms = 300

[[commands]]
name = "start dev service"
cmd = "cd ~/project && npm run dev"
delay_ms = 0
"#;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Config {
    #[serde(default)]
    pub off: OffConfig,
    #[serde(default)]
    pub apps: Vec<AppTarget>,
    #[serde(default)]
    pub urls: Vec<UrlTarget>,
    #[serde(default)]
    pub commands: Vec<CommandTarget>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct OffConfig {
    #[serde(default = "default_confirm_seconds")]
    pub confirm_seconds: u64,
    #[serde(default = "default_shutdown")]
    pub shutdown: bool,
    #[serde(default = "default_quit_timeout_secs")]
    pub quit_timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AppTarget {
    pub name: String,
    #[serde(default)]
    pub delay_ms: u64,
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UrlTarget {
    pub url: String,
    #[serde(default)]
    pub delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct CommandTarget {
    #[serde(default)]
    pub name: Option<String>,
    pub cmd: String,
    #[serde(default)]
    pub delay_ms: u64,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let raw = fs::read_to_string(path)
            .with_context(|| format!("failed to read config {}", path.display()))?;
        toml::from_str(&raw).with_context(|| format!("failed to parse config {}", path.display()))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            off: OffConfig::default(),
            apps: vec![
                AppTarget {
                    name: "Cursor".to_string(),
                    delay_ms: 500,
                    timeout_secs: None,
                },
                AppTarget {
                    name: "Google Chrome".to_string(),
                    delay_ms: 500,
                    timeout_secs: None,
                },
            ],
            urls: vec![UrlTarget {
                url: "https://calendar.google.com".to_string(),
                delay_ms: 300,
            }],
            commands: vec![CommandTarget {
                name: Some("start dev service".to_string()),
                cmd: "cd ~/project && npm run dev".to_string(),
                delay_ms: 0,
            }],
        }
    }
}

impl Default for OffConfig {
    fn default() -> Self {
        Self {
            confirm_seconds: default_confirm_seconds(),
            shutdown: default_shutdown(),
            quit_timeout_secs: default_quit_timeout_secs(),
        }
    }
}

pub fn resolve_config_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(expand_tilde(path)?);
    }

    let home = env::var_os("HOME").context("HOME is not set; pass --config <PATH> instead")?;
    Ok(PathBuf::from(home).join(".config/ezhomie/config.toml"))
}

fn expand_tilde(path: PathBuf) -> Result<PathBuf> {
    let path_string = path.to_string_lossy();
    if path_string == "~" {
        let home = env::var_os("HOME").context("HOME is not set")?;
        return Ok(PathBuf::from(home));
    }

    if let Some(rest) = path_string.strip_prefix("~/") {
        let home = env::var_os("HOME").context("HOME is not set")?;
        return Ok(PathBuf::from(home).join(rest));
    }

    Ok(path)
}

fn default_confirm_seconds() -> u64 {
    10
}

fn default_shutdown() -> bool {
    true
}

fn default_quit_timeout_secs() -> u64 {
    15
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_config() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();

        assert_eq!(config.off.confirm_seconds, 10);
        assert!(config.off.shutdown);
        assert_eq!(config.off.quit_timeout_secs, 15);
        assert_eq!(config.apps.len(), 2);
        assert_eq!(config.urls.len(), 1);
        assert_eq!(config.commands.len(), 1);
    }

    #[test]
    fn missing_fields_use_defaults() {
        let config: Config = toml::from_str("").unwrap();

        assert_eq!(config.off, OffConfig::default());
        assert!(config.apps.is_empty());
        assert!(config.urls.is_empty());
        assert!(config.commands.is_empty());
    }

    #[test]
    fn invalid_toml_fails() {
        let err = toml::from_str::<Config>("[[apps]\nname = true").unwrap_err();
        assert!(err.message().contains("invalid") || err.message().contains("expected"));
    }

    #[test]
    fn app_name_is_required() {
        let err = toml::from_str::<Config>("[[apps]]\ndelay_ms = 10").unwrap_err();
        assert!(err.to_string().contains("missing field `name`"));
    }
}
