use std::fmt;
use std::net::IpAddr;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone)]
pub struct IpInfo {
    pub ip: IpAddr,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub isp: Option<String>,
    pub org: Option<String>,
    pub as_number: Option<String>,
    pub db_name: String,
}

impl fmt::Display for IpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(ref country) = self.country {
            parts.push(country.clone());
        } else if let Some(ref code) = self.country_code {
            parts.push(code.clone());
        }
        if let Some(ref city) = self.city {
            parts.push(city.clone());
        }
        if let Some(ref isp) = self.isp {
            parts.push(isp.clone());
        }
        if parts.is_empty() {
            write!(f, "[{}]", self.db_name)
        } else {
            write!(f, "[{}]", parts.join(" "))
        }
    }
}

#[derive(Debug, Clone)]
pub struct DomainInfo {
    pub domain: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub db_name: String,
}

impl fmt::Display for DomainInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tags.is_empty() {
            write!(f, "[{}]", self.db_name)
        } else {
            write!(f, "[{}]", self.tags.join(", "))
        }
    }
}

pub trait IpDatabase: Send + Sync {
    fn name(&self) -> &str;
    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<IpInfo>>;
    fn db_type(&self) -> &str;

    fn lookup_domain(&self, _domain: &str) -> Result<Option<DomainInfo>> {
        Ok(None)
    }
}

pub trait DomainDatabase: Send + Sync {
    fn name(&self) -> &str;
    fn lookup_domain(&self, domain: &str) -> Result<Option<DomainInfo>>;
    fn db_type(&self) -> &str;
}

pub fn load_database(
    name: &str,
    db_type: &str,
    path: &Path,
) -> Result<Box<dyn IpDatabase>> {
    match db_type {
        "mmdb" => {
            let db = super::mmdb::MmdbDatabase::new(name, path)?;
            Ok(Box::new(db))
        }
        "v2ray" => {
            let db = super::v2ray::V2rayDatabase::new(name, path)?;
            Ok(Box::new(db))
        }
        _ => Err(anyhow::anyhow!("Unsupported database type: {}", db_type)),
    }
}
