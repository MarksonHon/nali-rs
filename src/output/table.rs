use comfy_table::{Table, presets::UTF8_FULL, modifiers::UTF8_ROUND_CORNERS, ContentArrangement};

use crate::query::QueryResult;

pub fn print_table(result: &QueryResult) {
    if !result.domains.is_empty() {
        print_domain_table(result);
        return;
    }

    if result.results.is_empty() {
        return;
    }

    println!("查询结果列表：");

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(["IP地址", "数据库", "国家/地区", "区域", "城市", "ISP/Org"]);

    for info in &result.results {
        let country = info
            .country
            .as_deref()
            .or(info.country_code.as_deref())
            .unwrap_or("-");

        let region = info.region.as_deref().unwrap_or("-");
        let city = info.city.as_deref().unwrap_or("-");

        let mut org_parts = Vec::new();
        if let Some(ref isp) = info.isp {
            org_parts.push(isp.clone());
        }
        if let Some(ref org) = info.org {
            if !org_parts.contains(org) {
                org_parts.push(org.clone());
            }
        }
        if let Some(ref asn) = info.as_number {
            org_parts.push(asn.clone());
        }
        let org = if org_parts.is_empty() {
            "-".to_string()
        } else {
            org_parts.join(", ")
        };

        table.add_row([
            info.ip.to_string(),
            info.db_name.clone(),
            country.to_string(),
            region.to_string(),
            city.to_string(),
            org,
        ]);
    }

    println!("{table}");
}

fn print_domain_table(result: &QueryResult) {
    println!("域名查询结果：");

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(["域名", "数据库", "集合"]);

    for info in &result.domains {
        let tags = if info.tags.is_empty() {
            "-".to_string()
        } else {
            info.tags.join(", ")
        };
        table.add_row([info.domain.clone(), info.db_name.clone(), tags]);
    }

    println!("{table}");
}
