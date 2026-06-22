use oxc_allocator::UnstableAddress;
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_empty_stmt};

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
    /// Disallows useless `default` cases in `switch` statements.
    ///
    /// ### Why is this bad?
    ///
    /// An empty case before the last `default` case is useless, as the
    /// `default` case will catch it regardless.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// switch (foo) {
    ///   case 1:
    ///   default:
    ///     handleDefaultCase();
    ///     break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// switch (foo) {
    ///   case 1:
    ///   case 2:
    ///     handleCase1And2();
    ///     break;
    /// }
    /// ```
    NoUselessSwitchCase,
    unicorn,
    pedantic,
    pending,
    version = "0.0.18",
    short_description = "Disallows useless `default` cases in `switch` statements.",
);

impl Rule for NoUselessSwitchCase {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchStatement(switch_statement) = node.kind() else {
            return;
        };

        let cases = &switch_statement.cases;

        let mut default_cases = cases.iter().filter(|case| case.test.is_none());
        let Some(default_case) = default_cases.next() else {
            return;
        };

        if default_cases.next().is_some() {
            return;
        }

        // Check if the `default` case is the last case
        if default_case.unstable_address() != cases.last().unwrap().unstable_address() {
            return;
        }

        let useless_cases = cases
            .iter()
            .rev()
            .skip(1)
            .take_while(|case| case.consequent.iter().all(|stmt| is_empty_stmt(stmt)));

        for useless_case in useless_cases {
            ctx.diagnostic(no_useless_switch_case_diagnostic(useless_case.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "
        switch (foo) {
            case a:
            case b:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            case a:
                handleCaseA();
                break;
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            case a:
                handleCaseA();
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            case a:
                break;
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            case a:
                handleCaseA();
                // Fallthrough
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
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
        "
        switch (1) {
                // This is not useless
                case 1:
                default:
                        console.log('1')
                case 1:
                        console.log('2')
        }
        ",
    ];

    let fail = vec![
        "
        switch (foo) {
            case a:
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            case a: {
            }
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
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
        "
        switch (foo) {
            case a:
            case (( b ))         :
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
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
        "
        switch (foo) {
            case a:
            case b:
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
        switch (foo) {
            // eslint-disable-next-line
            case a:
            case b:
            default:
                handleDefaultCase();
                break;
        }
        ",
        "
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

    Tester::new(NoUselessSwitchCase::NAME, NoUselessSwitchCase::PLUGIN, pass, fail)
        .test_and_snapshot();
}
