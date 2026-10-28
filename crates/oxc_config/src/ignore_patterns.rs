/// Validate a single config `ignorePatterns` entry.
///
/// Matching is rooted at the config file's directory and gitignore semantics cannot reach outside that root,
/// so a `..` component can never match anything.
/// Reject it with an error message instead of silently accepting a dead pattern.
///
/// # Errors
/// Returns the error message when the pattern contains a `..` path segment.
pub fn validate_ignore_pattern(pattern: &str) -> Result<(), String> {
    let path_part = pattern.strip_prefix('!').unwrap_or(pattern);
    // gitignore ignores unescaped trailing spaces; a trailing `\ ` escapes a literal space
    let path_part = if path_part.ends_with("\\ ") { path_part } else { path_part.trim_end() };
    // Split by `/` instead of `Path::components()`,
    // since gitignore patterns use `/` as the separator on all platforms.
    if path_part.split('/').any(|segment| segment == "..") {
        return Err(format!(
            "Invalid pattern `{pattern}` in `ignorePatterns`: `..` is not supported, patterns are resolved within the config file's directory"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::validate_ignore_pattern;

    #[test]
    fn rejects_parent_dir_segments() {
        for pattern in ["../src", "!../src", "src/../dist", "..", "../", ".. ", "src/.. "] {
            assert!(
                validate_ignore_pattern(pattern).is_err(),
                "pattern `{pattern}` should be rejected"
            );
        }
    }

    #[test]
    fn accepts_patterns_without_parent_dir_segments() {
        // `..` is only special as a whole path segment.
        // `..\ ` (escaped trailing space) literally matches a file named `.. `, not a parent directory.
        for pattern in ["dist", "*.min.js", "**/dist", "!dist", "..foo", "a..b", "foo..", "..\\ "] {
            assert!(
                validate_ignore_pattern(pattern).is_ok(),
                "pattern `{pattern}` should be accepted"
            );
        }
    }
}
