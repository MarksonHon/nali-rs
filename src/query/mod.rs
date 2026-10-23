pub mod ip;

use std::net::IpAddr;

use colored::Colorize;
use regex::Regex;

use crate::db::traits::{DomainInfo, IpInfo};
use crate::db::DatabaseManager;

#[derive(Debug, Clone)]
pub struct QueryResult {
    pub input: String,
    pub ip: Option<IpAddr>,
    pub results: Vec<IpInfo>,
    pub domains: Vec<DomainInfo>,
}

pub fn query_ip(db_manager: &DatabaseManager, input: &str) -> QueryResult {
    query_direct(db_manager, input)
}

/// 直接查询：IP 合并全部 IP 库；非 IP 按域名查 geosite。
pub fn query_direct(db_manager: &DatabaseManager, input: &str) -> QueryResult {
    let trimmed = input.trim().to_string();

    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        return QueryResult {
            input: trimmed,
            ip: Some(ip),
            results: db_manager.lookup_ip(ip),
            domains: Vec::new(),
        };
    }

    QueryResult {
        input: trimmed.clone(),
        ip: None,
        results: Vec::new(),
        domains: db_manager.lookup_domain(&trimmed),
    }
}

pub fn query_first(db_manager: &DatabaseManager, input: &str) -> QueryResult {
    query_direct(db_manager, input)
}

/// 管道模式：保留原始行。
///
/// IP 后面标注地理归属；域名后面列出 geosite 命中的集合。
///
/// `color` 为真时，原始内容为绿色，本工具追加的标注为黄色。
pub fn annotate_line(db_manager: &DatabaseManager, line: &str, color: bool) -> String {
    let pattern = Regex::new(
        r"(?x)
        (?:
            (?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)
            (?:\.(?:25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}
        )
        |
        (?:
            (?:[0-9A-Fa-f]{0,4}:){2,7}[0-9A-Fa-f]{0,4}
        )
        ",
    )
    .expect("ip regex");

    let mut plain = String::new();
    let mut marks: Vec<(usize, usize)> = Vec::new();
    let mut last = 0;
    for mat in pattern.find_iter(line) {
        let token = mat.as_str();
        let Ok(ip) = token.parse::<IpAddr>() else {
            continue;
        };
        plain.push_str(&line[last..mat.end()]);
        if let Some(info) = db_manager.first_lookup_ip(ip) {
            let note = format!(" {}", info);
            let start = plain.len();
            plain.push_str(&note);
            marks.push((start, plain.len()));
        }
        last = mat.end();
    }
    plain.push_str(&line[last..]);
    annotate_domains(db_manager, &plain, marks, color)
}

fn paint_source(text: &str, color: bool) -> String {
    if text.is_empty() || !color {
        return text.to_string();
    }
    text.green().to_string()
}

fn paint_annotation(text: &str, color: bool) -> String {
    if !color {
        return text.to_string();
    }
    text.yellow().to_string()
}

fn annotate_domains(
    db_manager: &DatabaseManager,
    line: &str,
    mut marks: Vec<(usize, usize)>,
    color: bool,
) -> String {
    let pattern = Regex::new(
        r"(?i)(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}",
    )
    .expect("domain regex");

    let mut inserts: Vec<(usize, String)> = Vec::new();
    for mat in pattern.find_iter(line) {
        let token = mat.as_str().trim_end_matches('.');
        let infos = db_manager.lookup_domain(token);
        if infos.is_empty() {
            continue;
        }
        let tags: Vec<String> = infos
            .iter()
            .flat_map(|info| info.tags.iter().cloned())
            .collect();
        if tags.is_empty() {
            continue;
        }
        inserts.push((mat.end(), format!(" [{}]", tags.join(", "))));
    }

    inserts.sort_by_key(|item| item.0);
    let mut plain = String::new();
    let mut notes: Vec<(usize, usize)> = Vec::new();
    let mut last = 0usize;
    for (at, note) in &inserts {
        plain.push_str(&line[last..*at]);
        let start = plain.len();
        plain.push_str(note);
        notes.push((start, plain.len()));
        let added = note.len();
        for (start_mark, end_mark) in marks.iter_mut() {
            if *start_mark >= *at {
                *start_mark += added;
                *end_mark += added;
            }
        }
        last = *at;
    }
    plain.push_str(&line[last..]);
    marks.extend(notes);
    paint_marked(&plain, &marks, color)
}

fn paint_marked(text: &str, marks: &[(usize, usize)], color: bool) -> String {
    if !color {
        return text.to_string();
    }
    let mut spans = marks.to_vec();
    spans.sort_by_key(|span| span.0);
    let mut out = String::new();
    let mut last = 0usize;
    for (start, end) in spans {
        if start < last || end > text.len() || start > end {
            continue;
        }
        out.push_str(&paint_source(&text[last..start], true));
        out.push_str(&paint_annotation(&text[start..end], true));
        last = end;
    }
    out.push_str(&paint_source(&text[last..], true));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annotate_keeps_text_without_db() {
        let db = DatabaseManager::new();
        let line = "服务器:  dns.example.com";
        assert_eq!(annotate_line(&db, line, false), line);

        let line = "Address:  1.1.1.1";
        assert_eq!(annotate_line(&db, line, false), line);
    }
}
