use crate::{FrameworkFlags, fixer::FixKind};

mod allow_warn_deny;

pub use allow_warn_deny::AllowWarnDeny;

/// Subset of options used directly by the linter.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct LintOptions {
    pub fix: FixKind,
    pub framework_hints: FrameworkFlags,
    pub report_unused_directive: Option<AllowWarnDeny>,
    /// Create "ignore this section / line" fixes for all diagnostics.
    pub with_ignore_fixes: bool,
    /// Use enabled test plugins as framework hints instead of inspecting the file.
    /// Intended for rule tests that do not provide framework imports.
    pub use_plugin_framework_hints: bool,
}
