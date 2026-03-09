use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct Refs;

declare_oxc_lint!(
    /// ### What it does
    /// Validates correct usage of refs (no access during render).
    Refs,
    react_compiler,
    correctness,
);

impl Rule for Refs {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(ctx, &super::react_compiler_rule::ReactCompilerConfig::default());
        super::cache::report_for_category(ctx, ErrorCategory::Refs);
    }
}

#[test]
fn test() {}
