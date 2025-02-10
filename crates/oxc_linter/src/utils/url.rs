/// Finds the first value of a query parameter in a URL. Is not guaranteed to be accurate
/// with the URL standard and is just meant to be a simple helper that doesn't require
/// fully parsing the URL.
///
/// # Example
///
/// ```rust
/// find_url_query_value("https://example.com/?foo=bar&baz=qux", "baz") // => Some("qux")
/// ```
pub fn find_url_query_value<'url>(url: &'url str, key: &str) -> Option<&'url str> {
    // Return None right away if this doesn't look like a URL at all.
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return None;
    }
    // Skip everything up to the first `?` as we're not parsing the host/path/etc.
    let url = url.split('?').nth(1)?;
    // Now parse the query string in pairs of `key=value`, we don't need
    // to be too strict about this as we're not trying to be spec-compliant.
    for pair in url.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v);
            }
        }
    }
    None
}

mod test {
    #[test]
    fn test_find_url_query_value() {
        use super::find_url_query_value;
        assert_eq!(find_url_query_value("something", "q"), None);
        assert_eq!(find_url_query_value("https://example.com/?foo=bar", "foo"), Some("bar"));
        assert_eq!(find_url_query_value("https://example.com/?foo=bar", "baz"), None);
        assert_eq!(
            find_url_query_value("https://example.com/?foo=bar&baz=qux", "baz"),
            Some("qux")
        );
        assert_eq!(
            find_url_query_value("https://example.com/?foo=bar&foo=qux", "foo"),
            Some("bar")
        );
        assert_eq!(
            find_url_query_value("https://polyfill.io/v3/polyfill.min.js?features=WeakSet%2CPromise%2CPromise.prototype.finally%2Ces2015%2Ces5%2Ces6", "features"),
            Some("WeakSet%2CPromise%2CPromise.prototype.finally%2Ces2015%2Ces5%2Ces6")
        );
    }
}
