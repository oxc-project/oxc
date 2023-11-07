use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(default-case-last): Enforce default clauses in switch statements to be last")]
#[diagnostic(severity(warning))]
struct DefaultCaseLastDiagnostic(#[label("Default clause should be the last clause.")] pub Span);

#[derive(Debug, Default, Clone)]
pub struct DefaultCaseLast;

declare_oxc_lint!(
    /// ### What it does
    /// Enforce default clauses in switch statements to be last
    ///
    /// ### Why is this bad?
    /// A switch statement can optionally have a default clause.
    /// If present, it’s usually the last clause, but it doesn’t need to be. It is also allowed to put the default clause before all case clauses, or anywhere between. The behavior is mostly the same as if it was the last clause. The default block will be still executed only if there is no match in the case clauses (including those defined after the default), but there is also the ability to “fall through” from the default clause to the following clause in the list. However, such flow is not common and it would be confusing to the readers.
    /// Even if there is no “fall through” logic, it’s still unexpected to see the default clause before or between the case clauses. By convention, it is expected to be the last clause.
    /// If a switch statement should have a default clause, it’s considered a best practice to define it as the last clause.
    ///
    /// ### Example
    /// ```javascript
    /// switch (foo) {
    ///     default:
    ///         bar();
    ///         break;
    ///     case "a":
    ///         baz();
    ///         break;
    /// }
    ///
    /// switch (foo) {
    ///     case 1:
    ///         bar();
    ///         break;
    ///     default:
    ///         baz();
    ///         break;
    ///     case 2:
    ///         qux();
    ///         break;
    /// }
    /// ```
    DefaultCaseLast,
    style
);

impl Rule for DefaultCaseLast {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else { return };
        let cases = &switch.cases;
        let index_of_default = cases.iter().position(|c| c.test.is_none());

        if let Some(index) = index_of_default {
            if index != cases.len() - 1 {
                let default_clause = &cases[index];
                ctx.diagnostic(DefaultCaseLastDiagnostic(Span::new(
                    default_clause.span.start,
                    default_clause.span.start + 7,
                )));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"switch (foo) {}"#,
        r#"switch (foo) { case 1: bar(); break; }"#,
        r#"switch (foo) { case 1: break; }"#,
        r#"switch (foo) { case 1: }"#,
        r#"switch (foo) { case 1: bar(); break; case 2: baz(); break; }"#,
        r#"switch (foo) { case 1: break; case 2: break; }"#,
        r#"switch (foo) { case 1: case 2: break; }"#,
        r#"switch (foo) { case 1: case 2: }"#,
        r#"switch (foo) { default: bar(); break; }"#,
        r#"switch (foo) { default: bar(); }"#,
        r#"switch (foo) { default: break; }"#,
        r#"switch (foo) { default: }"#,
        r#"switch (foo) { case 1: break; default: break; }"#,
        r#"switch (foo) { case 1: break; default: }"#,
        r#"switch (foo) { case 1: default: break; }"#,
        r#"switch (foo) { case 1: default: }"#,
        r#"switch (foo) { case 1: baz(); break; case 2: quux(); break; default: quuux(); break; }"#,
        r#"switch (foo) { case 1: break; case 2: break; default: break; }"#,
        r#"switch (foo) { case 1: break; case 2: break; default: }"#,
        r#"switch (foo) { case 1: case 2: break; default: break; }"#,
        r#"switch (foo) { case 1: break; case 2: default: break; }"#,
        r#"switch (foo) { case 1: break; case 2: default: }"#,
        r#"switch (foo) { case 1: case 2: default: }"#,
    ];

    let fail = vec![
        r#"switch (foo) { default: bar(); break; case 1: baz(); break; }"#,
        r#"switch (foo) { default: break; case 1: break; }"#,
        r#"switch (foo) { default: break; case 1: }"#,
        r#"switch (foo) { default: case 1: break; }"#,
        r#"switch (foo) { default: case 1: }"#,
        r#"switch (foo) { default: break; case 1: break; case 2: break; }"#,
        r#"switch (foo) { default: case 1: break; case 2: break; }"#,
        r#"switch (foo) { default: case 1: case 2: break; }"#,
        r#"switch (foo) { default: case 1: case 2: }"#,
        r#"switch (foo) { case 1: break; default: break; case 2: break; }"#,
        r#"switch (foo) { case 1: default: break; case 2: break; }"#,
        r#"switch (foo) { case 1: break; default: case 2: break; }"#,
        r#"switch (foo) { case 1: default: case 2: break; }"#,
        r#"switch (foo) { case 1: default: case 2: }"#,
    ];

    Tester::new_without_config(DefaultCaseLast::NAME, pass, fail).test_and_snapshot();
}
