//! Ports of internal/utils/string_literal.go helpers, added as rules need them.

/// string_literal.go QuoteSingleStringLiteral.
pub fn quote_single_string_literal(value: &str) -> String {
    let mut b = String::with_capacity(value.len() + 2);
    b.push('\'');
    for ch in value.chars() {
        match ch {
            '\t' => b.push_str("\\t"),
            '\u{000B}' => b.push_str("\\v"),
            '\u{000C}' => b.push_str("\\f"),
            '\u{0008}' => b.push_str("\\b"),
            '\r' => b.push_str("\\r"),
            '\n' => b.push_str("\\n"),
            '\\' => b.push_str("\\\\"),
            '\'' => b.push_str("\\'"),
            '\u{2028}' | '\u{2029}' | '\u{0085}' => write_unicode_escape(&mut b, ch),
            _ => {
                if ch <= '\u{001f}' {
                    write_unicode_escape(&mut b, ch);
                } else {
                    b.push(ch);
                }
            }
        }
    }
    b.push('\'');
    b
}

fn write_unicode_escape(b: &mut String, ch: char) {
    let hex = format!("{:X}", ch as u32);
    b.push_str("\\u");
    for _ in hex.len()..4 {
        b.push('0');
    }
    b.push_str(&hex);
}
