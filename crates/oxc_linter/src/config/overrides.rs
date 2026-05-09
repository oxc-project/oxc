use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxHashSet;
use schemars::{JsonSchema, r#gen, schema::Schema};
use serde::{Deserialize, Serialize};

use oxc_config::GlobSet;

use crate::{LintPlugins, OxlintEnv, OxlintGlobals, config::OxlintRules};

use super::external_plugins::{ExternalPluginEntry, external_plugins_schema};

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

    /// A list of glob patterns to exclude from this override.
    ///
    /// Files matching these patterns are not globally ignored; this override
    /// simply does not apply to them.
    ///
    /// ## Example
    /// `[ "*.generated.ts", "fixtures/**" ]`
    #[serde(default, skip_serializing_if = "GlobSet::is_empty")]
    #[schemars(skip)]
    pub ignores: GlobSet,

    /// Environments enable and disable collections of global variables.
    pub env: Option<OxlintEnv>,

    /// Enabled or disabled specific global variables.
    pub globals: Option<OxlintGlobals>,

    /// Optionally change what plugins are enabled for this override. When
    /// omitted, the base config's plugins are used.
    #[serde(default)]
    pub plugins: Option<LintPlugins>,

    /// JS plugins for this override, allows usage of ESLint plugins with Oxlint.
    ///
    /// Read more about JS plugins in
    /// [the docs](https://oxc.rs/docs/guide/usage/linter/js-plugins.html).
    ///
    /// Note: JS plugins are in alpha and not subject to semver.
    #[serde(rename = "jsPlugins", default, skip_serializing_if = "Option::is_none")]
    #[schemars(schema_with = "external_plugins_schema")]
    pub external_plugins: Option<FxHashSet<ExternalPluginEntry>>,

    #[serde(default)]
    pub rules: OxlintRules,
}

#[cfg(test)]
mod test {
    use crate::config::{globals::GlobalValue, plugins::LintPlugins};

    use super::*;
    use serde_json::{from_value, json};

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
    fn test_parsing_ignores() {
        let config: OxlintOverride = from_value(json!({
            "files": ["*.tsx"],
            "ignores": ["*.generated.tsx"],
        }))
        .unwrap();

        assert!(config.ignores.is_match("App.generated.tsx"));
        assert!(!config.ignores.is_match("App.tsx"));
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
