use crate::{context::LintContext, rule::Rule, AstNode};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

fn no_negated_condition_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected negated condition.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoNegatedCondition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows the use of negated conditions in `if` statements to improve readability.
    ///
    /// ### Why is this bad?
    ///
    /// Negated conditions can make code harder to read and understand, especially in complex logic.
    /// It is often clearer to use positive conditions or to refactor the code structure.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (!isReady) {
    ///   doSomething();
    /// } else {
    ///   doSomethingElse();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (isReady) {
    ///   doSomethingElse();
    /// } else {
    ///   doSomething();
    /// }
    /// ```
    NoNegatedCondition,
    style,
    pending,
);

impl Rule for NoNegatedCondition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // This rule is exactly the same as the eslint-plugin-unicorn's no-negated-condition rule.
        crate::rules::unicorn::no_negated_condition::NoNegatedCondition::default().run(node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (a) {}",
        "if (a) {} else {}",
        "if (!a) {}",
        "if (!a) {} else if (b) {}",
        "if (!a) {} else if (b) {} else {}",
        "if (a == b) {}",
        "if (a == b) {} else {}",
        "if (a != b) {}",
        "if (a != b) {} else if (b) {}",
        "if (a != b) {} else if (b) {} else {}",
        "if (a !== b) {}",
        "if (a === b) {} else {}",
        "a ? b : c",
    ];

    let fail = vec![
        "if (!a) {;} else {;}",
        "if (a != b) {;} else {;}",
        "if (a !== b) {;} else {;}",
        "!a ? b : c",
        "a != b ? c : d",
        "a !== b ? c : d",
    ];

    Tester::new(NoNegatedCondition::NAME, NoNegatedCondition::CATEGORY, pass, fail)
        .test_and_snapshot();
}
