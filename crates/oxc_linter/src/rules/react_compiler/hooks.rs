use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct Hooks;

declare_oxc_lint!(
    /// ### What it does
    /// Validates the rules of hooks.
    Hooks,
    react_compiler,
    restriction,
);

impl Rule for Hooks {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::Hooks);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Basic hook usage
        r#"
        function Component() {
          useHook();
          return <div>Hello world</div>;
        }
        "#,
        // Invariant: defined after use (not surfaced)
        r#"
        function Component(props) {
          let y = function () {
            m(x);
          };
          let x = { a };
          m(x);
          return y;
        }
        "#,
        // Classes don't throw
        r#"
        class Foo {
          #bar() {}
        }
        "#,
        // Cross-category: impure calls trigger Purity, not Hooks
        r#"
        function Component() {
          const date = Date.now();
          return <Foo date={date} />;
        }
        "#,
    ];
    let fail = vec![
        // Simple conditional hook violation
        r#"
        function Component() {
          if (cond) {
            useConditionalHook();
          }
          return <div />;
        }
        "#,
        // Multiple conditional hooks in same function
        r#"
        function Component() {
          cond ?? useConditionalHook();
          props.cond && useConditionalHook();
          return <div>Hello world</div>;
        }
        "#,
    ];
    Tester::new(Hooks::NAME, Hooks::PLUGIN, pass, fail).test_and_snapshot();
}
