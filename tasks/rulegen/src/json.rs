use std::borrow::Cow;

use lazy_static::lazy_static;

/// Convert a javascript object literal to JSON by wrapping the property keys in double quote
pub fn wrap_property_in_quotes(object: &str) -> String {
    use regex::{Captures, Regex};

    lazy_static! {
        static ref IDENT_MATCHER: Regex = Regex::new(r"(?P<ident>[[:alpha:]]\w*\s*):").unwrap();
        static ref DUP_QUOTE_MATCHER: Regex =
            Regex::new(r#"(?P<outer>"(?P<inner>"\w+")")"#).unwrap();
    }

    let add_quote = IDENT_MATCHER
        .replace_all(object, |capture: &Captures| {
            let ident = &capture["ident"];
            Cow::Owned(format!(r#""{ident}":"#))
        })
        .into_owned();
    add_quote
}
