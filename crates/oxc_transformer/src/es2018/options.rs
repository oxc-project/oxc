use serde::Deserialize;

use crate::env::{can_enable_plugin, Versions};

use super::ObjectRestSpreadOptions;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct ES2018Options {
    #[serde(skip)]
    pub object_rest_spread: Option<ObjectRestSpreadOptions>,
}

impl ES2018Options {
    pub fn with_object_rest_spread(
        &mut self,
        option: Option<ObjectRestSpreadOptions>,
    ) -> &mut Self {
        self.object_rest_spread = option;
        self
    }

    #[must_use]
    pub fn from_targets_and_bugfixes(targets: Option<&Versions>, bugfixes: bool) -> Self {
        Self {
            object_rest_spread: can_enable_plugin(
                "transform-object-rest-spread",
                targets,
                bugfixes,
            )
            .then(Default::default),
        }
    }
}
