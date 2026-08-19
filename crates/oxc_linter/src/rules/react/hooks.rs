use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Hooks;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Runs the React Compiler's Rules of Hooks validation: hooks must be
    /// called unconditionally, in a consistent order, at the top level of a
    /// component or hook, and not be used as first-class values.
    ///
    unlinked_upstream = "hooks",
    ///
    /// This rule overlaps with `react/rules-of-hooks`; upstream ships it
    /// disabled for that reason.
    ///
    /// ### Why is this bad?
    ///
    /// React tracks hook state by call order. A hook that is called
    /// conditionally or in a different order between renders breaks the
    /// association between each hook call and its state, corrupting
    /// component state.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   if (props.cond) {
    ///     useState(0); // hooks may not be called conditionally
    ///   }
    ///   return <div>{props.text}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   const [state, setState] = useState(0);
    ///   return <div onClick={() => setState(state + 1)}>{props.text}</div>;
    /// }
    /// ```
    Hooks,
    react,
    suspicious,
    version = "1.79.0",
    short_description = "Validates the Rules of Hooks with the React Compiler's analysis.",
);

impl Rule for Hooks {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Hooks);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ---- InvalidHooksRule-test.ts ----
        // Basic example
        "
function Component() {
  useHook();
  return <div>Hello world</div>;
}
",
    ];

    let fail = vec![
        // ---- InvalidHooksRule-test.ts ----
        // Simple violation
        "
function useConditional() {
  if (cond) {
    useConditionalHook();
  }
}
",
    ];

    Tester::new(Hooks::NAME, Hooks::PLUGIN, pass, fail).test_and_snapshot();
}
