use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct CapitalizedCalls;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Disallows calling capitalized functions or methods directly during
    /// render instead of rendering them with JSX, since capitalized names are
    /// reserved for components.
    ///
    unlinked_upstream = "capitalized-calls",
    ///
    /// ### Why is this bad?
    ///
    /// Calling a component as a plain function hides it from React: it gets
    /// no state isolation and no hooks context of its own, and it breaks
    /// memoization.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// import Child from './Child';
    /// function Component() {
    ///   return <div>{Child()}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// import Child from './Child';
    /// function Component() {
    ///   return <div><Child /></div>;
    /// }
    /// ```
    CapitalizedCalls,
    react,
    suspicious,
    version = "1.79.0",
    short_description = "Disallow calling capitalized functions and methods instead of using JSX.",
);

impl Rule for CapitalizedCalls {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::CapitalizedCalls);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
function Component(props) {
  return <div>{props.text}</div>;
}
",
    ];

    let fail = vec![
        // ---- NoCapitalizedCallsRule-test.ts ----
        // Simple violation
        "
import Child from './Child';
function Component() {
  return <>
    {Child()}
  </>;
}
",
        // Method call violation
        "
import myModule from './MyModule';
function Component() {
  return <>
    {myModule.Child()}
  </>;
}
",
        // Multiple diagnostics within the same function are surfaced
        "
import Child1 from './Child1';
import MyModule from './MyModule';
function Component() {
  return <>
    {Child1()}
    {MyModule.Child2()}
  </>;
}",
    ];

    Tester::new(CapitalizedCalls::NAME, CapitalizedCalls::PLUGIN, pass, fail).test_and_snapshot();
}
