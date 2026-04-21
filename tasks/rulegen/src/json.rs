use std::borrow::Cow;

use lazy_regex::{Captures, regex};

/// Convert a javascript object literal to JSON by wrapping the property keys in double quotes,
/// converting single quotes to double quotes, and replacing tabs with spaces.
pub fn convert_config_to_json_literal(object: &str) -> String {
    // Only rewrite identifiers that appear to be object keys (start of string or
    // immediately after `{`, `[` or `,`). This avoids touching colons that are
    // part of string contents such as "Note:".
    let ident_matcher = regex!(r"(^\s*|[\{\[,]\s*)(?P<ident>[[:alpha:]]\w*)\s*:");

    let after_ident = ident_matcher.replace_all(object, |capture: &Captures| {
        let prefix = capture.get(1).map_or("", |m| m.as_str());
        let ident = &capture["ident"];
        Cow::Owned(format!(r#"{prefix}"{ident}":"#))
    });
    let comment_matcher = regex!("//.*?\n");
    let whitespace_matcher = regex!(r"(\s)+");
    whitespace_matcher
        .replace_all(&comment_matcher.replace_all(&after_ident, ""), " ")
        .replace('\'', "\"")
        .replace('\n', " ")
        .replace('\t', "  ")
        .trim()
        .trim_end_matches(',')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::convert_config_to_json_literal;

    #[test]
    fn test_single_quotes() {
        let input = "{ foo: 'bar' }";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_tabs_and_single_quotes() {
        let input = "{\tfoo:\t'bar' }";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_multiple_tabs() {
        let input = "{\t\t\tfoo:\t\t\t'bar'\t\t\t\t}";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_multiple_tabs_and_spaces() {
        let input = "{\t  \t\tfoo:\t \t\t'bar'\t  \t\t\t}";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_multiple_spaces() {
        let input = "{   foo:   'bar'   }";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_leading_following_spaces() {
        let input = " {   foo:   'bar'   } ";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\" }");
    }

    #[test]
    fn test_multiple_attributes() {
        let input = r#" { foo: "bar", baz: "qux" } "#;
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": \"bar\", \"baz\": \"qux\" }");
    }

    #[test]
    fn test_trailing_commas() {
        let input = r"{
            foo: true
        },";
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "{ \"foo\": true }");
    }

    #[test]
    fn test_newlines_for_json() {
        let input = r#"[
            {
                foo: true,
                bar: "baz"
            }
        ]"#;
        let out = convert_config_to_json_literal(input);
        // TODO: We want to remove the extra spaces in here.
        assert_eq!(out, "[ { \"foo\": true, \"bar\": \"baz\" } ]");
    }

    #[test]
    fn test_newlines_for_json_with_trailers() {
        let input = r#"[
            {
                foo: true,
                bar: "baz",
            },
        ],"#;
        let out = convert_config_to_json_literal(input);
        // TODO: We want to remove the extra spaces and commas in here.
        assert_eq!(out, "[ { \"foo\": true, \"bar\": \"baz\", }, ]");
    }

    #[test]
    fn test_parsing_with_extra_colon() {
        let input = r#"[{ foo: "Note:" }]"#;
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "[{ \"foo\": \"Note:\" }]");

        let input = r#"[{ foo: "Note:Bar" }]"#;
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "[{ \"foo\": \"Note:Bar\" }]");

        let input = r#"[{ foo: ":foo:" }]"#;
        let out = convert_config_to_json_literal(input);
        assert_eq!(out, "[{ \"foo\": \":foo:\" }]");
    }
}
