use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct Config;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Reports invalid React Compiler configuration options.
    ///
    upstream = "config",
    ///
    /// ::: warning
    /// This rule is currently inactive: oxlint always runs the React Compiler
    /// with a fixed, valid configuration, so configuration errors cannot
    /// occur yet. It will activate once React Compiler options become
    /// configurable in oxlint.
    /// :::
    ///
    /// ### Why is this bad?
    ///
    /// An invalid compiler configuration prevents the React Compiler from
    /// analyzing and optimizing components at all.
    Config,
    react,
    correctness,
    version = "next",
    short_description = "Validates the React Compiler configuration options.",
);

impl Rule for Config {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Config);
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

    let fail = vec![];

    Tester::new(Config::NAME, Config::PLUGIN, pass, fail).test_and_snapshot();
}
