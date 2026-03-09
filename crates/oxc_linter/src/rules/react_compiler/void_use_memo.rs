use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct VoidUseMemo;

declare_oxc_lint!(
    /// ### What it does
    /// Validates that useMemo always returns a value.
    VoidUseMemo,
    react_compiler,
    nursery,
);

impl Rule for VoidUseMemo {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &super::react_compiler_rule::ReactCompilerConfig::default());
        super::cache::report_for_category(ctx, ErrorCategory::VoidUseMemo);
    }
}

#[test]
fn test() {}
