use serde::Deserialize;

use super::ArrowFunctionsOptions;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2015 transform options.
pub struct ES2015Options {
    /// Arrow-function transform options.
    #[serde(skip)]
    pub arrow_function: Option<ArrowFunctionsOptions>,
}
