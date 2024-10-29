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

    #[deprecated = "Not Implemented"]
    pub spec: bool,

    #[deprecated = "Not Implemented"]
    pub loose: bool,

    #[deprecated = "Not Implemented"]
    pub modules: Option<Value>,

    #[deprecated = "Not Implemented"]
    pub debug: bool,

    #[deprecated = "Not Implemented"]
    pub include: Option<Value>,

    #[deprecated = "Not Implemented"]
    pub exclude: Option<Value>,

    #[deprecated = "Not Implemented"]
    pub use_built_ins: Option<Value>,

    #[deprecated = "Not Implemented"]
    pub corejs: Option<Value>,

    #[deprecated = "Not Implemented"]
    pub force_all_transforms: bool,

    #[deprecated = "Not Implemented"]
    pub config_path: Option<String>,

    #[deprecated = "Not Implemented"]
    pub ignore_browserslist_config: bool,

    #[deprecated = "Not Implemented"]
    pub shipped_proposals: bool,
}

impl EnvOptions {
    /// # Errors
    ///
    pub fn get_targets(&self) -> Result<Versions, Error> {
        self.targets.clone().get_targets()
    }
}
