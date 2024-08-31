use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2019Options {
    #[serde(skip)]
    pub optional_catch_binding: bool,
}

impl ES2019Options {
    pub fn with_optional_catch_binding(&mut self, enable: bool) -> &mut Self {
        self.optional_catch_binding = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            optional_catch_binding: can_enable_plugin(
                "transform-optional-catch-binding",
                targets,
                bugfixes,
            ),
        }
    }
}
