use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2022Options {
    #[serde(skip)]
    pub class_static_block: bool,
}

impl ES2022Options {
    pub fn with_class_static_block(&mut self, enable: bool) -> &mut Self {
        self.class_static_block = enable;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            class_static_block: can_enable_plugin(
                "transform-class-static-block",
                targets,
                bugfixes,
            ),
        }
    }
}
