use std::net::IpAddr;

pub fn parse_ip(input: &str) -> Option<IpAddr> {
    let trimmed = input.trim();
    trimmed.parse().ok()
}

pub fn is_valid_ip(input: &str) -> bool {
    parse_ip(input).is_some()
}

pub fn extract_ips_from_text(text: &str) -> Vec<String> {
    let mut ips = Vec::new();
    for token in split_ip_tokens(text) {
        if let Some(ip) = parse_ip(&token) {
            ips.push(ip.to_string());
        }
    }
    ips
}

/// 按空白和常见分隔符切分，并剥掉 nslookup 等工具附带的标点。
///
/// 例如 `Address:  8.8.8.8`、`8.8.8.8#53`、`(1.1.1.1)`。
fn split_ip_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    for word in text.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '|' | '，' | '；'))
    {
        let trimmed = word.trim_matches(|c: char| {
            matches!(
                c,
                '(' | ')' | '[' | ']' | '{' | '}' | '<' | '>' | '"' | '\'' | ':' | '：'
            )
        });
        if trimmed.is_empty() {
            continue;
        }

        // dig/nslookup 端口后缀：8.8.8.8#53
        let without_port = trimmed.split('#').next().unwrap_or(trimmed);
        tokens.push(without_port.to_string());
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ipv4() {
        assert!(parse_ip("1.2.3.4").is_some());
        assert!(parse_ip("255.255.255.255").is_some());
        assert!(parse_ip("0.0.0.0").is_some());
    }

    #[test]
    fn test_parse_ipv6() {
        assert!(parse_ip("::1").is_some());
        assert!(parse_ip("2001:db8::1").is_some());
        assert!(parse_ip("fe80::1").is_some());
    }

    #[test]
    fn test_invalid() {
        assert!(!is_valid_ip("not_an_ip"));
        assert!(!is_valid_ip("256.1.1.1"));
        assert!(!is_valid_ip(""));
    }

    #[test]
    fn test_extract_ips() {
        let text = "Server: 8.8.8.8 Address: 8.8.8.8#53";
        let ips = extract_ips_from_text(text);
        assert_eq!(ips.len(), 2);
        assert_eq!(ips[0], "8.8.8.8");
    }

    #[test]
    fn test_extract_nslookup_punctuation() {
        let text = "Address:  1.1.1.1";
        assert_eq!(extract_ips_from_text(text), vec!["1.1.1.1".to_string()]);

        let text = "Non-authoritative answer: (8.8.4.4)";
        assert_eq!(extract_ips_from_text(text), vec!["8.8.4.4".to_string()]);
    }
}
