use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Validate a single glob pattern.
///
/// `fast_glob::glob_match` does not validate patterns,
/// so user-written patterns must be rejected up front with this.
///
/// # Errors
/// Returns the error message when the pattern is not a valid glob.
pub fn validate_glob_pattern(pattern: &str) -> Result<(), String> {
    fast_glob::validate(pattern).map_err(|err| format!("Invalid glob pattern `{pattern}`: {err}"))
}

/// A set of glob patterns.
/// Patterns are matched against paths relative to the configuration file's directory.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, JsonSchema)]
pub struct GlobSet(Vec<String>);

impl<'de> Deserialize<'de> for GlobSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::try_new(Vec::<String>::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl GlobSet {
    /// Returns `true` when the glob set has no patterns.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(patterns: I) -> Self {
        Self(
            patterns
                .into_iter()
                .map(|pat| {
                    let pattern = pat.as_ref();
                    // Normalize patterns starting with "./" to remove the prefix
                    // since paths are matched relative to the config file's directory
                    let (pattern, had_dot_slash) =
                        pattern.strip_prefix("./").map_or((pattern, false), |s| (s, true));

                    if pattern.contains('/') {
                        pattern.to_owned()
                    } else if had_dot_slash {
                        // Pattern started with "./", treat as literal path relative to config
                        pattern.to_owned()
                    } else {
                        // Pattern has no path separator, make it recursive
                        let mut s = String::with_capacity(pattern.len() + 3);
                        s.push_str("**/");
                        s.push_str(pattern);
                        s
                    }
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Like [`Self::new`], but validates each pattern as written by the user, before normalization.
    /// Deserialization goes through this.
    fn try_new<S: AsRef<str>, I: IntoIterator<Item = S>>(patterns: I) -> Result<Self, String> {
        let patterns = patterns.into_iter().collect::<Vec<_>>();
        for pattern in &patterns {
            validate_glob_pattern(pattern.as_ref())?;
        }
        Ok(Self::new(patterns))
    }

    pub fn is_match(&self, path: &str) -> bool {
        self.0.iter().any(|glob| fast_glob::glob_match(glob, path))
    }
}

#[cfg(test)]
mod test {
    use serde_json::{from_value, json};

    use super::GlobSet;

    #[test]
    fn test_globset() {
        let glob_set: GlobSet = from_value(json!(["*.tsx"])).unwrap();
        assert!(glob_set.is_match("/some/path/foo.tsx"));
        assert!(!glob_set.is_match("/some/path/foo.ts"));

        let glob_set: GlobSet = from_value(json!(["lib/*.ts"])).unwrap();
        assert!(glob_set.is_match("lib/foo.ts"));
        assert!(!glob_set.is_match("src/foo.ts"));

        // Test that patterns with "./" prefix are normalized
        // Fixes https://github.com/oxc-project/oxc/issues/18952
        let glob_set: GlobSet = from_value(json!(["./index.js"])).unwrap();
        assert!(glob_set.is_match("index.js"));
        assert!(!glob_set.is_match("src/index.js"));

        let glob_set: GlobSet = from_value(json!(["./src/*.ts"])).unwrap();
        assert!(glob_set.is_match("src/foo.ts"));
        assert!(!glob_set.is_match("lib/foo.ts"));

        // Test "./*.js" pattern - should match only files in current directory
        let glob_set: GlobSet = from_value(json!(["./*.js"])).unwrap();
        assert!(glob_set.is_match("file.js"));
        assert!(!glob_set.is_match("src/file.js"));
        assert!(!glob_set.is_match("nested/dir/file.js"));

        // Test "./**/*.js" pattern - should match .js files in all subdirectories
        let glob_set: GlobSet = from_value(json!(["./**/*.js"])).unwrap();
        assert!(glob_set.is_match("src/file.js"));
        assert!(glob_set.is_match("nested/dir/file.js"));
        assert!(glob_set.is_match("file.js"));
        assert!(!glob_set.is_match("file.ts"));

        // Test that patterns with "../" prefix are kept as-is (not normalized)
        let glob_set: GlobSet = from_value(json!(["../foo.js"])).unwrap();
        assert!(glob_set.is_match("../foo.js"));
        assert!(!glob_set.is_match("foo.js"));
    }

    #[test]
    fn test_globset_invalid_pattern() {
        // Invalid patterns must be rejected at deserialization time.
        // Fixes https://github.com/oxc-project/oxc/issues/24612
        let err = from_value::<GlobSet>(json!(["src/**/*.{js,ts"])).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Invalid glob pattern `src/**/*.{js,ts`"), "{message}");
        assert!(message.contains("unclosed brace expansion"), "{message}");

        let err = from_value::<GlobSet>(json!(["src/[abpp.js"])).unwrap_err();
        assert!(err.to_string().contains("unclosed character class"), "{}", err.to_string());

        // One invalid pattern rejects the whole set, even alongside valid ones
        assert!(from_value::<GlobSet>(json!(["*.ts", "src/**/*.{js,ts"])).is_err());
    }
}
