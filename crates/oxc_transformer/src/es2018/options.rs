use serde::Deserialize;

use super::ObjectRestSpreadOptions;

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// ES2018 transform options.
pub struct ES2018Options {
    /// Object rest/spread transform options.
    #[serde(skip)]
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,

    /// Enable async generator function transform.
    #[serde(skip)]
    pub async_generator_functions: bool,
}
