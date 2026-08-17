use cow_utils::CowUtils;

/// Normalize the user config path to a watch pattern that can be used to watch for changes.
///
/// Watch pattern like `./oxlintrc.json` is not supported by some editors (VS Code), so we need to normalize it to `oxlintrc.json`.
pub fn normalize_user_config_path_to_watch_pattern(config_path: &str) -> String {
    let path = config_path.cow_replace('\\', "/");
    let path = path.cow_replace("/./", "/");
    let path = path.strip_prefix("./").unwrap_or(&path);

    let mut out = String::with_capacity(path.len());
    for ch in path.chars() {
        match ch {
            // escape path characters that have special meaning in glob patterns
            '*' | '?' | '[' | ']' | '{' | '}' | ',' | '!' => {
                out.push('\\');
                out.push(ch);
            }
            _ => out.push(ch),
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_user_config_path_to_watch_pattern() {
        assert_eq!(normalize_user_config_path_to_watch_pattern("./oxlintrc.json"), "oxlintrc.json");
        assert_eq!(
            normalize_user_config_path_to_watch_pattern(".\\oxlintrc.json"),
            "oxlintrc.json"
        );
        assert_eq!(normalize_user_config_path_to_watch_pattern("oxlintrc.json"), "oxlintrc.json");
        assert_eq!(
            normalize_user_config_path_to_watch_pattern("/home/oxlintrc.json"),
            "/home/oxlintrc.json"
        );
        assert_eq!(
            normalize_user_config_path_to_watch_pattern("C:\\home\\oxlintrc.json"),
            "C:/home/oxlintrc.json"
        );
    }
}
