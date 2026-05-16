use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct MemoDependencies;

declare_oxc_lint!(
    /// ### What it does
    /// Validates exhaustive useMemo/useCallback dependencies.
    MemoDependencies,
    react_compiler,
    restriction,
    version = "next",
);

impl Rule for MemoDependencies {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(
            ctx,
            ErrorCategory::MemoDependencies,
            <Self as crate::rule::RuleMeta>::NAME,
        );
    }
}

#[test]
fn test() {}
