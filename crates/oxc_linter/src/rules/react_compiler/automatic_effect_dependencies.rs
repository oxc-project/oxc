use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct AutomaticEffectDependencies;

declare_oxc_lint!(
    /// ### What it does
    /// Verifies that automatic effect dependencies are compiled if opted-in
    AutomaticEffectDependencies,
    react_compiler,
    nursery,
    version = "next",
);

impl Rule for AutomaticEffectDependencies {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::AutomaticEffectDependencies);
    }
}

#[test]
fn test() {}
