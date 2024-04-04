use serde::Deserialize;

use crate::options::default_as_true;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2024Options {
    /// https://babeljs.io/docs/babel-plugin-transform-unicode-sets-regex
    #[serde(default = "default_as_true")]
    pub unicode_sets_regex: bool,
}

impl Default for Es2024Options {
    fn default() -> Self {
        Self { unicode_sets_regex: default_as_true() }
    }
}
