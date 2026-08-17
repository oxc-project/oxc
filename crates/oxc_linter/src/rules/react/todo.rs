use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Todo;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Reports code that React Compiler cannot yet analyze because it uses
    /// features the compiler has not implemented. These are skipped
    /// optimizations (bail-outs), not rule violations.
    ///
    upstream = "todo",
    ///
    /// ### Why is this bad?
    ///
    /// The affected component or hook is left unoptimized. Enable this rule
    /// only when you want visibility into what the compiler skips; upstream
    /// ships it as an off-by-default hint.
    Todo,
    react,
    restriction,
    version = "next",
    short_description = "Reports code using features the React Compiler has not implemented yet.",
);

impl Rule for Todo {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Todo);
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
        "
function Component() {
  const fbt = 'span';
  return <div>{fbt}</div>;
}
",
    ];

    let fail = vec![
        // User-facing unsupported-syntax diagnostics should not expose
        // internal compiler implementation names.
        "import { useEffect } from 'react';
            function Component() {
                useEffect(() => {
                    try {
                        doSomething();
                    } finally {
                        cleanup();
                    }
                }, []);
                return <div />;
            }",
    ];

    Tester::new(Todo::NAME, Todo::PLUGIN, pass, fail).test_and_snapshot();
}
