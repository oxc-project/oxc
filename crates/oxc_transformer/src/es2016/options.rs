use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2016Options {
    #[serde(skip)]
    pub exponentiation_operator: bool,
}

impl ES2016Options {
    pub fn with_exponentiation_operator(&mut self, enable: bool) -> &mut Self {
        self.exponentiation_operator = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            exponentiation_operator: can_enable_plugin(
                "transform-exponentiation-operator",
                targets,
                bugfixes,
            ),
        }
    }
}
