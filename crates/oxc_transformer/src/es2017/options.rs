use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2017Options {
    #[serde(skip)]
    pub async_to_generator: bool,
}

impl ES2017Options {
    pub fn with_async_to_generator(&mut self, enable: bool) -> &mut Self {
        self.async_to_generator = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            async_to_generator: can_enable_plugin(
                "transform-async-to-generator",
                targets,
                bugfixes,
            ),
        }
    }
}
