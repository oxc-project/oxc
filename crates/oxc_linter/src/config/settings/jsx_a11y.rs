use rustc_hash::FxHashMap;
use serde::Deserialize;

/// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations
#[derive(Debug, Deserialize, Default)]
pub struct ESLintSettingsJSXA11y {
    #[serde(rename = "polymorphicPropName")]
    pub polymorphic_prop_name: Option<String>,
    #[serde(default)]
    pub components: FxHashMap<String, String>,
}
