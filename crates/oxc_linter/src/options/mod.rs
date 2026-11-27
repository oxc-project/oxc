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
    /// How to report unused disable directives, for every file.
    ///
    /// `None` lets the `reportUnusedDisableDirectives` option of the config which governs each
    /// file decide. `Some(AllowWarnDeny::Allow)` turns reporting off, and any other value
    /// overrides the configs, as `--report-unused-disable-directives` and the editor setting do.
    pub report_unused_directive: Option<AllowWarnDeny>,
    // If true, the linter will create "ignore this section / line" fixes for all diagnostics
    pub with_ignore_fixes: bool,
}
