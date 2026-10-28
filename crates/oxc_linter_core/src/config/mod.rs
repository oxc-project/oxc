use std::path::PathBuf;

mod env;
mod globals;
mod options;
pub mod plugins;
pub mod settings;

pub use env::OxlintEnv;
pub use globals::{GlobalValue, OxlintGlobals};
pub use options::OxlintOptions;
pub use plugins::LintPlugins;
pub use settings::{OxlintSettings, ReactVersion, jsdoc::JSDocPluginSettings};

/// Resolved configuration shared by the rules linting a file.
#[derive(Debug, Default, Clone)]
pub struct LintConfig {
    pub plugins: LintPlugins,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
    /// Absolute path to the configuration file (may be `None` if there is no file).
    pub path: Option<PathBuf>,
    /// Options for the linter.
    pub options: OxlintOptions,
}
