use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct ComponentHookFactories;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against higher order functions defining nested components or hooks. Components and hooks should be defined at the module level
    ComponentHookFactories,
    react_compiler,
    correctness,
    version = "next",
);

impl Rule for ComponentHookFactories {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Factories);
    }
}

#[test]
fn test() {}
