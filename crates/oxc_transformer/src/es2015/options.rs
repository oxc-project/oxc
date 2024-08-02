use serde::Deserialize;

use super::ArrowFunctionsOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2015Options {
    #[serde(skip)]
    pub arrow_function: Option<ArrowFunctionsOptions>,
}

impl ES2015Options {
    #[must_use]
    pub fn with_arrow_function(mut self, arrow_function: Option<ArrowFunctionsOptions>) -> Self {
        self.arrow_function = arrow_function;
        self
    }
}
