use std::borrow::Cow;

use lazy_regex::{Captures, regex};

/// Convert a javascript object literal to JSON by wrapping the property keys in double quotes,
/// converting single quotes to double quotes, and replacing tabs with spaces.
pub fn convert_config_to_json_literal(object: &str) -> String {
    let ident_matcher = regex!(r"(?P<ident>[[:alpha:]]\w*\s*):");

    let after_ident = ident_matcher.replace_all(object, |capture: &Captures| {
        let ident = &capture["ident"];
        Cow::Owned(format!(r#""{ident}":"#))
    });
    let comment_matcher = regex!("//.*?\n");
    let whitespace_matcher = regex!(r"(\t| )+");
    whitespace_matcher
        .replace_all(&comment_matcher.replace_all(&after_ident, ""), " ")
        .replace('\'', "\"")
        .replace('\n', " ")
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
}
