use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub index: IndexConfig,
    pub query: QueryConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    #[serde(deserialize_with = "deserialize_duration_secs")]
    pub idle_timeout: Duration,
    pub content_budget_mb: usize,
    pub log_level: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct IndexConfig {
    pub index_content: bool,
    pub respect_ignore: bool,
    pub include_hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct QueryConfig {
    pub max_results: usize,
    #[serde(deserialize_with = "deserialize_duration_secs")]
    pub query_timeout: Duration,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(300),
            content_budget_mb: 256,
            log_level: "info".to_string(),
        }
    }
}

impl Default for IndexConfig {
    fn default() -> Self {
        Self {
            index_content: true,
            respect_ignore: true,
            include_hidden: false,
        }
    }
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            max_results: 10_000,
            query_timeout: Duration::from_secs(30),
        }
    }
}

/// Load configuration with precedence: env vars > config file > defaults.
///
/// CLI flag overrides are applied by the caller (e.g., ff-cli) after loading.
/// Environment variables use the `FF_*` prefix (e.g., `FF_IDLE_TIMEOUT`).
/// Config file is read from `$XDG_CONFIG_HOME/ff/config.toml` or `~/.config/ff/config.toml`.
#[must_use]
pub fn load() -> Config {
    let mut config = Config::default();

    if let Some(config_path) = config_file_path()
        && config_path.exists()
    {
        match std::fs::read_to_string(&config_path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(file_config) => config = file_config,
                Err(e) => {
                    tracing::warn!("failed to parse config at {}: {e}", config_path.display());
                }
            },
            Err(e) => {
                tracing::warn!("failed to read config at {}: {e}", config_path.display());
            }
        }
    }

    apply_env_overrides(&mut config);

    config
}

#[must_use]
fn config_file_path() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("ff").join("config.toml"));
    }

    if let Ok(home) = std::env::var("HOME") {
        return Some(
            PathBuf::from(home)
                .join(".config")
                .join("ff")
                .join("config.toml"),
        );
    }

    None
}

fn apply_env_overrides(config: &mut Config) {
    if let Ok(val) = std::env::var("FF_IDLE_TIMEOUT")
        && let Ok(secs) = val.parse::<u64>()
    {
        config.daemon.idle_timeout = Duration::from_secs(secs);
    }

    if let Ok(val) = std::env::var("FF_CONTENT_BUDGET_MB")
        && let Ok(mb) = val.parse::<usize>()
    {
        config.daemon.content_budget_mb = mb;
    }

    if let Ok(val) = std::env::var("FF_LOG_LEVEL") {
        config.daemon.log_level = val;
    }

    if let Ok(val) = std::env::var("FF_INDEX_CONTENT") {
        config.index.index_content = val == "true" || val == "1";
    }

    if let Ok(val) = std::env::var("FF_RESPECT_IGNORE") {
        config.index.respect_ignore = val == "true" || val == "1";
    }

    if let Ok(val) = std::env::var("FF_INCLUDE_HIDDEN") {
        config.index.include_hidden = val == "true" || val == "1";
    }

    if let Ok(val) = std::env::var("FF_MAX_RESULTS")
        && let Ok(n) = val.parse::<usize>()
    {
        config.query.max_results = n;
    }

    if let Ok(val) = std::env::var("FF_QUERY_TIMEOUT")
        && let Ok(secs) = val.parse::<u64>()
    {
        config.query.query_timeout = Duration::from_secs(secs);
    }
}

fn deserialize_duration_secs<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs = u64::deserialize(deserializer)?;
    Ok(Duration::from_secs(secs))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = Config::default();
        assert_eq!(config.daemon.idle_timeout, Duration::from_secs(300));
        assert_eq!(config.daemon.content_budget_mb, 256);
        assert_eq!(config.daemon.log_level, "info");
        assert!(config.index.index_content);
        assert!(config.index.respect_ignore);
        assert!(!config.index.include_hidden);
        assert_eq!(config.query.max_results, 10_000);
        assert_eq!(config.query.query_timeout, Duration::from_secs(30));
    }

    #[test]
    fn env_override_idle_timeout() {
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::set_var("FF_IDLE_TIMEOUT", "600");
        }
        let mut config = Config::default();
        apply_env_overrides(&mut config);
        assert_eq!(config.daemon.idle_timeout, Duration::from_secs(600));
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::remove_var("FF_IDLE_TIMEOUT");
        }
    }

    #[test]
    fn env_override_content_budget() {
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::set_var("FF_CONTENT_BUDGET_MB", "512");
        }
        let mut config = Config::default();
        apply_env_overrides(&mut config);
        assert_eq!(config.daemon.content_budget_mb, 512);
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::remove_var("FF_CONTENT_BUDGET_MB");
        }
    }

    #[test]
    fn env_override_max_results() {
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::set_var("FF_MAX_RESULTS", "5000");
        }
        let mut config = Config::default();
        apply_env_overrides(&mut config);
        assert_eq!(config.query.max_results, 5000);
        // SAFETY: test runs in isolation, no concurrent env access
        unsafe {
            std::env::remove_var("FF_MAX_RESULTS");
        }
    }
}
