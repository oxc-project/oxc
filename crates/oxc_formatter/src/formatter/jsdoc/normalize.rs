/// Normalize JSDoc tag aliases to their canonical form.
pub fn normalize_tag_kind(kind: &str) -> &str {
    match kind {
        "return" => "returns",
        "arg" | "argument" => "param",
        "yield" => "yields",
        "prop" => "property",
        "augments" => "extends",
        "constructor" => "class",
        "const" => "constant",
        "defaultvalue" => "default",
        "desc" => "description",
        "host" => "external",
        "fileoverview" | "overview" => "file",
        "emits" => "fires",
        "func" | "method" => "function",
        "var" => "member",
        "virtual" => "abstract",
        "linkcode" | "linkplain" => "link",
        "exception" => "throws",
        _ => kind,
    }
}

/// Capitalize the first ASCII lowercase letter of a string.
/// Skips if the string starts with a backtick (inline code).
pub fn capitalize_first(s: &str) -> String {
    if s.is_empty() || s.starts_with('`') {
        return s.to_string();
    }

    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {
            let mut result = String::with_capacity(s.len());
            result.push(c.to_ascii_uppercase());
            result.push_str(chars.as_str());
            result
        }
        _ => s.to_string(),
    }
}

/// Collapse runs of whitespace to a single space within a type expression, and trim.
pub fn normalize_type_whitespace(type_str: &str) -> String {
    let mut result = String::with_capacity(type_str.len());
    let mut prev_was_space = false;
    for ch in type_str.trim().chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            result.push(ch);
            prev_was_space = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_tag_kind() {
        assert_eq!(normalize_tag_kind("return"), "returns");
        assert_eq!(normalize_tag_kind("arg"), "param");
        assert_eq!(normalize_tag_kind("argument"), "param");
        assert_eq!(normalize_tag_kind("yield"), "yields");
        assert_eq!(normalize_tag_kind("prop"), "property");
        assert_eq!(normalize_tag_kind("param"), "param");
        assert_eq!(normalize_tag_kind("returns"), "returns");
        assert_eq!(normalize_tag_kind("custom"), "custom");
    }

    #[test]
    fn test_capitalize_first() {
        assert_eq!(capitalize_first("hello"), "Hello");
        assert_eq!(capitalize_first("Hello"), "Hello");
        assert_eq!(capitalize_first("`code`"), "`code`");
        assert_eq!(capitalize_first(""), "");
        assert_eq!(capitalize_first("123"), "123");
        assert_eq!(capitalize_first("a"), "A");
    }

    #[test]
    fn test_normalize_type_whitespace() {
        assert_eq!(normalize_type_whitespace("string"), "string");
        assert_eq!(normalize_type_whitespace("  string  |  number  "), "string | number");
        assert_eq!(normalize_type_whitespace("Array< string >"), "Array< string >");
        assert_eq!(normalize_type_whitespace("  a   b  "), "a b");
    }
}
