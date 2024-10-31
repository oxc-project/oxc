mod data;
mod targets;

pub use data::{bugfix_features, features};
pub use targets::{Targets, Version};

use serde::Deserialize;

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

impl EnvOptions {
    pub fn can_enable_plugin(&self, plugin_name: &str) -> bool {
        let versions = if self.bugfixes {
            bugfix_features().get(plugin_name).unwrap_or_else(|| &features()[plugin_name])
        } else {
            &features()[plugin_name]
        };
        self.targets.should_enable(versions)
    }
}
