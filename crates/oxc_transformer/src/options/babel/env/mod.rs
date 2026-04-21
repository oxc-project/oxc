use serde::Deserialize;

use crate::{Module, options::EngineTargets};

fn default_as_true() -> bool {
    true
}

#[derive(Default, Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
/// Babel `preset-env` style configuration.
pub struct BabelEnvOptions {
    /// Target engines used to determine which transforms should be enabled.
    #[serde(default)]
    pub targets: EngineTargets,

    /// Enable Babel's bugfix transforms.
    #[deprecated = "Not Implemented"]
    #[serde(default = "default_as_true")]
    pub bugfixes: bool,

    /// Enable spec-compliant transforms over looser output.
    #[deprecated = "Not Implemented"]
    pub spec: bool,

    /// Enable loose mode for eligible transforms.
    #[deprecated = "Not Implemented"]
    pub loose: bool,

    /// Module transform mode.
    pub modules: Module,

    /// Print debugging information while selecting transforms.
    #[deprecated = "Not Implemented"]
    pub debug: bool,

    /// Force-enable specific transforms.
    #[deprecated = "Not Implemented"]
    pub include: Option<serde_json::Value>,

    /// Force-disable specific transforms.
    #[deprecated = "Not Implemented"]
    pub exclude: Option<serde_json::Value>,

    /// Polyfill injection mode.
    #[deprecated = "Not Implemented"]
    pub use_built_ins: Option<serde_json::Value>,

    /// `core-js` version/options for polyfill injection.
    #[deprecated = "Not Implemented"]
    pub corejs: Option<serde_json::Value>,

    /// Force all eligible transforms regardless of targets.
    #[deprecated = "Not Implemented"]
    pub force_all_transforms: bool,

    /// Explicit path to config file.
    #[deprecated = "Not Implemented"]
    pub config_path: Option<String>,

    /// Skip loading `.browserslistrc`.
    #[deprecated = "Not Implemented"]
    pub ignore_browserslist_config: bool,

    /// Enable proposal transforms that are shipped in browsers.
    #[deprecated = "Not Implemented"]
    pub shipped_proposals: bool,
}

#[derive(Default, Debug, Clone, Deserialize)]
/// Raw Babel `modules` option values.
pub enum BabelModule {
    /// Let Babel infer module behavior.
    #[default]
    #[serde(rename = "auto")]
    Auto,
    /// Transform modules to AMD.
    #[serde(rename = "amd")]
    Amd,
    /// Transform modules to UMD.
    #[serde(rename = "umd")]
    Umd,
    /// Transform modules to SystemJS.
    #[serde(rename = "systemjs")]
    Systemjs,
    /// Transform modules to CommonJS.
    #[serde(rename = "commonjs", alias = "cjs")]
    Commonjs,
    /// Babel accepts booleans for module options in some configurations.
    #[serde(untagged)]
    Boolean(bool),
}
