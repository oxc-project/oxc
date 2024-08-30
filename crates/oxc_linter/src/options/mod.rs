mod allow_warn_deny;
mod filter;
mod plugins;

use crate::{fixer::FixKind, FrameworkFlags};

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};
pub use plugins::LintPlugins;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub(crate) struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub plugins: LintPlugins,
}
