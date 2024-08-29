use super::object_rest_spread::ObjectRestSpreadOptions;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2018Options {
    #[serde(skip)]
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,
}

impl ES2018Options {
    #[must_use]
    pub fn with_object_rest_spread(mut self, option: Option<ObjectRestSpreadOptions>) -> Self {
        self.object_rest_spread = option;
        self
    }
}
