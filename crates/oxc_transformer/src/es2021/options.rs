use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2021Options {
    #[serde(skip)]
    pub logical_assignment_operators: bool,
}

impl ES2021Options {
    pub fn with_logical_assignment_operators(&mut self, enable: bool) -> &mut Self {
        self.logical_assignment_operators = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            logical_assignment_operators: can_enable_plugin(
                "transform-logical-assignment-operators",
                targets,
                bugfixes,
            ),
        }
    }
}
