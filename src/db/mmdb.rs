use std::net::IpAddr;
use std::path::Path;

use anyhow::{Context, Result};
use maxminddb::geoip2::City;
use maxminddb::Reader;

use super::traits::{IpDatabase, IpInfo};

pub struct MmdbDatabase {
    name: String,
    reader: Reader<Vec<u8>>,
}

impl MmdbDatabase {
    pub fn new(name: &str, path: &Path) -> Result<Self> {
        let reader = Reader::open_readfile(path)
            .with_context(|| format!("Failed to read mmdb file: {}", path.display()))?;

        Ok(Self {
            name: name.to_string(),
            reader,
        })
    }
}

impl IpDatabase for MmdbDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "mmdb"
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<IpInfo>> {
        let city: City = match self.reader.lookup(ip) {
            Ok(city) => city,
            Err(maxminddb::MaxMindDBError::AddressNotFoundError(_)) => return Ok(None),
            Err(e) => return Err(anyhow::anyhow!("MMDB lookup error: {}", e)),
        };

        let country = city
            .country
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("en").copied().or_else(|| n.values().next().copied()))
            .map(|s| s.to_string());

        let country_code = city
            .country
            .as_ref()
            .and_then(|c| c.iso_code)
            .map(|s| s.to_string());

        let region = city
            .subdivisions
            .as_ref()
            .and_then(|s| s.first())
            .and_then(|s| s.names.as_ref())
            .and_then(|n| n.get("en").copied().or_else(|| n.values().next().copied()))
            .map(|s| s.to_string());

        let city_name = city
            .city
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|n| n.get("en").copied().or_else(|| n.values().next().copied()))
            .map(|s| s.to_string());

        Ok(Some(IpInfo {
            ip,
            country,
            country_code,
            region,
            city: city_name,
            isp: None,
            org: None,
            as_number: None,
            db_name: self.name.clone(),
        }))
    }
}
