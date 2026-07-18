use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de};

/// A set of glob patterns.
/// Patterns are matched against paths relative to the configuration file's directory.
#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, JsonSchema)]
pub struct GlobSet(Vec<String>);

impl<'de> Deserialize<'de> for GlobSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let patterns = Vec::<String>::deserialize(deserializer)?;
        for pattern in &patterns {
            validate_glob(pattern).map_err(|reason| {
                de::Error::custom(format_args!("invalid glob `{pattern}`: {reason}"))
            })?;
        }
        Ok(Self::new(patterns))
    }
}

fn validate_glob(pattern: &str) -> Result<(), &'static str> {
    let mut brace_depth = 0_u8;
    let mut in_class = false;
    let mut class_has_item = false;
    let mut class_at_start = false;
    let mut escaped = false;

    for byte in pattern.bytes() {
        if escaped {
            escaped = false;
            if in_class {
                class_has_item = true;
                class_at_start = false;
            }
            continue;
        }

        if byte == b'\\' {
            escaped = true;
            continue;
        }

        if in_class {
            if matches!(byte, b'!' | b'^') && class_at_start {
                class_at_start = false;
            } else if byte == b']' && class_has_item {
                in_class = false;
            } else {
                class_has_item = true;
                class_at_start = false;
            }
            continue;
        }

        match byte {
            b'[' => {
                in_class = true;
                class_has_item = false;
                class_at_start = true;
            }
            b'{' => {
                brace_depth = brace_depth
                    .checked_add(1)
                    .filter(|depth| *depth <= 10)
                    .ok_or("brace expansion nesting exceeds 10 levels")?;
            }
            b'}' if brace_depth > 0 => brace_depth -= 1,
            _ => {}
        }
    }

    if escaped {
        Err("trailing escape")
    } else if in_class {
        Err("unclosed character class")
    } else if brace_depth > 0 {
        Err("unclosed brace expansion")
    } else {
        Ok(())
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
    fn rejects_malformed_globs() {
        for (pattern, reason) in [
            ("src/**/*.{js,ts", "unclosed brace expansion"),
            ("src/[abpp.js", "unclosed character class"),
            ("src/file.js\\", "trailing escape"),
        ] {
            let error = from_value::<GlobSet>(json!([pattern])).unwrap_err().to_string();
            assert!(error.contains(pattern), "{error}");
            assert!(error.contains(reason), "{error}");
        }
    }

    #[test]
    fn accepts_escaped_glob_metacharacters() {
        from_value::<GlobSet>(json!([r"src/\{app\}.js", r"src/\[app\].js"])).unwrap();
    }
}
