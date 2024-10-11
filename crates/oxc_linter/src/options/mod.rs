mod allow_warn_deny;
mod filter;
mod plugins;

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};
pub use plugins::LintPlugins;

use crate::{fixer::FixKind, FrameworkFlags};

/// Subset of options used directly by the linter.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub plugins: LintPlugins,
}
