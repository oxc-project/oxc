use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn default_case_last_diagnostic(span: Span) -> OxcDiagnostic {
    let default_span = Span::sized(span.start, 7);

    OxcDiagnostic::warn("Enforce default clauses in switch statements to be last")
        .with_label(default_span.label("Default clause should be the last clause."))
}

#[derive(Debug, Default, Clone)]
pub struct DefaultCaseLast;

declare_oxc_lint!(
    /// ### What it does
    /// Requires the `default` clause in `switch` statements to be the last one.
    ///
    /// ### Why is this bad?
    /// By convention and for readability, the `default` clause should be the last one in a `switch`.
    /// While it is legal to place it before or between `case` clauses, doing so is confusing and may
    /// lead to unexpected "fall-through" behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /* default-case-last: "error" */
    ///
    /// switch (foo) {
    ///   default:
    ///     bar();
    ///     break;
    ///   case "a":
    ///     baz();
    ///     break;
    /// }
    ///
    /// switch (foo) {
    ///   case 1:
    ///     bar();
    ///     break;
    ///   default:
    ///     baz();
    ///     break;
    ///   case 2:
    ///     qux();
    ///     break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /* default-case-last: "error" */
    ///
    /// switch (foo) {
    ///   case 1:
    ///     bar();
    ///     break;
    ///   case 2:
    ///     qux();
    ///     break;
    ///   default:
    ///     baz();
    ///     break;
    /// }
    ///
    /// switch (foo) {
    ///   case "x":
    ///     bar();
    ///     break;
    ///   case "y":
    ///   default:
    ///     baz();
    ///     break;
    /// }
    /// ```
    DefaultCaseLast,
    eslint,
    style
);

impl Rule for DefaultCaseLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else {
            return;
        };

        let cases = &switch.cases;
        let cases_without_last = &cases[..cases.len().saturating_sub(1)];
        if let Some(default_clause) = cases_without_last.iter().find(|c| c.test.is_none()) {
            ctx.diagnostic(default_case_last_diagnostic(default_clause.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"switch (foo) {}",
        r"switch (foo) { case 1: bar(); break; }",
        r"switch (foo) { case 1: break; }",
        r"switch (foo) { case 1: }",
        r"switch (foo) { case 1: bar(); break; case 2: baz(); break; }",
        r"switch (foo) { case 1: break; case 2: break; }",
        r"switch (foo) { case 1: case 2: break; }",
        r"switch (foo) { case 1: case 2: }",
        r"switch (foo) { default: bar(); break; }",
        r"switch (foo) { default: bar(); }",
        r"switch (foo) { default: break; }",
        r"switch (foo) { default: }",
        r"switch (foo) { case 1: break; default: break; }",
        r"switch (foo) { case 1: break; default: }",
        r"switch (foo) { case 1: default: break; }",
        r"switch (foo) { case 1: default: }",
        r"switch (foo) { case 1: baz(); break; case 2: quux(); break; default: quuux(); break; }",
        r"switch (foo) { case 1: break; case 2: break; default: break; }",
        r"switch (foo) { case 1: break; case 2: break; default: }",
        r"switch (foo) { case 1: case 2: break; default: break; }",
        r"switch (foo) { case 1: break; case 2: default: break; }",
        r"switch (foo) { case 1: break; case 2: default: }",
        r"switch (foo) { case 1: case 2: default: }",
    ];

    let fail = vec![
        r"switch (foo) { default: bar(); break; case 1: baz(); break; }",
        r"switch (foo) { default: break; case 1: break; }",
        r"switch (foo) { default: break; case 1: }",
        r"switch (foo) { default: case 1: break; }",
        r"switch (foo) { default: case 1: }",
        r"switch (foo) { default: break; case 1: break; case 2: break; }",
        r"switch (foo) { default: case 1: break; case 2: break; }",
        r"switch (foo) { default: case 1: case 2: break; }",
        r"switch (foo) { default: case 1: case 2: }",
        r"switch (foo) { case 1: break; default: break; case 2: break; }",
        r"switch (foo) { case 1: default: break; case 2: break; }",
        r"switch (foo) { case 1: break; default: case 2: break; }",
        r"switch (foo) { case 1: default: case 2: break; }",
        r"switch (foo) { case 1: default: case 2: }",
    ];

    Tester::new(DefaultCaseLast::NAME, DefaultCaseLast::PLUGIN, pass, fail).test_and_snapshot();
}
