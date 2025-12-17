use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use rustc_hash::FxHashSet;
use schemars::{JsonSchema, r#gen, schema::Schema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
#[serde(deny_unknown_fields)]
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

    /// JS plugins for this override.
    ///
    /// Note: JS plugins are experimental and not subject to semver.
    /// They are not supported in language server at present.
    #[serde(
        rename = "jsPlugins",
        deserialize_with = "deserialize_external_plugins_override",
        serialize_with = "serialize_external_plugins_override",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "Option<FxHashSet<String>>")]
    pub external_plugins: Option<
        FxHashSet<(PathBuf /* config file directory */, String /* plugin specifier */)>,
    >,

    #[serde(default)]
    pub rules: OxlintRules,
}

/// A set of glob patterns.
#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
pub struct GlobSet(Vec<String>);

impl<'de> Deserialize<'de> for GlobSet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::new(Vec::<String>::deserialize(deserializer)?))
    }
}

impl GlobSet {
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(patterns: I) -> Self {
        Self(
            patterns
                .into_iter()
                .map(|pat| {
                    let pattern = pat.as_ref();
                    if pattern.contains('/') {
                        pattern.to_owned()
                    } else {
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

fn deserialize_external_plugins_override<'de, D>(
    deserializer: D,
) -> Result<Option<FxHashSet<(PathBuf, String)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_set: Option<FxHashSet<String>> = Option::deserialize(deserializer)?;
    Ok(opt_set
        .map(|set| set.into_iter().map(|specifier| (PathBuf::default(), specifier)).collect()))
}

#[expect(clippy::ref_option)]
fn serialize_external_plugins_override<S>(
    plugins: &Option<FxHashSet<(PathBuf, String)>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize as an array of original specifiers (the values in the map)
    match plugins {
        Some(set) => serializer.collect_seq(set.iter().map(|(_, specifier)| specifier)),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod test {
    use crate::config::{globals::GlobalValue, plugins::LintPlugins};

    use super::*;
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
        assert_eq!(config.plugins, Some(LintPlugins::empty()));

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "plugins": ["typescript", "react"],
        }))
        .unwrap();
        assert_eq!(config.plugins, Some(LintPlugins::REACT | LintPlugins::TYPESCRIPT));
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
