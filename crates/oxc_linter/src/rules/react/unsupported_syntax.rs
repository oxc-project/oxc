use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct UnsupportedSyntax;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Warns on syntax that React Compiler does not plan to support, such as
    /// `eval`; components and hooks using it are skipped, not optimized.
    ///
    upstream = "unsupported-syntax",
    ///
    /// ### Why is this bad?
    ///
    /// Constructs like `eval` make data flow unanalyzable, so the component
    /// permanently opts out of compiler optimization.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   eval('props.x = true');
    ///   return <div />;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component(props) {
    ///   return <div>{props.x}</div>;
    /// }
    /// ```
    UnsupportedSyntax,
    react,
    restriction,
    version = "1.79.0",
    short_description = "Warns on syntax that the React Compiler does not plan to support, such as `eval`.",
);

impl Rule for UnsupportedSyntax {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::UnsupportedSyntax);
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
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
function Component(props) {
  eval('props.x = true');
  return <div />;
}
",
    ];

    Tester::new(UnsupportedSyntax::NAME, UnsupportedSyntax::PLUGIN, pass, fail).test_and_snapshot();
}
