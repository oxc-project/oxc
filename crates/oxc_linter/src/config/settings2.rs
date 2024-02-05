use rustc_hash::FxHashMap;
use serde::Deserialize;

/// The `settings` field from ESLint config
///
/// An object containing name-value pairs of information that should be available to all rules
///
/// TS type is `Object`
/// https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L53
/// But each plugin extends this with their own properties.
#[derive(Debug, Deserialize)]
pub struct ESLintSettings {
    // react: ,
    #[serde(default)]
    #[serde(rename = "jsx-a11y")]
    jsx_a11y: ESLintSettingsJSXA11y,
    // nextjs: ,
}

/// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations
#[derive(Debug, Deserialize)]
struct ESLintSettingsJSXA11y {
    #[serde(rename(deserialize = "polymorphicPropName"))]
    polymorphic_prop_name: Option<String>,
    #[serde(default)]
    components: FxHashMap<String, String>,
}

impl Default for ESLintSettingsJSXA11y {
    fn default() -> Self {
        Self { polymorphic_prop_name: None, components: FxHashMap::default() }
    }
}

// https://github.com/jsx-eslint/eslint-plugin-react#configuration-legacy-eslintrc-
// https://nextjs.org/docs/pages/building-your-application/configuring/eslint#eslint-plugin

impl Default for ESLintSettings {
    fn default() -> Self {
        Self { jsx_a11y: ESLintSettingsJSXA11y::default() }
    }
}
