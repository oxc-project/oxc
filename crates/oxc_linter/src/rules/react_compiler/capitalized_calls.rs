use oxc_macros::declare_oxc_lint;
use oxc_react_compiler::compiler_error::ErrorCategory;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Default, Clone)]
pub struct CapitalizedCalls;

declare_oxc_lint!(
    /// ### What it does
    /// Validates against calling component-like functions directly.
    CapitalizedCalls,
    react_compiler,
    restriction,
);

impl Rule for CapitalizedCalls {
    fn run_once(&self, ctx: &LintContext<'_>) {
        super::cache::ensure_compiled(
            ctx,
            &super::react_compiler_rule::ReactCompilerConfig::default(),
        );
        super::cache::report_for_category(ctx, ErrorCategory::CapitalizedCalls);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    let pass = vec![
        // Valid: capitalized component invoked via JSX, not called directly
        r"
        import Child from './Child';
        function Component() {
          return <Child />;
        }
        ",
        // Cross-category: conditional hook triggers Hooks, not CapitalizedCalls
        r"
        function useConditional() {
          if (cond) {
            useConditionalHook();
          }
        }
        ",
    ];
    let fail = vec![
        // Direct capitalized call
        r"
        import Child from './Child';
        function Component() {
          return <>
            {Child()}
          </>;
        }
        ",
        // Method call with capitalized name
        r"
        import myModule from './MyModule';
        function Component() {
          return <>
            {myModule.Child()}
          </>;
        }
        ",
        // Multiple capitalized calls
        r"
        import Child1 from './Child1';
        import MyModule from './MyModule';
        function Component() {
          return <>
            {Child1()}
            {MyModule.Child2()}
          </>;
        }
        ",
    ];
    Tester::new(CapitalizedCalls::NAME, CapitalizedCalls::PLUGIN, pass, fail).test_and_snapshot();
}
