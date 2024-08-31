use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

use super::ArrowFunctionsOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2015Options {
    #[serde(skip)]
    pub arrow_function: Option<ArrowFunctionsOptions>,
}

impl ES2015Options {
    pub fn with_arrow_function(
        &mut self,
        arrow_function: Option<ArrowFunctionsOptions>,
    ) -> &mut Self {
        self.arrow_function = arrow_function;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            arrow_function: can_enable_plugin("transform-arrow-functions", targets, bugfixes)
                .then(Default::default),
        }
    }
}
