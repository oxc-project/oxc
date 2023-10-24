use oxc_ast::{ast::Statement, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_formatter::Gen;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(switch-case-braces):  Empty switch case shouldn't have braces and not-empty case should have braces around it.")]
#[diagnostic(
    severity(warning),
    help("There is less visual clutter for empty cases and proper scope for non-empty cases.")
)]
struct SwitchCaseBracesDiagnostic(#[label] pub Span);

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
    correctness
);

impl Rule for SwitchCaseBraces {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch) = node.kind() else { return };

        if switch.cases.is_empty() {
            return;
        }

        for case in &switch.cases {
            let Some(case_test) = &case.test else { return };

            for case_body in &case.consequent {
                match case_body {
                    Statement::BlockStatement(case_block) => {
                        if case_block.body.is_empty() {
                            ctx.diagnostic_with_fix(
                                SwitchCaseBracesDiagnostic(case_block.span),
                                || Fix::new("", case_block.span),
                            );
                        }
                    }
                    Statement::EmptyStatement(_) => {}
                    _ => {
                        ctx.diagnostic_with_fix(SwitchCaseBracesDiagnostic(case.span), || {
                            let modified_code = {
                                let mut formatter = ctx.formatter();

                                formatter.print_str(b"case ");
                                case_test.gen(&mut formatter);
                                formatter.print_colon();
                                formatter.print_space();
                                formatter.print(b'{');
                                case.consequent.iter().for_each(|x| x.gen(&mut formatter));
                                formatter.print(b'}');

                                formatter.into_code()
                            };

                            Fix::new(modified_code, case.span)
                        });
                        ctx.diagnostic(SwitchCaseBracesDiagnostic(switch.span));
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec!["switch(something) { case 1: case 2: {console.log('something'); break;}}"];

    let fail = vec![
        "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
        "switch(something) { case 1: case 2: console.log('something'); break;}",
    ];

    let fix = vec![
        (
            "switch(something) { case 1: {} case 2: {console.log('something'); break;}}",
            "switch(something) { case 1:  case 2: {console.log('something'); break;}}",
            None,
        ),
        (
            "switch(something) { case 1: {} case 2: console.log('something'); break;}",
            "switch(something) { case 1:  case 2: {console.log(\"something\");\nbreak;\n}}",
            None,
        ),
    ];

    Tester::new_without_config(SwitchCaseBraces::NAME, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
