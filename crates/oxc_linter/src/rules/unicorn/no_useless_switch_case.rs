use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_empty_stmt, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(no-useless-switch-case): Useless case in switch statement.")]
#[diagnostic(
    severity(warning),
    help("Consider removing this case or removing the `default` case.")
)]
struct NoUselessSwitchCaseDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUselessSwitchCase;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows useless default cases in switch statements.
    ///
    /// ### Why is this bad?
    ///
    /// An empty case before the last default case is useless.
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// switch (foo) {
    /// 	case 1:
    /// 	default:
    /// 		handleDefaultCase();
    /// 		break;
    /// }
    /// // good:
    /// switch (foo) {
    ///	case 1:
    ///	case 2:
    ///		handleCase1And2();
    ///		break;
    /// }
    /// ```
    NoUselessSwitchCase,
    pedantic
);

impl Rule for NoUselessSwitchCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch_statement) = node.kind() else {
            return;
        };

        let cases = &switch_statement.cases;

        let default_cases = cases.iter().filter(|v| v.test.is_none()).collect::<Vec<_>>();

        if default_cases.len() != 1 {
            return;
        }

        let default_case = default_cases[0];

        // Check if the `default` case is the last case
        if default_case as *const _ != cases.last().unwrap() as *const _ {
            return;
        }

        let mut useless_cases = vec![];

        for case in cases.iter().rev().skip(1) {
            if case.consequent.iter().all(|v| is_empty_stmt(v)) {
                useless_cases.push(case);
            } else {
                break;
            }
        }

        if useless_cases.is_empty() {
            return;
        }

        for case in useless_cases {
            ctx.diagnostic(NoUselessSwitchCaseDiagnostic(case.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"
        switch (foo) {
            case a:
            case b:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
                handleCaseA();
                break;
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
                handleCaseA();
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
                break;
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
                handleCaseA();
                // Fallthrough
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
            default:
                handleDefaultCase();
                break;
            case b:
                handleCaseB();
                break;
        }
        ",
        r"
        switch (1) {
                // This is not useless
                case 1:
                default:
                        console.log('1')
                case 1:
                        console.log('2')
        }
        ",
        r"
        switch (1) {
            default:
                handleDefaultCase1();
                break;
            case 1:
            default:
                handleDefaultCase2();
                break;
        }
        ",
    ];

    let fail = vec![
        r"
        switch (foo) {
            case a:
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a: {
            }
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a: {
                ;;
                {
                    ;;
                    {
                        ;;
                    }
                }
            }
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
            case (( b ))         :
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
            case b:
                handleCaseAB();
                break;
            case d:
            case d:
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
            case b:
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            // eslint-disable-next-line
            case a:
            case b:
            default:
                handleDefaultCase();
                break;
        }
        ",
        r"
        switch (foo) {
            case a:
            // eslint-disable-next-line
            case b:
            default:
                handleDefaultCase();
                break;
        }
        ",
    ];

    Tester::new_without_config(NoUselessSwitchCase::NAME, pass, fail).test_and_snapshot();
}
