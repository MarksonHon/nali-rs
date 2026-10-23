use colored::Colorize;

use crate::query::QueryResult;

pub fn print_compact(result: &QueryResult) {
    if result.results.is_empty() && result.domains.is_empty() {
        println!("{}", result.input.red());
        return;
    }

    if !result.domains.is_empty() {
        for (i, info) in result.domains.iter().enumerate() {
            let info_str = info.to_string();
            if i == 0 {
                println!(
                    "{} {}",
                    result.input.green().bold(),
                    info_str.yellow()
                );
            } else {
                println!(
                    "{} {}",
                    " ".repeat(result.input.len()),
                    info_str.yellow()
                );
            }
        }
        return;
    }

    if result.results.len() == 1 {
        let info = &result.results[0];
        let info_str = format_info(info);
        println!(
            "{} {}",
            result.input.green().bold(),
            info_str.bright_blue()
        );
    } else {
        for (i, info) in result.results.iter().enumerate() {
            let info_str = format_info(info);
            if i == 0 {
                println!(
                    "{} {}",
                    result.input.green().bold(),
                    info_str.bright_blue()
                );
            } else {
                println!(
                    "{} {}",
                    " ".repeat(result.input.len()).green().bold(),
                    info_str.bright_blue()
                );
            }
        }
    }
}

fn format_info(info: &crate::db::traits::IpInfo) -> String {
    let mut parts = Vec::new();

    if let Some(ref country) = info.country {
        parts.push(country.clone());
    } else if let Some(ref code) = info.country_code {
        parts.push(code.clone());
    }

    if let Some(ref region) = info.region {
        parts.push(region.clone());
    }

    if let Some(ref city) = info.city {
        parts.push(city.clone());
    }

    if let Some(ref isp) = info.isp {
        parts.push(isp.clone());
    }

    if let Some(ref org) = info.org {
        if !parts.contains(org) {
            parts.push(org.clone());
        }
    }

    if let Some(ref asn) = info.as_number {
        parts.push(asn.clone());
    }

    if parts.is_empty() {
        format!("[{}]", info.db_name)
    } else {
        format!("[{}]", parts.join(" "))
    }
}
