use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2020Options {
    #[serde(skip)]
    pub nullish_coalescing_operator: bool,
}

impl ES2020Options {
    pub fn with_nullish_coalescing_operator(&mut self, enable: bool) -> &mut Self {
        self.nullish_coalescing_operator = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            nullish_coalescing_operator: can_enable_plugin(
                "transform-nullish-coalescing-operator",
                targets,
                bugfixes,
            ),
        }
    }
}
