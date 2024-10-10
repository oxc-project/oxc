use std::borrow::Cow;

use serde::Deserialize;

#[derive(Default, Clone, Copy, Debug, Deserialize)]
pub enum HelperLoaderMode {
    /// Babel default mode
    Inline,
    /// Babel test default mode
    External,
    /// Like @babel/plugin-transform-runtime does
    #[default]
    Runtime,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HelperLoaderOptions {
    #[serde(default = "default_as_module_name")]
    pub module_name: Cow<'static, str>,
    pub mode: HelperLoaderMode,
}

impl Default for HelperLoaderOptions {
    fn default() -> Self {
        Self { module_name: default_as_module_name(), mode: HelperLoaderMode::default() }
    }
}

fn default_as_module_name() -> Cow<'static, str> {
    Cow::Borrowed("@babel/runtime")
}
