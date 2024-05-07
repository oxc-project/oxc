use serde::Deserialize;

use crate::CompilerAssumptions;

use super::ObjectRestSpreadOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ES2018Options {
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,

    #[serde(skip)]
    pub assumptions: CompilerAssumptions,
}
