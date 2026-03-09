use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct RuleSuppression;

declare_oxc_lint!(
    /// ### What it does
    /// Reports rule suppression issues.
    RuleSuppression,
    react_compiler,
    restriction,
);

impl Rule for RuleSuppression {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Suppression);
    }
}

#[test]
fn test() {}
