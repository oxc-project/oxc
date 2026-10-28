use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Syntax;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Reports invalid JavaScript encountered by React Compiler while
    /// analyzing a component or hook, such as reassigning a `const` binding.
    ///
    unlinked_upstream = "syntax",
    ///
    /// ### Why is this bad?
    ///
    /// The code would throw at runtime; the compiler skips the function
    /// instead of optimizing it.
    Syntax,
    react,
    restriction,
    version = "1.79.0",
    short_description = "Reports invalid JavaScript syntax encountered by the React Compiler.",
);

impl Rule for Syntax {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Syntax);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // ---- PluginTest-test.ts ----
        // Classes don't throw
        "
class Foo {
  #bar() {}
}
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
function Component() {
  const x = 0;
  x = 1;
  return <div>{x}</div>;
}
",
    ];

    Tester::new(Syntax::NAME, Syntax::PLUGIN, pass, fail).test_and_snapshot();
}
