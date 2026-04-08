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
    suggestion,
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

        let break_text = ctx.source_range(last_statement.span());
        let source_text = ctx.source_text();

        // Figure out indentation of statements inside the block
        let last_body_stmt = block_statement.body.last().unwrap();
        let body_indent = {
            let before = &source_text[..last_body_stmt.span().start as usize];
            let line_start = before.rfind('\n').map_or(0, |i| i + 1);
            &source_text[line_start..last_body_stmt.span().start as usize]
        };

        // Figure out indentation of the closing brace
        let brace_pos = (block_statement.span.end - 1) as usize;
        let brace_indent = {
            let before = &source_text[..brace_pos];
            let line_start = before.rfind('\n').map_or(0, |i| i + 1);
            &source_text[line_start..brace_pos]
        };

        // Find start of whitespace/newline before the closing brace
        let before_brace = &source_text[..brace_pos];
        let brace_line_start = before_brace.rfind('\n').map_or(0, |i| i);

        // Replace from before the closing brace's newline through the break statement
        // This preserves all content inside the block (including comments) and moves
        // the break before the closing brace
        let replace_span = Span::new(brace_line_start as u32, last_statement.span().end);

        ctx.diagnostic_with_suggestion(
            switch_case_break_position_diagnostic(last_statement.span(), keyword),
            |fixer| {
                fixer
                    .replace(replace_span, format!("\n{body_indent}{break_text}\n{brace_indent}}}"))
                    .with_message(format!("Move `{keyword}` inside the block statement."))
            },
        );
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

    let fix = vec![
        (
            "switch(foo) {
                case 1: {
                    doStuff();
                }
                break;
            }",
            "switch(foo) {
                case 1: {
                    doStuff();
                    break;
                }
            }",
        ),
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
        .expect_fix(fix)
        .test_and_snapshot();
}
