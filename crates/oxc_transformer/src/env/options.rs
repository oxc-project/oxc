use oxc_diagnostics::Error;
use serde::Deserialize;

use super::targets::{query::Targets, Versions};

fn default_as_true() -> bool {
    true
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvOptions {
    #[serde(default)]
    targets: Targets,

    #[serde(default = "default_as_true")]
    pub bugfixes: bool,
}

impl EnvOptions {
    /// # Errors
    ///
    pub fn get_targets(&self) -> Result<Versions, Error> {
        self.targets.clone().get_targets()
    }
}
