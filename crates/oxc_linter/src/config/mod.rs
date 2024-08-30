mod env;
mod globals;
mod oxlintrc;
mod rules;
mod settings;

pub use self::{
    env::OxlintEnv,
    globals::OxlintGlobals,
    oxlintrc::Oxlintrc,
    rules::OxlintRules,
    settings::{jsdoc::JSDocPluginSettings, OxlintSettings},
};

/// Configuration used by the linter, fixer, and rules.
///
/// This is a mapping from the public [`Oxlintrc`] API to a trimmed down
/// version that is also better suited for internal use. Do not expose this
/// struct outside this crate.
#[derive(Debug, Default)]
pub(crate) struct LintConfig {
    pub(crate) settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub(crate) env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub(crate) globals: OxlintGlobals,
}
