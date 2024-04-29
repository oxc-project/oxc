use serde::Deserialize;

use super::ArrowFunctionsOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ES2015Options {
    #[serde(skip)]
    pub arrow_function: Option<ArrowFunctionsOptions>,
}
