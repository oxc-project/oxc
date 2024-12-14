use std::{borrow::Cow, ops::Deref, path::Path};

use nonmax::NonMaxU32;
use schemars::{gen, schema::Schema, JsonSchema};
use serde::{de, ser, Deserialize, Serialize};

use oxc_index::{Idx, IndexVec};

use crate::{config::OxlintRules, LintPlugins};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct OverrideId(NonMaxU32);

impl Idx for OverrideId {
    #[allow(clippy::cast_possible_truncation)]
    fn from_usize(idx: usize) -> Self {
        assert!(idx < u32::MAX as usize);
        // SAFETY: We just checked `idx` is a legal value for `NonMaxU32`
        Self(unsafe { NonMaxU32::new_unchecked(idx as u32) })
    }

    fn index(self) -> usize {
        self.0.get() as usize
    }
}

// nominal wrapper required to add JsonSchema impl
#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct OxlintOverrides(IndexVec<OverrideId, OxlintOverride>);

impl Deref for OxlintOverrides {
    type Target = IndexVec<OverrideId, OxlintOverride>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl OxlintOverrides {
    #[inline]
    pub fn empty() -> Self {
        Self(IndexVec::new())
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

    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema {
        gen.subschema_for::<Vec<OxlintOverride>>()
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

    /// Optionally change what plugins are enabled for this override. When
    /// omitted, the base config's plugins are used.
    #[serde(default)]
    pub plugins: Option<LintPlugins>,

    #[serde(default)]
    pub rules: OxlintRules,
}

/// A glob pattern.
///
/// Thin wrapper around [`globset::GlobSet`] because that struct doesn't implement Serialize or schemars
/// traits.
#[derive(Clone, Debug, Default)]
pub struct GlobSet {
    /// Raw patterns from the config. Inefficient, but required for [serialization](Serialize),
    /// which in turn is required for `--print-config`.
    raw: Vec<String>,
    globs: globset::GlobSet,
}

impl GlobSet {
    pub fn new<S: AsRef<str>, I: IntoIterator<Item = S>>(
        patterns: I,
    ) -> Result<Self, globset::Error> {
        let patterns = patterns.into_iter();
        let size_hint = patterns.size_hint();

        let mut builder = globset::GlobSetBuilder::new();
        let mut raw = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0));

        for pattern in patterns {
            let pattern = pattern.as_ref();
            let glob = globset::Glob::new(pattern)?;
            builder.add(glob);
            raw.push(pattern.to_string());
        }

        let globs = builder.build()?;
        Ok(Self { raw, globs })
    }

    pub fn is_match<P: AsRef<Path>>(&self, path: P) -> bool {
        self.globs.is_match(path)
    }
}

impl ser::Serialize for GlobSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.raw.serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for GlobSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
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

    fn json_schema(gen: &mut gen::SchemaGenerator) -> Schema {
        gen.subschema_for::<Vec<String>>()
    }
}

mod test {
    #[test]
    fn test_globset() {
        use serde_json::{from_value, json};

        use super::*;

        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx",],
        }))
        .unwrap();
        assert!(config.files.globs.is_match("/some/path/foo.tsx"));
        assert!(!config.files.globs.is_match("/some/path/foo.ts"));

        let config: OxlintOverride = from_value(json!({
            "files": ["lib/*.ts",],
        }))
        .unwrap();
        assert!(config.files.globs.is_match("lib/foo.ts"));
        assert!(!config.files.globs.is_match("src/foo.ts"));
    }

    #[test]
    fn test_parsing_plugins() {
        use serde_json::{from_value, json};

        use super::*;

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
}
