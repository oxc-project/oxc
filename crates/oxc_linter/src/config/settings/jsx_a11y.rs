use oxc_span::CompactStr;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;

// <https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations>
#[derive(Debug, Deserialize, Default, JsonSchema)]
pub struct JSXA11yPluginSettings {
    #[serde(rename = "polymorphicPropName")]
    pub polymorphic_prop_name: Option<CompactStr>,
    #[serde(default)]
    pub components: FxHashMap<CompactStr, CompactStr>,
}
