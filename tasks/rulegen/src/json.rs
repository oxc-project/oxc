use std::borrow::Cow;

use lazy_regex::{Captures, regex};

/// Convert a javascript object literal to JSON by wrapping the property keys in double quote,
/// and convert the single quote to a double quote.
pub fn convert_config_to_json_literal(object: &str) -> String {
    let ident_matcher = regex!(r"(?P<ident>[[:alpha:]]\w*\s*):");

    ident_matcher
        .replace_all(object, |capture: &Captures| {
            let ident = &capture["ident"];
            Cow::Owned(format!(r#""{ident}":"#))
        })
        .replace('\'', "\"")
        .replace('\n', "")
}
