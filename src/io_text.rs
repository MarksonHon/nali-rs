//! 标准输入解码。
//!
//! 优先按 UTF-8（含 BOM）解释；若字节流不是合法 UTF-8，则在 Windows 上
//! 回退到当前 ANSI 代码页（简体中文为 GBK/CP936，繁体中文为 Big5/CP950 等）。
//! 非 Windows 平台回退为 Latin-1，保证字节不会被丢弃。

use std::io::{self, Read};

use encoding_rs::Encoding;

/// 让控制台按 UTF-8 显示输出。
///
/// PowerShell 7 会把外部程序的标准输出当成 UTF-8。若仍按 GBK 写回，
/// 解码失败后管道结果不会显示（只剩 nslookup 写到 stderr 的那一行）。
pub fn prepare_console() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{SetConsoleCP, SetConsoleOutputCP};
        unsafe {
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);
        }
    }
}

/// 标准输入解码结果：文本，以及它原来是否已是 UTF-8。
pub struct DecodedInput {
    pub text: String,
    /// 为 true 时输出保持 UTF-8；为 false 时按同一遗留代码页写回。
    pub was_utf8: bool,
}

/// 读取全部标准输入并解码为 Unicode 文本。
pub fn read_stdin_text() -> io::Result<DecodedInput> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    Ok(decode_input(&bytes))
}

/// 将原始字节解码为字符串，并记录是否本来就是 UTF-8。
pub fn decode_input(bytes: &[u8]) -> DecodedInput {
    if bytes.is_empty() {
        return DecodedInput {
            text: String::new(),
            was_utf8: true,
        };
    }

    let without_bom = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    if let Ok(text) = std::str::from_utf8(without_bom) {
        return DecodedInput {
            text: text.to_string(),
            was_utf8: true,
        };
    }

    DecodedInput {
        text: decode_legacy(bytes),
        was_utf8: false,
    }
}

/// 按输入原来的编码写回。
///
/// UTF-8 输入保持 UTF-8。GBK/Big5 等遗留输入按当前 ANSI 代码页编码，
/// 这样 Windows `nslookup`（代码页 936）的管道在 cmd 里不会乱码。
pub fn encode_like_input(text: &str, was_utf8: bool) -> Vec<u8> {
    if was_utf8 {
        return text.as_bytes().to_vec();
    }
    let encoding = legacy_encoding();
    if encoding == encoding_rs::UTF_8 {
        return text.as_bytes().to_vec();
    }
    let (bytes, _, _) = encoding.encode(text);
    bytes.into_owned()
}

/// 将原始字节解码为字符串。
///
/// 合法 UTF-8（可带 BOM）原样返回；否则使用平台默认遗留编码。
pub fn decode_bytes(bytes: &[u8]) -> String {
    decode_input(bytes).text
}

fn decode_legacy(bytes: &[u8]) -> String {
    let encoding = legacy_encoding();
    let (text, _, _) = encoding.decode(bytes);
    text.into_owned()
}

fn legacy_encoding() -> &'static Encoding {
    #[cfg(windows)]
    {
        ansi_code_page_encoding().unwrap_or(encoding_rs::GBK)
    }
    #[cfg(not(windows))]
    {
        encoding_rs::WINDOWS_1252
    }
}

/// 将 Windows ANSI 代码页映射到 encoding_rs 编码。
///
/// 覆盖简体中文 GBK、繁体中文 Big5，以及常见西欧/中欧/日韩代码页。
#[cfg(windows)]
fn ansi_code_page_encoding() -> Option<&'static Encoding> {
    use windows_sys::Win32::Globalization::GetACP;

    let cp = unsafe { GetACP() };
    match cp {
        936 => Some(encoding_rs::GBK),
        950 => Some(encoding_rs::BIG5),
        54936 => Some(encoding_rs::GB18030),
        932 => Some(encoding_rs::SHIFT_JIS),
        949 => Some(encoding_rs::EUC_KR),
        1250 => Some(encoding_rs::WINDOWS_1250),
        1251 => Some(encoding_rs::WINDOWS_1251),
        1252 => Some(encoding_rs::WINDOWS_1252),
        1253 => Some(encoding_rs::WINDOWS_1253),
        1254 => Some(encoding_rs::WINDOWS_1254),
        1255 => Some(encoding_rs::WINDOWS_1255),
        1256 => Some(encoding_rs::WINDOWS_1256),
        1257 => Some(encoding_rs::WINDOWS_1257),
        1258 => Some(encoding_rs::WINDOWS_1258),
        65001 => Some(encoding_rs::UTF_8),
        _ => Encoding::for_label(format!("cp{cp}").as_bytes())
            .or_else(|| Encoding::for_label(format!("windows-{cp}").as_bytes())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_roundtrip() {
        let raw = "服务器: dns.example.com\nAddress: 1.1.1.1\n".as_bytes();
        assert_eq!(decode_bytes(raw), "服务器: dns.example.com\nAddress: 1.1.1.1\n");
    }

    #[test]
    fn utf8_bom_is_stripped() {
        let mut raw = vec![0xEF, 0xBB, 0xBF];
        raw.extend_from_slice("1.2.3.4".as_bytes());
        assert_eq!(decode_bytes(&raw), "1.2.3.4");
    }

    #[test]
    fn gbk_fallback_when_not_utf8() {
        // "服务器" 的 GBK 编码，不是合法 UTF-8
        let raw = [0xB7, 0xFE, 0xCE, 0xF1, 0xC6, 0xF7, b':', b' ', b'1', b'.', b'1', b'.', b'1', b'.', b'1'];
        let text = {
            let (cow, _, _) = encoding_rs::GBK.decode(&raw);
            cow.into_owned()
        };
        // 仅在当前 ANSI 代码页为 GBK（936）时，回退结果应与 GBK 解码一致。
        // 其他代码页下只断言：非法 UTF-8 不会 panic，且 IP 数字仍保留。
        let decoded = decode_input(&raw);
        assert!(!decoded.was_utf8);
        assert!(decoded.text.contains("1.1.1.1"), "decoded={:?}", decoded.text);
        #[cfg(windows)]
        {
            use windows_sys::Win32::Globalization::GetACP;
            if unsafe { GetACP() } == 936 {
                assert_eq!(decoded.text, text);
                let encoded = encode_like_input(&decoded.text, false);
                assert_eq!(encoded, raw);
            }
        }
        let _ = text;
    }
}
