use std::fs;
use std::net::IpAddr;
use std::path::Path;

use anyhow::{Context, Result};
use geosite_rs::{decode_geoip, decode_geosite, GeoIpList, GeoSiteList};

use super::traits::{DomainDatabase, DomainInfo, IpDatabase, IpInfo};

pub struct V2rayDatabase {
    name: String,
    data: Vec<u8>,
    db_kind: V2rayDbKind,
}

enum V2rayDbKind {
    GeoIp(GeoIpList),
    GeoSite(GeoSiteList),
}

impl V2rayDatabase {
    pub fn new(name: &str, path: &Path) -> Result<Self> {
        let data = fs::read(path)
            .with_context(|| format!("Failed to read v2ray dat file: {}", path.display()))?;

        if let Ok(geoip) = decode_geoip(&data) {
            log::debug!("Loaded v2ray GeoIP database: {} ({} entries)", name, geoip.entry.len());
            return Ok(Self {
                name: name.to_string(),
                data,
                db_kind: V2rayDbKind::GeoIp(geoip),
            });
        }

        if let Ok(geosite) = decode_geosite(&data) {
            log::debug!("Loaded v2ray GeoSite database: {} ({} entries)", name, geosite.entry.len());
            return Ok(Self {
                name: name.to_string(),
                data,
                db_kind: V2rayDbKind::GeoSite(geosite),
            });
        }

        Err(anyhow::anyhow!(
            "Failed to decode v2ray dat file as either GeoIP or GeoSite: {}",
            path.display()
        ))
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

fn ip_in_cidr(ip: IpAddr, cidr_ip: &[u8], prefix: u32) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            if cidr_ip.len() != 4 {
                return false;
            }
            let addr_bytes = v4.octets();
            let cidr_bytes: [u8; 4] = cidr_ip.try_into().unwrap_or([0; 4]);

            let mask = if prefix == 0 {
                0u32
            } else {
                !0u32 << (32 - prefix)
            };

            let ip_num = u32::from_be_bytes(addr_bytes);
            let cidr_num = u32::from_be_bytes(cidr_bytes);

            (ip_num & mask) == (cidr_num & mask)
        }
        IpAddr::V6(v6) => {
            if cidr_ip.len() != 16 {
                return false;
            }
            let addr_bytes = v6.octets();
            let cidr_bytes: [u8; 16] = cidr_ip.try_into().unwrap_or([0; 16]);

            let ip_num = u128::from_be_bytes(addr_bytes);
            let cidr_num = u128::from_be_bytes(cidr_bytes);

            let mask = if prefix == 0 {
                0u128
            } else {
                !0u128 << (128 - prefix)
            };

            (ip_num & mask) == (cidr_num & mask)
        }
    }
}

impl IpDatabase for V2rayDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "v2ray"
    }

    fn lookup_ip(&self, ip: IpAddr) -> Result<Option<IpInfo>> {
        match &self.db_kind {
            V2rayDbKind::GeoIp(geoip_list) => {
                let mut matched_codes = Vec::new();

                for entry in &geoip_list.entry {
                    for cidr in &entry.cidr {
                        if ip_in_cidr(ip, &cidr.ip, cidr.prefix) {
                            if entry.reverse_match {
                                continue;
                            }
                            matched_codes.push(entry.country_code.clone());
                        } else if entry.reverse_match {
                            matched_codes.push(entry.country_code.clone());
                        }
                    }
                }

                if matched_codes.is_empty() {
                    return Ok(None);
                }

                let country_code = matched_codes.join(",");
                Ok(Some(IpInfo {
                    ip,
                    country: None,
                    country_code: Some(country_code),
                    region: None,
                    city: None,
                    isp: None,
                    org: None,
                    as_number: None,
                    db_name: self.name.clone(),
                }))
            }
            V2rayDbKind::GeoSite(_) => Ok(None),
        }
    }

    fn lookup_domain(&self, domain: &str) -> Result<Option<DomainInfo>> {
        self.lookup_domain_info(domain)
    }
}

impl DomainDatabase for V2rayDatabase {
    fn name(&self) -> &str {
        &self.name
    }

    fn db_type(&self) -> &str {
        "v2ray"
    }

    fn lookup_domain(&self, domain: &str) -> Result<Option<DomainInfo>> {
        self.lookup_domain_info(domain)
    }
}

impl V2rayDatabase {
    fn lookup_domain_info(&self, domain: &str) -> Result<Option<DomainInfo>> {
        let V2rayDbKind::GeoSite(site_list) = &self.db_kind else {
            return Ok(None);
        };

        let domain = domain.trim().trim_end_matches('.').to_ascii_lowercase();
        if domain.is_empty() {
            return Ok(None);
        }

        let mut tags = Vec::new();
        for entry in &site_list.entry {
            if entry_matches_domain(entry, &domain) {
                tags.push(entry.country_code.clone());
            }
        }

        if tags.is_empty() {
            return Ok(None);
        }

        tags.sort();
        tags.dedup();
        Ok(Some(DomainInfo {
            domain,
            category: tags.first().cloned(),
            tags,
            db_name: self.name.clone(),
        }))
    }
}

/// v2ray Domain.Type：0 Plain、1 Regex、2 Domain（根域名）、3 Full。
fn entry_matches_domain(entry: &geosite_rs::GeoSite, domain: &str) -> bool {
    entry.domain.iter().any(|rule| match rule.r#type {
        0 => !rule.value.is_empty() && domain.contains(&rule.value.to_ascii_lowercase()),
        1 => regex::Regex::new(&rule.value)
            .map(|re| re.is_match(domain))
            .unwrap_or(false),
        2 => domain_matches_root(domain, &rule.value),
        3 => domain == rule.value.to_ascii_lowercase(),
        _ => false,
    })
}

fn domain_matches_root(domain: &str, root: &str) -> bool {
    let root = root.trim().trim_start_matches('.').to_ascii_lowercase();
    domain == root || domain.ends_with(&format!(".{root}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_domain_matches_subdomain() {
        assert!(domain_matches_root("www.qq.com", "qq.com"));
        assert!(domain_matches_root("qq.com", "qq.com"));
        assert!(!domain_matches_root("notqq.com", "qq.com"));
        assert!(!domain_matches_root("qq.com.evil.test", "qq.com"));
    }
}
