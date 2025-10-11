// Inspired by https://github.com/rust-lang/rust-clippy/blob/95f5a00d8628fba223c802251af3c42547927c5a/clippy_lints/src/ifs/branches_sharing_code.rs
use oxc_ast::{
    AstKind,
    ast::{Expression, IfStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{ContentEq, GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

fn branches_sharing_code_at_start_diagnostic(
    span: Span,
    duplicated_code_spans: impl Iterator<Item = Span>,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("All `if` blocks contain the same code at the start")
        .with_help("Move the shared code outside the `if` statement to reduce code duplication")
        .with_labels(
            std::iter::once(span.primary_label("`if` statement declared here"))
                .chain(duplicated_code_spans.map(Into::into)),
        )
}

fn branches_sharing_code_at_end_diagnostic(
    span: Span,
    duplicated_code_spans: impl Iterator<Item = Span>,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("All `if` blocks contain the same code at the end")
        .with_help("Move the shared code outside the `if` statement to reduce code duplication")
        .with_labels(
            std::iter::once(span.primary_label("`if` statement declared here"))
                .chain(duplicated_code_spans.map(Into::into)),
        )
}

#[derive(Debug, Default, Clone)]
pub struct BranchesSharingCode;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Checks if the `if` and `else` blocks contain shared code that can be moved out of the blocks.
    ///
    /// ### Why is this bad?
    ///
    /// Duplicate code is less maintainable. Extracting common code from branches makes the code more DRY (Don't Repeat Yourself)
    /// and easier to maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (condition) {
    ///     console.log("Hello");
    ///     return 13;
    /// } else {
    ///     console.log("Hello");
    ///     return 42;
    /// };
    ///
    /// if (condition) {
    ///     doSomething();
    ///     cleanup();
    /// } else {
    ///     doSomethingElse();
    ///     cleanup();
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// console.log("Hello");
    /// if (condition) {
    ///     return 13;
    /// } else {
    ///     return 42;
    /// };
    ///
    /// if (condition) {
    ///     doSomething();
    /// } else {
    ///     doSomethingElse();
    /// }
    /// cleanup();
    /// ```
    BranchesSharingCode,
    oxc,
    nursery
);

impl Rule for BranchesSharingCode {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        let (conditions, bodies) = extract_if_sequence(if_stmt);

        if bodies.len() < 2 || bodies.len() == conditions.len() {
            return;
        }

        let (start_eq, end_eq) = scan_blocks_for_eq(&bodies);

        let if_span = Span::new(if_stmt.span.start, if_stmt.span.start + 2);

        if let Some(start) = start_eq {
            let spans = bodies.iter().map(|body| get_duplicated_span(start, body, false));
            ctx.diagnostic(branches_sharing_code_at_start_diagnostic(if_span, spans));
        }

        if let Some(end) = end_eq {
            let spans = bodies.iter().map(|body| get_duplicated_span(end, body, true));
            ctx.diagnostic(branches_sharing_code_at_end_diagnostic(if_span, spans));
        }
    }
}

fn get_block_statements<'a>(stmt: &'a Statement<'a>) -> &'a [Statement<'a>] {
    match stmt {
        Statement::BlockStatement(block) => &block.body,
        _ => &[],
    }
}

fn get_duplicated_span(count: usize, body: &Statement, reverse: bool) -> Span {
    let stmts = get_block_statements(body);
    let range = if reverse { &stmts[stmts.len() - count..] } else { &stmts[..count] };
    let start = range.first().map(|s| s.span().start).unwrap();
    let end = range.last().map(|s| s.span().end).unwrap();
    Span::new(start, end)
}

fn scan_blocks_for_eq<'a>(bodies: &[&'a Statement<'a>]) -> (Option<usize>, Option<usize>) {
    let first_stmts = get_block_statements(bodies[0]);

    let min_stmt_count =
        bodies.iter().map(|body| get_block_statements(body).len()).min().unwrap_or(0);

    let start_end_eq = first_stmts
        .iter()
        .enumerate()
        .take_while(|(i, stmt)| {
            bodies[1..].iter().all(|body| {
                let stmts = get_block_statements(body);
                stmts.get(*i).is_some_and(|s| s.content_eq(stmt))
            })
        })
        .count();

    if start_end_eq >= min_stmt_count {
        return (None, None);
    }

    let max_end_search = min_stmt_count - start_end_eq;

    let end_begin_eq = (1..=max_end_search)
        .take_while(|&offset| {
            bodies.iter().all(|body| {
                let stmts = get_block_statements(body);
                let idx = stmts.len() - offset;
                let stmt = &stmts[idx];
                let first_stmt = &first_stmts[first_stmts.len() - offset];
                stmt.content_eq(first_stmt)
            })
        })
        .count();

    let has_remaining_code_for_start = start_end_eq > 0
        && bodies.iter().any(|body| {
            let stmts = get_block_statements(body);
            stmts.len() > start_end_eq
        });

    let has_remaining_code_for_end = end_begin_eq > 0
        && bodies.iter().any(|body| {
            let stmts = get_block_statements(body);
            let start_idx = stmts.len().saturating_sub(end_begin_eq);
            start_idx > start_end_eq
        });

    (
        if has_remaining_code_for_start { Some(start_end_eq) } else { None },
        if has_remaining_code_for_end { Some(end_begin_eq) } else { None },
    )
}

