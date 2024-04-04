use serde::Deserialize;

use crate::options::default_as_true;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Es2020Options {
    /// https://babeljs.io/docs/babel-plugin-proposal-dynamic-import
    pub dynamic_import: bool,

    /// https://babeljs.io/docs/babel-plugin-transform-export-namespace-from
    #[serde(default = "default_as_true")]
    pub export_namespace_from: bool,

    /// https://babeljs.io/docs/babel-plugin-transform-nullish-coalescing-operator
    #[serde(default = "default_as_true")]
    pub nullish_coalescing_operator: bool,
}

impl Default for Es2020Options {
    fn default() -> Self {
        Self {
            dynamic_import: false, // Let bundlers handle it!
            export_namespace_from: default_as_true(),
            nullish_coalescing_operator: default_as_true(),
        }
    }
}
