mod allow_warn_deny;
mod filter;

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};

use crate::{fixer::FixKind, FrameworkFlags};

/// Subset of options used directly by the linter.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq))]
pub struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
}