fn extract_if_sequence<'a>(
    if_stmt: &'a IfStatement<'a>,
) -> (Vec<&'a Expression<'a>>, Vec<&'a Statement<'a>>) {
    let mut conditions = Vec::new();
    let mut bodies = Vec::new();

    let mut current_if = Some(if_stmt);
    while let Some(if_node) = current_if {
        conditions.push(&if_node.test);
        bodies.push(&if_node.consequent);

        match &if_node.alternate {
            Some(Statement::IfStatement(else_if)) => {
                current_if = Some(else_if);
            }
            Some(else_stmt) => {
                bodies.push(else_stmt);
                current_if = None;
            }
            None => {
                current_if = None;
            }
        }
    }

    (conditions, bodies)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (condition) { foo(); } else { bar(); }",
        "if (condition) { foo(); bar(); } else { baz(); qux(); }",
        "if (condition) { foo(); }",
        "if (condition) { foo(); } else { foo(); bar(); }",
        "if (condition) { foo(); bar(); } else { foo(); }",
        "if (condition) { foo(); bar(); } else if (condition2) { foo(); }",
        r"
        if (condition) {
            doA();
            doB();
        } else {
            doC();
            doD();
        }
        ",
        r"
        if (x > 0) {
            const a = 1;
            console.log('positive');
        } else {
            const a = 2;
            console.log('negative');
        }
        ",
        "if (condition) {} else {}",
    ];

    let fail = vec![
        r"
        if (condition) {
            console.log('hello');
            doA();
        } else {
            console.log('hello');
            doB();
        }
        ",
        r"
        if (condition) {
            doA();
            cleanup();
        } else {
            doB();
            cleanup();
        }
        ",
        r"
        let foo;
        if (condition) {
            console.log('start');
            foo = 13;
        } else {
            console.log('start');
            foo = 42;
        }
        ",
        r"
        if (x) {
            console.log('before');
            doX();
            console.log('after');
        } else {
            console.log('before');
            doY();
            console.log('after');
        }
        ",
        r"
        if (flag) {
            initialize();
            processData();
            finalize();
        } else {
            initialize();
            processOtherData();
            finalize();
        }
        ",
        r"
        if (test) {
            a++;
            b++;
        } else {
            a++;
            c++;
        }
        ",
        r"
        if (x > 0) {
            const a = 1;
            console.log(a);
        } else {
            const a = 2;
            console.log(a);
        }
        ",
        r"
        if (flag) {
            console.log('start');
            doA();
        } else if (otherFlag) {
            console.log('start');
            doB();
        } else {
            console.log('start');
            doC();
        }
        ",
        r"
        if (x === 1) {
            setup();
            return 1;
        } else if (x === 2) {
            setup();
            return 2;
        } else {
            setup();
            return 3;
        }
        ",
    ];

    Tester::new(BranchesSharingCode::NAME, BranchesSharingCode::PLUGIN, pass, fail)
        .test_and_snapshot();
}
