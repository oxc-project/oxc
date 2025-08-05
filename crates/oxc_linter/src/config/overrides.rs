use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    path::Path,
};

use fast_glob::glob_match;
use schemars::{JsonSchema, r#gen, schema::Schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::{LintPlugins, OxlintEnv, OxlintGlobals, config::OxlintRules};

// nominal wrapper required to add JsonSchema impl
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct OxlintOverrides(Vec<OxlintOverride>);

impl Deref for OxlintOverrides {
    type Target = Vec<OxlintOverride>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OxlintOverrides {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for OxlintOverrides {
    type Item = OxlintOverride;
    type IntoIter = <Vec<OxlintOverride> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a OxlintOverrides {
    type Item = &'a OxlintOverride;
    type IntoIter = std::slice::Iter<'a, OxlintOverride>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl OxlintOverrides {
    #[inline]
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    // must be explicitly defined to make serde happy
    /// Returns `true` if the overrides list has no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl JsonSchema for OxlintOverrides {
    fn schema_name() -> String {
        "OxlintOverrides".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintOverrides")
    }

    fn json_schema(r#gen: &mut r#gen::SchemaGenerator) -> Schema {
        r#gen.subschema_for::<Vec<OxlintOverride>>()
    }
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[non_exhaustive]
pub struct OxlintOverride {
    /// A list of glob patterns to override.
    ///
    /// ## Example
    /// `[ "*.test.ts", "*.spec.ts" ]`
    pub files: GlobSet,

    /// Environments enable and disable collections of global variables.
    pub env: Option<OxlintEnv>,

    /// Enabled or disabled specific global variables.
    pub globals: Option<OxlintGlobals>,

    /// Optionally change what plugins are enabled for this override. When
    /// omitted, the base config's plugins are used.
    #[serde(default)]
    pub plugins: Option<LintPlugins>,

    #[serde(default)]
    pub rules: OxlintRules,
}

/// A glob pattern.
///
/// Thin wrapper around pattern matching using `fast-glob` because we need to implement Serialize and schemars
/// traits.
#[derive(Clone, Default)]
pub struct GlobSet {
    /// Raw patterns from the config. Used for pattern matching and serialization.
    raw: Vec<String>,
}

impl GlobSet {
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(
        patterns: I,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let patterns = patterns.into_iter();
        let size_hint = patterns.size_hint();

        let mut raw = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));

        for pattern in patterns {
            let pattern = pattern.as_ref();
            // Validate the pattern by testing it against an empty string
            // This helps catch invalid glob patterns early
            let _ = glob_match(pattern, "");
            raw.push(pattern.to_string());
        }

        Ok(Self { raw })
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        self.raw.iter().any(|pattern| {
            // First try direct match
            if glob_match(pattern, path_str.as_ref()) {
                return true;
            }
            
            // For patterns that don't start with ** and don't contain /, 
            // also try matching with **/ prefix to be compatible with globset behavior
            if !pattern.starts_with("**/") && !pattern.contains('/') {
                let prefixed_pattern = format!("**/{}", pattern);
                if glob_match(&prefixed_pattern, path_str.as_ref()) {
                    return true;
                }
            }
            
            false
        })
    }
}

impl std::fmt::Debug for GlobSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("GlobSet").field(&self.raw).finish()
    }
}

impl Serialize for GlobSet {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for GlobSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let globs = Vec::<String>::deserialize(deserializer)?;
        Self::new(globs).map_err(de::Error::custom)
    }
}

impl JsonSchema for GlobSet {
    fn schema_name() -> String {
        Self::schema_id().into()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("GlobSet")
    }

    fn json_schema(r#gen: &mut r#gen::SchemaGenerator) -> Schema {
        r#gen.subschema_for::<Vec<String>>()
    }
}

#[cfg(test)]
mod test {
    use crate::config::{globals::GlobalValue, plugins::BuiltinLintPlugins};

    use super::*;
    use rustc_hash::FxHashSet;
    use serde_json::{from_value, json};

    #[test]
    fn test_globset() {
        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx",],
        }))
        .unwrap();
        assert!(config.files.is_match("/some/path/foo.tsx"));
        assert!(!config.files.is_match("/some/path/foo.ts"));

        let config: OxlintOverride = from_value(json!({
            "files": ["lib/*.ts",],
        }))
        .unwrap();
        assert!(config.files.is_match("lib/foo.ts"));
        assert!(!config.files.is_match("src/foo.ts"));
    }

    #[test]
    fn test_parsing_plugins() {
        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
        }))
        .unwrap();
        assert_eq!(config.plugins, None);

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "plugins": [],
        }))
        .unwrap();
        assert_eq!(
            config.plugins,
            Some(LintPlugins::new(BuiltinLintPlugins::empty(), FxHashSet::default()))
        );

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "plugins": ["typescript", "react"],
        }))
        .unwrap();
        assert_eq!(
            config.plugins,
            Some(LintPlugins::new(
                BuiltinLintPlugins::REACT | BuiltinLintPlugins::TYPESCRIPT,
                FxHashSet::default()
            ))
        );
    }

    #[test]
    fn test_parsing_globals() {
        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
        }))
        .unwrap();
        assert!(config.globals.is_none());

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "globals": {
                "Foo": "readable"
            },
        }))
        .unwrap();

        assert_eq!(*config.globals.unwrap().get("Foo").unwrap(), GlobalValue::Readonly);
    }

    #[test]
    fn test_parsing_env() {
        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
        }))
        .unwrap();
        assert!(config.env.is_none());

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "env": {
                "es2022": true,
                "es2023": false,
            },
        }))
        .unwrap();

        let env = &config.env.unwrap();
        assert!(env.contains("es2022"));
        assert!(!env.contains("es2023"));
    }
}
