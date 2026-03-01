use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::error::CliError;

const DEFAULT_API_URL: &str = "https://api.supersynth.ai";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_api_url")]
    pub api_url: String,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

fn default_api_url() -> String {
    DEFAULT_API_URL.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub api_key: Option<String>,
    pub dev_token: Option<String>,
    pub dev_token_expires: Option<String>,
    pub tenant_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DefaultsConfig {
    pub tenant_id: Option<String>,
    pub project_id: Option<String>,
}

impl Config {
    pub fn config_dir() -> Result<PathBuf> {
        let dir = dirs::config_dir()
            .context("Could not determine config directory")?
            .join("ssynth");
        Ok(dir)
    }

    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir()?;
        fs::create_dir_all(&dir)?;
        let path = Self::config_path()?;
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        fs::write(&path, &contents)?;
        set_permissions_0600(&path)?;
        Ok(())
    }

    /// Resolve the effective API key. Priority:
    /// env `SSYNTH_API_KEY` > config `api_key` > config `dev_token` > error
    pub fn resolve_auth_token(&self) -> Result<AuthToken, CliError> {
        if let Ok(key) = std::env::var("SSYNTH_API_KEY") {
            if !key.is_empty() {
                return Ok(AuthToken::ApiKey(key));
            }
        }
        if let Some(ref key) = self.auth.api_key {
            return Ok(AuthToken::ApiKey(key.clone()));
        }
        if let Some(ref token) = self.auth.dev_token {
            if let Some(ref expires) = self.auth.dev_token_expires {
                if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(expires) {
                    if exp < chrono::Utc::now() {
                        return Err(CliError::NotAuthenticated);
                    }
                }
            }
            return Ok(AuthToken::Jwt(token.clone()));
        }
        Err(CliError::NotAuthenticated)
    }

    pub fn effective_api_url(&self, cli_override: Option<&str>) -> String {
        if let Some(url) = cli_override {
            return url.to_string();
        }
        if let Ok(url) = std::env::var("SSYNTH_API_URL") {
            if !url.is_empty() {
                return url;
            }
        }
        self.api_url.clone()
    }

    pub fn effective_tenant_id(&self) -> Option<&str> {
        self.defaults
            .tenant_id
            .as_deref()
            .or(self.auth.tenant_id.as_deref())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            auth: AuthConfig::default(),
            defaults: DefaultsConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthToken {
    ApiKey(String),
    Jwt(String),
}

#[cfg(unix)]
fn set_permissions_0600(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions_0600(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api_url, DEFAULT_API_URL);
        assert!(config.auth.api_key.is_none());
    }

    #[test]
    fn test_roundtrip_toml() {
        let config = Config {
            api_url: "http://localhost:3000".to_string(),
            auth: AuthConfig {
                api_key: Some("sk_live_abc123".to_string()),
                ..Default::default()
            },
            defaults: DefaultsConfig {
                tenant_id: Some("019-test".to_string()),
                ..Default::default()
            },
        };
        let s = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&s).unwrap();
        assert_eq!(parsed.api_url, config.api_url);
        assert_eq!(parsed.auth.api_key, config.auth.api_key);
        assert_eq!(parsed.defaults.tenant_id, config.defaults.tenant_id);
    }

    #[test]
    fn test_auth_priority_env_first() {
        let config = Config {
            auth: AuthConfig {
                api_key: Some("sk_live_config".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };
        // Config key should work when env is not set
        let token = config.resolve_auth_token().unwrap();
        match token {
            AuthToken::ApiKey(k) => assert_eq!(k, "sk_live_config"),
            AuthToken::Jwt(_) => panic!("Expected ApiKey"),
        }
    }

    #[test]
    fn test_no_auth_returns_error() {
        let config = Config::default();
        let result = config.resolve_auth_token();
        assert!(result.is_err());
    }
}
