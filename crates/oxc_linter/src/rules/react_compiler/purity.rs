use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct Purity;

declare_oxc_lint!(
    /// ### What it does
    /// Validates pure functions.
    Purity,
    react_compiler,
    correctness,
);

impl Rule for Purity {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Purity);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: non-impure function calls are fine
        r"
        function Component(props) {
          const x = Math.max(props.a, props.b);
          return <div>{x}</div>;
        }
        ",
        // Cross-category: conditional hook triggers Hooks, not Purity
        r"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        ",
    ];
    let fail = vec![
        // Known impure function calls
        r"
        function Component() {
          const date = Date.now();
          const now = performance.now();
          const rand = Math.random();
          return <Foo date={date} now={now} rand={rand} />;
        }
        ",
    ];
    Tester::new(Purity::NAME, Purity::PLUGIN, pass, fail).test_and_snapshot();
}
