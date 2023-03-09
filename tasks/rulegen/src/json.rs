use std::borrow::Cow;

use lazy_static::lazy_static;

/// Convert a javascript object literal to JSON by wrapping the property keys in double quote
pub fn wrap_property_in_quotes(object: &str) -> String {
    use regex::{Captures, Regex};

    lazy_static! {
        static ref IDENT_MATCHER: Regex = Regex::new(r"(?P<ident>[[:alpha:]]\w*)").unwrap();
        static ref DUP_QUOTE_MATCHER: Regex =
            Regex::new(r#"(?P<outer>"(?P<inner>"\w+")")"#).unwrap();
    }

    let add_quote = IDENT_MATCHER
        .replace_all(object, |capture: &Captures| {
            // don't replace true and false, which are json boolean values
            let ident = &capture["ident"];
            if ident == "true" || ident == "false" {
                Cow::Owned(ident.to_string())
            } else {
                Cow::Owned(format!(r#""{ident}""#))
            }
        })
        .into_owned();

    // After the above step, valid json strings will have duplicate quotes now
    // This step removes duplicate quotes.
    let remove_dup_quote = DUP_QUOTE_MATCHER
        .replace_all(&add_quote, |capture: &Captures| Cow::Owned(capture["inner"].to_string()))
        .into_owned();

    remove_dup_quote
}
