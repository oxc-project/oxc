use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct CompilerConfig;

declare_oxc_lint!(
    /// ### What it does
    /// Reports compiler configuration errors.
    CompilerConfig,
    react_compiler,
    correctness,
);

impl Rule for CompilerConfig {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Config);
    }
}

#[test]
fn test() {}
