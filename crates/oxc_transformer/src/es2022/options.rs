use serde::Deserialize;

use super::ClassPropertiesOptions;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2022 transform options.
pub struct ES2022Options {
    /// Enable class static block transform.
    #[serde(skip)]
    pub class_static_block: bool,

    /// Class properties transform options.
    #[serde(skip)]
    pub class_properties: Option<ClassPropertiesOptions>,

    /// Enable top-level await transform.
    #[serde(skip)]
    pub top_level_await: bool,
}
