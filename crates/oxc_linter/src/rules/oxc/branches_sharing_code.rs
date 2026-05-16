// Inspired by https://github.com/rust-lang/rust-clippy/blob/95f5a00d8628fba223c802251af3c42547927c5a/clippy_lints/src/ifs/branches_sharing_code.rs
use oxc_ast::{
    AstKind,
    ast::{Expression, IdentifierReference, IfStatement, Statement},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, SymbolId};
use oxc_span::{ContentEq, GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{AstNode, ast_util::get_preceding_indent_str, context::LintContext, rule::Rule};

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
    pedantic,
    suggestion,
    version = "1.22.0",
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

        let if_span = Span::sized(if_stmt.span.start, 2);

        if let Some(start) =
            start_eq.filter(|&start| !duplicated_stmts_are_empty(start, &bodies, false))
        {
            let spans = bodies
                .iter()
                .map(|body| get_duplicated_span(start, body, false))
                .collect::<Vec<_>>();
            let diagnostic =
                branches_sharing_code_at_start_diagnostic(if_span, spans.iter().copied());

            if start == 1
                && let Some(indent) = get_preceding_indent_str(ctx.source_text(), if_stmt.span)
            {
                let delete_spans = bodies
                    .iter()
                    .map(|body| get_duplicated_delete_span(start, body, false))
                    .collect::<Vec<_>>();
                let moved_code = ctx.source_range(spans[0]).to_string();
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let fixer = fixer.for_multifix();
                    let mut fix = fixer.new_fix_with_capacity(spans.len() + 1);
                    fix.push(fixer.insert_text_before(if_stmt, format!("{moved_code}\n{indent}")));
                    for span in delete_spans {
                        fix.push(fixer.delete_range(span));
                    }
                    fix.with_message("Move the shared statements before the `if` statement.")
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
        }

        if let Some(end) = end_eq.filter(|&end| !duplicated_stmts_are_empty(end, &bodies, true)) {
            let spans =
                bodies.iter().map(|body| get_duplicated_span(end, body, true)).collect::<Vec<_>>();
            let diagnostic =
                branches_sharing_code_at_end_diagnostic(if_span, spans.iter().copied());

            if end == 1
                && let Some(indent) = get_preceding_indent_str(ctx.source_text(), if_stmt.span)
                && !duplicated_end_references_branch_locals(end, &bodies, ctx)
            {
                let delete_spans = bodies
                    .iter()
                    .map(|body| get_duplicated_delete_span(end, body, true))
                    .collect::<Vec<_>>();
                let moved_code = ctx.source_range(spans[0]).to_string();
                ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                    let fixer = fixer.for_multifix();
                    let mut fix = fixer.new_fix_with_capacity(spans.len() + 1);
                    for span in delete_spans {
                        fix.push(fixer.delete_range(span));
                    }
                    fix.push(fixer.insert_text_after(if_stmt, format!("\n{indent}{moved_code}")));
                    fix.with_message("Move the shared statements after the `if` statement.")
                });
            } else {
                ctx.diagnostic(diagnostic);
            }
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

fn get_duplicated_delete_span(count: usize, body: &Statement, reverse: bool) -> Span {
    let stmts = get_block_statements(body);
    let duplicated_span = get_duplicated_span(count, body, reverse);

    if reverse {
        let start_idx = stmts.len() - count;
        return Span::new(
            start_idx
                .checked_sub(1)
                .and_then(|idx| stmts.get(idx))
                .map_or(duplicated_span.start, |s| s.span().end),
            duplicated_span.end,
        );
    }

    Span::new(
        duplicated_span.start,
        stmts.get(count).map_or(duplicated_span.end, |s| s.span().start),
    )
}

fn duplicated_end_references_branch_locals(
    count: usize,
    bodies: &[&Statement],
    ctx: &LintContext,
) -> bool {
    bodies.iter().any(|body| {
        let stmts = get_block_statements(body);
        let start_idx = stmts.len() - count;
        let mut symbols = FxHashSet::default();

        for stmt in &stmts[..start_idx] {
            collect_lexical_declaration_symbols(stmt, &mut symbols);
        }

        if symbols.is_empty() {
            return false;
        }

        duplicated_statements(body, count, true).iter().any(|stmt| {
            let mut references = ReferenceCollector::default();
            references.visit_statement(stmt);
            references.references.iter().any(|reference_id| {
                ctx.scoping()
                    .get_reference(*reference_id)
                    .symbol_id()
                    .is_some_and(|symbol_id| symbols.contains(&symbol_id))
            })
        })
    })
}

fn duplicated_statements<'a>(
    body: &'a Statement<'a>,
    count: usize,
    reverse: bool,
) -> &'a [Statement<'a>] {
    let stmts = get_block_statements(body);
    if reverse { &stmts[stmts.len() - count..] } else { &stmts[..count] }
}

fn collect_lexical_declaration_symbols(stmt: &Statement, symbols: &mut FxHashSet<SymbolId>) {
    if let Statement::VariableDeclaration(decl) = stmt
        && decl.kind.is_lexical()
    {
        symbols.extend(decl.declarations.iter().flat_map(|decl| decl.id.get_symbol_ids()));
    }
}

#[derive(Default)]
struct ReferenceCollector {
    references: FxHashSet<ReferenceId>,
}

impl<'a> Visit<'a> for ReferenceCollector {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.references.insert(ident.reference_id());
    }
}

fn duplicated_stmts_are_empty(count: usize, bodies: &[&Statement], reverse: bool) -> bool {
    bodies.iter().all(|body| {
        let stmts = get_block_statements(body);
        let range = if reverse { &stmts[stmts.len() - count..] } else { &stmts[..count] };
        range.iter().all(|stmt| matches!(stmt, Statement::EmptyStatement(_)))
    })
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
    use crate::{fixer::FixKind, tester::Tester};

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
        r"
        if (isArray(value)) {
            ;(originObject as IAnyObject)[property] = [...value]
        } else if (isPlainObject(value)) {
            ;(originObject as IAnyObject)[property] = { ...value }
        } else {
            ;(originObject as IAnyObject)[property] = assignObject[property] as unknown
        }
        ",
        r#"
        if (maybe) {
            if (maybe2) { console.log("maybe and maybe2"); }
        } else {
            if (maybe2) { console.log("not maybe and maybe2"); }
        }
        "#,
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

    let fix = vec![
        (
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
        console.log('hello');
        if (condition) {
            doA();
        } else {
            doB();
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        if (condition) {
            doA();
        } else {
            doB();
        }
        cleanup();
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        let foo;
        console.log('start');
        if (condition) {
            foo = 13;
        } else {
            foo = 42;
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        console.log('before');
        if (x) {
            doX();
            console.log('after');
        } else {
            doY();
            console.log('after');
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        initialize();
        if (flag) {
            processData();
            finalize();
        } else {
            processOtherData();
            finalize();
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        a++;
        if (test) {
            b++;
        } else {
            c++;
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        if (x > 0) {
            const a = 1;
            console.log(a);
        } else {
            const a = 2;
            console.log(a);
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
        console.log('start');
        if (flag) {
            doA();
        } else if (otherFlag) {
            doB();
        } else {
            doC();
        }
        ",
            None,
            FixKind::Suggestion,
        ),
        (
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
            r"
        setup();
        if (x === 1) {
            return 1;
        } else if (x === 2) {
            return 2;
        } else {
            return 3;
        }
        ",
            None,
            FixKind::Suggestion,
        ),
    ];

    Tester::new(BranchesSharingCode::NAME, BranchesSharingCode::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
