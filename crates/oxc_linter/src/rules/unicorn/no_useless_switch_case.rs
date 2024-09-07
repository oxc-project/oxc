use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::is_empty_stmt, AstNode};

fn no_useless_switch_case_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Useless case in switch statement.")
        .with_help("Consider removing this case or removing the `default` case.")
        .with_label(span)
}

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
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// switch (foo) {
    /// 	case 1:
    /// 	default:
    /// 		handleDefaultCase();
    /// 		break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// switch (foo) {
    ///	case 1:
    ///	case 2:
    ///		handleCase1And2();
    ///		break;
    /// }
    /// ```
    NoUselessSwitchCase,
    pedantic,
    pending
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
        if std::ptr::from_ref(default_case) != std::ptr::from_ref(cases.last().unwrap()) {
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
            ctx.diagnostic(no_useless_switch_case_diagnostic(case.span));
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

    Tester::new(NoUselessSwitchCase::NAME, pass, fail).test_and_snapshot();
}
