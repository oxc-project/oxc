use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct Immutability;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against mutations of props, hook arguments, and hook return values.
    Immutability,
    react_compiler,
    correctness,
);

impl Rule for Immutability {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &super::react_compiler_rule::ReactCompilerConfig::default());
        super::cache::report_for_category(ctx, ErrorCategory::Immutability);
    }
}

#[test]
fn test() {}
