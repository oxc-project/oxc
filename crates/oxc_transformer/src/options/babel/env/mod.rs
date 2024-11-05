mod data;
mod targets;

use serde::Deserialize;

pub use self::{
    data::{bugfix_features, features},
    targets::BabelTargets,
};

use crate::options::EngineTargets;

fn default_as_true() -> bool {
    true
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct BabelEnvOptions {
    #[serde(default)]
    pub targets: EngineTargets,

    #[serde(default = "default_as_true")]
    pub bugfixes: bool,

    #[deprecated = "Not Implemented"]
    pub spec: bool,

    #[deprecated = "Not Implemented"]
    pub loose: bool,

    #[deprecated = "Not Implemented"]
    pub modules: Option<serde_json::Value>,

    #[deprecated = "Not Implemented"]
    pub debug: bool,

    #[deprecated = "Not Implemented"]
    pub include: Option<serde_json::Value>,

    #[deprecated = "Not Implemented"]
    pub exclude: Option<serde_json::Value>,

    #[deprecated = "Not Implemented"]
    pub use_built_ins: Option<serde_json::Value>,

    #[deprecated = "Not Implemented"]
    pub corejs: Option<serde_json::Value>,

    #[deprecated = "Not Implemented"]
    pub force_all_transforms: bool,

    #[deprecated = "Not Implemented"]
    pub config_path: Option<String>,

    #[deprecated = "Not Implemented"]
    pub ignore_browserslist_config: bool,

    #[deprecated = "Not Implemented"]
    pub shipped_proposals: bool,
}

impl BabelEnvOptions {
    pub fn can_enable_plugin(&self, plugin_name: &str) -> bool {
        let versions = if self.bugfixes {
            bugfix_features().get(plugin_name).unwrap_or_else(|| &features()[plugin_name])
        } else {
            &features()[plugin_name]
        };
        self.targets.should_enable(versions)
    }
}
