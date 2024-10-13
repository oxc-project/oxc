use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn switch_case_braces_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        " Empty switch case shouldn't have braces and not-empty case should have braces around it.",
    )
    .with_help("There is less visual clutter for empty cases and proper scope for non-empty cases.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SwitchCaseBraces;

declare_oxc_lint!(
    /// ### What it does
    /// Require empty switch cases to not have braces. Non-empty braces are required to have braces around them.
    ///
    /// ### Why is this bad?
    /// There is less visual clutter for empty cases and proper scope for non-empty cases.
    ///
    /// ### Example
    /// ```javascript
    /// switch (num) {
    ///     case 1: {
    ///
    ///     }
    ///     case 2:
    ///         console.log('Case 2');
    ///         break;
    /// }
    /// ```
    SwitchCaseBraces,
    style,
    fix
);

impl Rule for SwitchCaseBraces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else {
            return;
        };

        if switch.cases.is_empty() {
            return;
        }

        for case in &switch.cases {
            for case_consequent in &case.consequent {
                match case_consequent {
                    Statement::BlockStatement(case_block) => {
                        if case_block.body.is_empty() {
                            ctx.diagnostic_with_fix(
                                switch_case_braces_diagnostic(case_block.span),
                                |fixer| fixer.delete_range(case_block.span),
                            );
                        }
                    }
                    Statement::EmptyStatement(_) => {}
                    _ => {
                        let Some(first_statement) = &case.consequent.first() else {
                            return;
                        };
                        let Some(last_statement) = &case.consequent.last() else {
                            return;
                        };

                        let case_body_span =
                            Span::new(first_statement.span().start, last_statement.span().end);

                        ctx.diagnostic_with_fix(
                            switch_case_braces_diagnostic(case_body_span),
                            |fixer| {
                                let modified_code = {
                                    let mut formatter = fixer.codegen();

                                    if let Some(case_test) = &case.test {
                                        formatter.print_str("case ");
                                        formatter.print_expression(case_test);
                                    } else {
                                        formatter.print_str("default");
                                    }

                                    formatter.print_ascii_byte(b':');
                                    formatter.print_ascii_byte(b' ');
                                    formatter.print_ascii_byte(b'{');

                                    let source_text = ctx.source_text();
                                    for x in &case.consequent {
                                        formatter.print_str(x.span().source_text(source_text));
                                    }

                                    formatter.print_ascii_byte(b'}');

                                    formatter.into_source_text()
                                };

                                fixer.replace(case.span, modified_code)
                            },
                        );

                        // After first incorrect consequent we have to break to not repeat the work
                        break;
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "switch(something) { case 1: case 2: {console.log('something'); break;}}",
        "switch(foo){ case 1: { break; } }",
        "switch(foo){ case 1: { ; /* <-- not empty */} }",
        "switch(foo){ case 1: { {} /* <-- not empty */} }",
        "switch(foo){ case 1: { break; } }",
        "switch(foo){ default: { doSomething(); } }",
    ];

    let fail = vec![
        "switch(s){case'':/]/}",
        "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
        "switch(something) { case 1: case 2: console.log('something'); break;}",
        "switch(foo) { case 1: {} case 2: {} default: { doSomething(); } }",
        "switch(foo) { case 1: { /* fallthrough */ } default: {}/* fallthrough */ case 3: { doSomething(); break; } }",
        "switch(foo) { default: doSomething(); }",
        "switch(foo) { case 1: { doSomething(); } break; /* <-- This should be between braces */ }",
        "switch(foo) { default: label: {} }",
        "switch(something) { case 1: case 2: { console.log('something'); break; } case 3: console.log('something else'); }",
    ];

    let fix = vec![
        (
            "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
            "switch(something) { case 1:  case 2: {console.log('something'); break;}}",
            None,
        ),
        (
            "switch(something) { case 1: {} case 2: console.log('something'); break;}",
            "switch(something) { case 1:  case 2: {console.log('something');break;}}",
            None,
        ),
        (
            "switch(foo) { default: doSomething(); }",
            "switch(foo) { default: {doSomething();} }",
            None,
        ),
        ("switch(s){case'':/]/}", "switch(s){case '': {/]/}}", None),
    ];

    Tester::new(SwitchCaseBraces::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
