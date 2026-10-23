use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::ProjectDirs;

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "nalide";
const APPLICATION: &str = "nalide";

#[derive(Debug, Clone)]
pub struct NalideDirs {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub databases_dir: PathBuf,
    pub config_file: PathBuf,
    pub timestamps_file: PathBuf,
}

impl NalideDirs {
    pub fn new() -> Result<Self> {
        if let Some(proj_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
            let config_dir = proj_dirs.config_dir().to_path_buf();
            let data_dir = proj_dirs.data_dir().to_path_buf();
            let cache_dir = proj_dirs.cache_dir().to_path_buf();
            let databases_dir = data_dir.join("databases");
            let config_file = config_dir.join("config.toml");
            let timestamps_file = config_dir.join("timestamps.toml");

            Ok(Self {
                config_dir,
                data_dir,
                cache_dir,
                databases_dir,
                config_file,
                timestamps_file,
            })
        } else {
            let home = dirs_with_fallback();
            let base = home.join(".nalide");
            let config_dir = base.join("config");
            let data_dir = base.join("data");
            let cache_dir = base.join("cache");
            let databases_dir = data_dir.join("databases");
            let config_file = config_dir.join("config.toml");
            let timestamps_file = config_dir.join("timestamps.toml");

            Ok(Self {
                config_dir,
                data_dir,
                cache_dir,
                databases_dir,
                config_file,
                timestamps_file,
            })
        }
    }

    pub fn with_overrides(
        config_dir: Option<PathBuf>,
        data_dir: Option<PathBuf>,
        cache_dir: Option<PathBuf>,
    ) -> Result<Self> {
        let defaults = Self::new()?;

        let config_dir = config_dir.unwrap_or(defaults.config_dir);
        let data_dir = data_dir.unwrap_or(defaults.data_dir);
        let cache_dir = cache_dir.unwrap_or(defaults.cache_dir);
        let databases_dir = data_dir.join("databases");
        let config_file = config_dir.join("config.toml");
        let timestamps_file = config_dir.join("timestamps.toml");

        Ok(Self {
            config_dir,
            data_dir,
            cache_dir,
            databases_dir,
            config_file,
            timestamps_file,
        })
    }

    pub fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(&self.config_dir)
            .with_context(|| format!("Failed to create config dir: {:?}", self.config_dir))?;
        fs::create_dir_all(&self.data_dir)
            .with_context(|| format!("Failed to create data dir: {:?}", self.data_dir))?;
        fs::create_dir_all(&self.cache_dir)
            .with_context(|| format!("Failed to create cache dir: {:?}", self.cache_dir))?;
        fs::create_dir_all(&self.databases_dir)
            .with_context(|| format!("Failed to create databases dir: {:?}", self.databases_dir))?;
        Ok(())
    }

    pub fn display(&self) {
        println!("配置目录: {}", self.config_dir.display());
        println!("数据目录: {}", self.data_dir.display());
        println!("缓存目录: {}", self.cache_dir.display());
        println!("数据库目录: {}", self.databases_dir.display());
        println!("配置文件: {}", self.config_file.display());
    }
}

fn dirs_with_fallback() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}
