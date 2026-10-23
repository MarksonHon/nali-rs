use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::dirs::NalideDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_language")]
    pub language: String,

    #[serde(default = "default_output_mode")]
    pub output_mode: OutputMode,

    #[serde(default)]
    pub directories: DirectoryConfig,

    #[serde(default)]
    pub databases: Vec<DatabaseConfig>,

    #[serde(default)]
    pub query: QueryConfig,

    #[serde(default)]
    pub update: UpdateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    Compact,
    Table,
    Both,
}

impl Default for OutputMode {
    fn default() -> Self {
        Self::Both
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DirectoryConfig {
    pub config_dir: Option<PathBuf>,
    pub data_dir: Option<PathBuf>,
    pub cache_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub name: String,
    #[serde(rename = "type")]
    pub db_type: DatabaseType,
    pub path: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_priority")]
    pub priority: u32,
    pub update_url: Option<String>,
    #[serde(default = "default_update_interval")]
    pub update_interval: u64,
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Mmdb,
    V2ray,
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Mmdb => write!(f, "mmdb"),
            DatabaseType::V2ray => write!(f, "v2ray"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            cache_size: default_cache_size(),
            timeout: default_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    #[serde(default = "default_true")]
    pub auto_update: bool,
    #[serde(default = "default_true")]
    pub check_on_startup: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_update: default_true(),
            check_on_startup: default_true(),
        }
    }
}

fn default_language() -> String {
    "zh-CN".to_string()
}

fn default_output_mode() -> OutputMode {
    OutputMode::Both
}

fn default_true() -> bool {
    true
}

fn default_priority() -> u32 {
    100
}

fn default_update_interval() -> u64 {
    604800
}

fn default_cache_size() -> usize {
    1000
}

fn default_timeout() -> u64 {
    5000
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: default_language(),
            output_mode: default_output_mode(),
            directories: DirectoryConfig::default(),
            databases: vec![
                DatabaseConfig {
                    name: "GeoLite2-City".to_string(),
                    db_type: DatabaseType::Mmdb,
                    path: "GeoLite2-City.mmdb".to_string(),
                    enabled: true,
                    priority: 1,
                    update_url: Some(
                        "https://github.com/P3TERX/GeoLite.mmdb/raw/download/GeoLite2-City.mmdb"
                            .to_string(),
                    ),
                    update_interval: 86400,
                    last_updated: None,
                },
                DatabaseConfig {
                    name: "v2ray-geoip".to_string(),
                    db_type: DatabaseType::V2ray,
                    path: "geoip.dat".to_string(),
                    enabled: true,
                    priority: 2,
                    update_url: Some(
                        "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geoip.dat"
                            .to_string(),
                    ),
                    update_interval: 604800,
                    last_updated: None,
                },
                DatabaseConfig {
                    name: "v2ray-geosite".to_string(),
                    db_type: DatabaseType::V2ray,
                    path: "geosite.dat".to_string(),
                    enabled: true,
                    priority: 3,
                    update_url: Some(
                        "https://cdn.jsdelivr.net/gh/Loyalsoldier/v2ray-rules-dat@release/geosite.dat"
                            .to_string(),
                    ),
                    update_interval: 604800,
                    last_updated: None,
                },
            ],
            query: QueryConfig::default(),
            update: UpdateConfig::default(),
        }
    }
}

impl Config {
    pub fn load(dirs: &NalideDirs) -> Result<Self> {
        if !dirs.config_file.exists() {
            let config = Config::default();
            config.save(dirs)?;
            return Ok(config);
        }

        let content = fs::read_to_string(&dirs.config_file)
            .with_context(|| format!("Failed to read config file: {:?}", dirs.config_file))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", dirs.config_file))?;

        Ok(config)
    }

    pub fn save(&self, dirs: &NalideDirs) -> Result<()> {
        dirs.ensure_dirs()?;

        let content =
            toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(&dirs.config_file, content)
            .with_context(|| format!("Failed to write config file: {:?}", dirs.config_file))?;

        Ok(())
    }

    pub fn resolve_db_path(&self, db_config: &DatabaseConfig, dirs: &NalideDirs) -> PathBuf {
        let path = PathBuf::from(&db_config.path);
        if path.is_absolute() {
            path
        } else {
            dirs.databases_dir.join(path)
        }
    }
}
