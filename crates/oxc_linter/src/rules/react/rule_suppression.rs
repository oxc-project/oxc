use oxc_react_compiler::ErrorCategory;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    utils::{run_react_compiler_rule, should_run_react_compiler},
};

#[derive(Debug, Default, Clone)]
pub struct RuleSuppression;

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Reports ESLint/Oxlint suppressions of React rules (for example
    /// `eslint-disable-next-line react-hooks/exhaustive-deps`) inside a
    /// component or hook. The React Compiler skips functions containing such
    /// suppressions, since the suppressed violation may make compilation
    /// unsafe.
    ///
    unlinked_upstream = "rule-suppression",
    ///
    /// ### Why is this bad?
    ///
    /// Suppressing a React rule hides a violation the compiler must assume is
    /// real; the whole function loses optimization until the suppression is
    /// removed and the underlying error fixed.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// function Component({ value }) {
    ///   // eslint-disable-next-line react-hooks/exhaustive-deps
    ///   const doubled = value * 2;
    ///   return <div>{doubled}</div>;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// function Component({ value }) {
    ///   const doubled = value * 2;
    ///   return <div>{doubled}</div>;
    /// }
    /// ```
    RuleSuppression,
    react,
    restriction,
    version = "1.79.0",
    short_description = "Reports suppressions of React rules, which make the React Compiler skip the affected function.",
);

impl Rule for RuleSuppression {
    fn run_once(&self, ctx: &LintContext) {
        run_react_compiler_rule(ctx, ErrorCategory::Suppression);
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
function Component({value}) {
  const doubled = value * 2;
  return <div>{doubled}</div>;
}
",
        // Suppressions of unrelated rules are ignored.
        "
function Component({value}) {
  // eslint-disable-next-line no-console
  console.log(value);
  return <div>{value}</div>;
}
",
    ];

    let fail = vec![
        // Derived from crates/oxc_react_compiler/fixtures cases.
        "
function Component({value}) {
  // eslint-disable-next-line react-hooks/exhaustive-deps
  const doubled = value * 2;
  return <div>{doubled}</div>;
}
",
    ];

    Tester::new(RuleSuppression::NAME, RuleSuppression::PLUGIN, pass, fail).test_and_snapshot();
}
