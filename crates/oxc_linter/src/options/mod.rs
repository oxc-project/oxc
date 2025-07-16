use crate::{FrameworkFlags, fixer::FixKind};

mod allow_warn_deny;
mod filter;

pub use allow_warn_deny::AllowWarnDeny;
pub use filter::{InvalidFilterKind, LintFilter, LintFilterKind};

/// Subset of options used directly by the linter.
#[derive(Debug, Default, Clone, Copy)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub report_unused_directive: Option<AllowWarnDeny>,
}
