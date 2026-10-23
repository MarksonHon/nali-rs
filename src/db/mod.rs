pub mod mmdb;
pub mod traits;
pub mod v2ray;

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::dirs::NalideDirs;
use traits::{IpDatabase, load_database};

pub struct DatabaseManager {
    databases: Vec<Arc<dyn IpDatabase>>,
}

impl DatabaseManager {
    pub fn new() -> Self {
        Self {
            databases: Vec::new(),
        }
    }

    pub fn load_all(&mut self, config: &Config, dirs: &NalideDirs) -> Vec<String> {
        let mut failed = Vec::new();

        for db_config in &config.databases {
            if !db_config.enabled {
                log::info!("Skipping disabled database: {}", db_config.name);
                continue;
            }

            let db_path = config.resolve_db_path(db_config, dirs);
            if !db_path.exists() {
                log::warn!(
                    "Database file not found: {} ({}).",
                    db_path.display(),
                    db_config.name
                );
                failed.push(db_config.name.clone());
                continue;
            }

            match load_database(&db_config.name, &db_config.db_type.to_string(), &db_path) {
                Ok(db) => {
                    log::info!("Loaded database: {} ({})", db_config.name, db.db_type());
                    self.databases.push(Arc::from(db));
                }
                Err(e) => {
                    log::error!("Failed to load database {}: {}", db_config.name, e);
                    failed.push(db_config.name.clone());
                }
            }
        }

        self.databases
            .sort_by(|a, b| a.name().cmp(b.name()));

        failed
    }

    pub fn load_single(&mut self, name: &str, db_type: &str, path: &Path) -> Result<()> {
        let db = load_database(name, db_type, path)
            .with_context(|| format!("Failed to load database: {}", name))?;
        self.databases.push(Arc::from(db));
        Ok(())
    }

    pub fn databases(&self) -> &[Arc<dyn IpDatabase>] {
        &self.databases
    }

    pub fn lookup_ip(&self, ip: std::net::IpAddr) -> Vec<traits::IpInfo> {
        let mut results = Vec::new();
        for db in &self.databases {
            match db.lookup_ip(ip) {
                Ok(Some(info)) => results.push(info),
                Ok(None) => {}
                Err(e) => {
                    log::error!("Error querying database {}: {}", db.name(), e);
                }
            }
        }
        results
    }

    pub fn lookup_domain(&self, domain: &str) -> Vec<traits::DomainInfo> {
        let mut results = Vec::new();
        for db in &self.databases {
            match db.lookup_domain(domain) {
                Ok(Some(info)) => results.push(info),
                Ok(None) => {}
                Err(e) => {
                    log::error!("Error querying domain in {}: {}", db.name(), e);
                }
            }
        }
        results
    }

    pub fn first_lookup_ip(&self, ip: std::net::IpAddr) -> Option<traits::IpInfo> {
        for db in &self.databases {
            match db.lookup_ip(ip) {
                Ok(Some(info)) => return Some(info),
                Ok(None) => continue,
                Err(e) => {
                    log::error!("Error querying database {}: {}", db.name(), e);
                    continue;
                }
            }
        }
        None
    }
}
