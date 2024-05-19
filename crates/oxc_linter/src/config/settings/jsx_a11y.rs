use rustc_hash::FxHashMap;
use schematic::Config;
use serde::Deserialize;

/// <https://github.com/jsx-eslint/eslint-plugin-jsx-a11y#configurations>
#[derive(Debug, Clone, PartialEq, Deserialize, Config)]
pub struct JSXA11yPluginSettings {
    #[serde(rename = "polymorphicPropName")]
    pub polymorphic_prop_name: Option<String>,
    #[serde(default)]
    pub components: FxHashMap<String, String>,
}
