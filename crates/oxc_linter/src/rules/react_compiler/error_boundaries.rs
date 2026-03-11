use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct ErrorBoundaries;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against try/catch in place of error boundaries.
    ErrorBoundaries,
    react_compiler,
    correctness,
);

impl Rule for ErrorBoundaries {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::ErrorBoundaries);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: JSX outside try/catch is fine
        r#"
        function Component(props) {
          return <Child value={props.value} />;
        }
        "#,
        // Cross-category: conditional hook triggers Hooks, not ErrorBoundaries
        r#"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        "#,
    ];
    let fail = vec![
        // JSX in try blocks
        r#"
        function Component(props) {
          let el;
          try {
            el = <Child />;
          } catch {
            return null;
          }
          return el;
        }
        "#,
    ];
    Tester::new(ErrorBoundaries::NAME, ErrorBoundaries::PLUGIN, pass, fail).test_and_snapshot();
}
