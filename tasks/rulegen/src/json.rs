use std::borrow::Cow;

use lazy_static::lazy_static;

/// Convert a javascript object literal to JSON by wrapping the property keys in double quote,
/// and convert the single quote to a double quote.
pub fn convert_config_to_json_literal(object: &str) -> String {
    use regex::{Captures, Regex};

    lazy_static! {
        static ref IDENT_MATCHER: Regex = Regex::new(r"(?P<ident>[[:alpha:]]\w*\s*):").unwrap();
    }

    let add_quote = IDENT_MATCHER
        .replace_all(object, |capture: &Captures| {
            let ident = &capture["ident"];
            Cow::Owned(format!(r#""{ident}":"#))
        })
        .replace('\'', "\"")
        .replace('\n', "");
    add_quote
}
