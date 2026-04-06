use oxc_ast::{AstKind, ast::Statement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn switch_case_break_position_diagnostic(span: Span, keyword: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Move `{keyword}` inside the block statement.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct SwitchCaseBreakPosition;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce consistent `break`/`return`/`continue`/`throw` position in `case` clauses.
    ///
    /// ### Why is this bad?
    ///
    /// Enforce that terminating statements (`break`, `return`, `continue`, `throw`) appear inside the block statement of a `case` clause, not after it.
    /// This can happen when refactoring — for example, removing an `if` wrapper but leaving the `break` outside the braces.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// switch(foo) {
    /// 	case 1: {
    /// 		doStuff();
    /// 	}
    /// 	break;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// switch(foo) {
    /// 	case 1: {
    /// 		doStuff();
    /// 		break;
    /// 	}
    /// }
    /// ```
    SwitchCaseBreakPosition,
    unicorn,
    style,
    pending,
);

impl Rule for SwitchCaseBreakPosition {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::SwitchCase(switch_case) = node.kind() else {
            return;
        };

        let consequent = &switch_case.consequent;

        let [Statement::BlockStatement(block_statement), last_statement] = consequent.as_slice()
        else {
            return;
        };

        let keyword = match last_statement {
            Statement::BreakStatement(_) => "break",
            Statement::ReturnStatement(_) => "return",
            Statement::ContinueStatement(_) => "continue",
            Statement::ThrowStatement(_) => "throw",
            _ => return,
        };

        if block_statement.body.is_empty() {
            return;
        }

        ctx.diagnostic(switch_case_break_position_diagnostic(last_statement.span(), keyword));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "switch(foo) {
                case 1: {
                    doStuff();
                    break;
                }
            }",
        "function bar() {
                switch(foo) {
                    case 1: {
                        doStuff();
                        return;
                    }
                }
            }",
        "switch(foo) {
                case 1:
                    doStuff();
                    break;
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                case 2: {
                    doOtherStuff();
                    break;
                }
            }",
        "switch(foo) {
                case 1:
                case 2: {
                    doStuff();
                    break;
                }
            }",
        "switch(foo) {
                case 1: {
                    break;
                }
            }",
        "switch(foo) {
                default: {
                    doStuff();
                    break;
                }
            }",
        "switch(foo) {
                case 1: {}
                break;
            }",
    ];

    let fail = vec![
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                break;
            }",
        "function bar() {
                switch(foo) {
                    case 1: {
                        doStuff();
                    }
                    return;
                }
            }",
        "function bar() {
                switch(foo) {
                    case 1: {
                        doStuff();
                    }
                    return result;
                }
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                throw new Error('bad');
            }",
        "switch(foo) {
                default: {
                    doStuff();
                }
                break;
            }",
        "switch(foo) {
                case 1:
                    {
                        doStuff();
                    }
                    break;
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                    break;
                }
                case 2: {
                    doOtherStuff();
                }
                break;
            }",
        "for (const foo of items) {
                switch(foo) {
                    case 1: {
                        doStuff();
                    }
                    continue;
                }
            }",
        "outer: for (let i = 0; i < 3; i++) {
                switch(foo) {
                    case 1: {
                        doStuff();
                    }
                    break outer;
                }
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                // This break is intentional
                break;
            }",
        "function bar() {
                switch(foo) {
                    case 1: {
                        const value = 1;
                    }
                    return value;
                }
            }",
        "function bar() {
                switch(foo) {
                    case 1: {
                        const error = new Error('inner');
                    }
                    throw error;
                }
            }",
        "switch(foo) {
                case 1: {
                    doStuff(); // Keep this comment with the statement
                }
                break;
            }",
        "switch(foo) {
                case 1: {
                    doStuff(); /* Keep this block comment with the statement */
                }
                break;
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                    // Keep this comment before the inserted break
                }
                break;
            }",
        "switch(foo) {
                case 1: {
                    console.log(foo); // eslint-disable-line no-console
                }
                break;
            }",
        "switch(foo) { case 1: { doStuff(); } break; }",
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                break; // keep with break
            }",
        "switch(foo) {
                case 1: {
                    doStuff();
                }
                break; /* keep with break */
            }",
        "function foo() {
                switch(bar) {
                    case 1: {
                        doStuff();
                    }
                    return value; // keep with return
                }
            }",
    ];

    // TODO: add fixer
    #[expect(clippy::useless_vec)]
    let _fix = vec![
        (
            "switch(foo) {
                case 1: {
                    doStuff(); // Keep this comment with the statement
                }
                break;
            }",
            "switch(foo) {
                case 1: {
                    doStuff(); // Keep this comment with the statement
                    break;
                }
            }",
        ),
        (
            "switch(foo) {
                case 1: {
                    doStuff(); /* Keep this block comment with the statement */
                }
                break;
            }",
            "switch(foo) {
                case 1: {
                    doStuff(); /* Keep this block comment with the statement */
                    break;
                }
            }",
        ),
        (
            "switch(foo) {
                case 1: {
                    doStuff();
                    // Keep this comment before the inserted break
                }
                break;
            }",
            "switch(foo) {
                case 1: {
                    doStuff();
                    // Keep this comment before the inserted break
                    break;
                }
            }",
        ),
        (
            "switch(foo) {
                case 1: {
                    console.log(foo); // eslint-disable-line no-console
                }
                break;
            }",
            "switch(foo) {
                case 1: {
                    console.log(foo); // eslint-disable-line no-console
                    break;
                }
            }",
        ),
    ];

    Tester::new(SwitchCaseBreakPosition::NAME, SwitchCaseBreakPosition::PLUGIN, pass, fail)
        .test_and_snapshot();
}
