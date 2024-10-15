use serde::Deserialize;
use serde_json::Value;

use oxc_diagnostics::Error;

use super::targets::{query::Targets, Versions};

fn default_as_true() -> bool {
    true
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct EnvOptions {
    #[serde(default)]
    pub targets: Targets,

    #[serde(default = "default_as_true")]
    pub bugfixes: bool,

    /// Unused.
    pub spec: bool,

    /// Unused.
    pub loose: bool,

    /// Unused.
    pub modules: Option<Value>,

    /// Unused.
    pub debug: bool,

    /// Unused.
    pub include: Option<Value>,

    /// Unused.
    pub exclude: Option<Value>,

    /// Unused.
    pub use_built_ins: Option<Value>,

    /// Unused.
    pub corejs: Option<Value>,

    /// Unused.
    pub force_all_transforms: bool,

    /// Unused.
    pub config_path: Option<String>,

    /// Unused.
    pub ignore_browserslist_config: bool,

    /// Unused.
    pub shipped_proposals: bool,
}

impl EnvOptions {
    /// # Errors
    ///
    pub fn get_targets(&self) -> Result<Versions, Error> {
        self.targets.clone().get_targets()
    }
}
