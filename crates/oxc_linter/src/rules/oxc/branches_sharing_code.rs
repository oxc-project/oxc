// Inspired by https://github.com/rust-lang/rust-clippy/blob/95f5a00d8628fba223c802251af3c42547927c5a/clippy_lints/src/ifs/branches_sharing_code.rs
use smallvec::SmallVec;

use oxc_ast::{
    AstKind,
    ast::{IdentifierReference, IfStatement, Statement},
};
use oxc_ast_visit::Visit;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::ScopeId;
use oxc_span::{ContentEq, GetSpan, Span};

use crate::{AstNode, ast_util::get_preceding_indent_str, context::LintContext, rule::Rule};

const MOVE_SHARED_CODE_HELP: &str =
    "Move the shared code outside the `if` statement to reduce code duplication";

fn branches_sharing_code_at_start_diagnostic(
    span: Span,
    duplicated_code_spans: impl Iterator<Item = Span>,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("All `if` blocks contain the same code at the start").with_labels(
        std::iter::once(span.primary_label("`if` statement declared here"))
            .chain(duplicated_code_spans.map(Into::into)),
    )
}

fn branches_sharing_code_at_end_diagnostic(
    span: Span,
    duplicated_code_spans: impl Iterator<Item = Span>,
) -> OxcDiagnostic {
    OxcDiagnostic::warn("All `if` blocks contain the same code at the end").with_labels(
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
    short_description = "Checks if the `if` and `else` blocks contain shared code that can be moved out of the blocks.",
);

impl Rule for BranchesSharingCode {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::IfStatement(if_stmt) = node.kind() else {
            return;
        };

        // Analyze the entire chain from its root instead of reporting on an `else if` suffix.
        if let AstKind::IfStatement(parent_if) = ctx.nodes().parent_kind(node.id())
            && parent_if
                .alternate
                .as_ref()
                .is_some_and(|alternate| alternate.span() == if_stmt.span)
        {
            return;
        }

        let Some(bodies) = extract_if_sequence(if_stmt) else {
            return;
        };

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
                && !duplicated_code_references_branch_locals(&bodies, false, ctx)
            {
                let delete_spans =
                    bodies.iter().map(|body| get_duplicated_delete_span(start, body, false));
                let moved_code = ctx.source_range(spans[0]);
                let diagnostic = diagnostic.with_help(MOVE_SHARED_CODE_HELP);
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
                && !duplicated_code_references_branch_locals(&bodies, true, ctx)
            {
                let delete_spans =
                    bodies.iter().map(|body| get_duplicated_delete_span(end, body, true));
                let moved_code = ctx.source_range(spans[0]);
                let diagnostic = diagnostic.with_help(MOVE_SHARED_CODE_HELP);
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

fn duplicated_code_references_branch_locals<'a>(
    bodies: &[&Statement<'a>],
    reverse: bool,
    ctx: &LintContext<'a>,
) -> bool {
    bodies.iter().any(|body| {
        let Statement::BlockStatement(block) = body else {
            return false;
        };
        let scope_id = block.scope_id();
        let scoping = ctx.scoping();
        if scoping.get_bindings(scope_id).is_empty()
            && (!ctx.source_type().is_javascript()
                || scoping.scope_flags(scope_id).is_strict_mode()
                || !block.body.iter().any(is_hoistable_block_function))
        {
            return false;
        }

        let statement = if reverse { block.body.last() } else { block.body.first() };
        let mut visitor = BranchLocalReferenceFinder { ctx, scope_id, found: false };
        if let Some(statement) = statement {
            visitor.visit_statement(statement);
        }
        visitor.found
    })
}

fn is_hoistable_block_function(mut statement: &Statement) -> bool {
    while let Statement::LabeledStatement(labeled) = statement {
        statement = &labeled.body;
    }
    matches!(statement, Statement::FunctionDeclaration(func) if !func.r#async && !func.generator)
}

struct BranchLocalReferenceFinder<'c, 'a> {
    ctx: &'c LintContext<'a>,
    scope_id: ScopeId,
    found: bool,
}

impl<'a> Visit<'a> for BranchLocalReferenceFinder<'_, 'a> {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        if self.found {
            return;
        }
        let scoping = self.ctx.scoping();
        let Some(symbol_id) = scoping.get_reference(ident.reference_id()).symbol_id() else {
            return;
        };
        if scoping.symbol_scope_id(symbol_id) == self.scope_id {
            self.found = true;
            return;
        }
        if !scoping.symbol_flags(symbol_id).is_function() {
            return;
        }

        // Annex B block functions can be recorded as bindings of the enclosing var scope.
        let declaration = scoping.symbol_declaration(symbol_id);
        let nodes = self.ctx.nodes();
        self.found = matches!(nodes.kind(declaration), AstKind::Function(func) if func.is_declaration())
            && matches!(
                nodes.ancestor_kinds(declaration).find(|kind| !matches!(kind, AstKind::LabeledStatement(_))),
                Some(AstKind::BlockStatement(block)) if block.scope_id() == self.scope_id
            );
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
) -> Option<SmallVec<[&'a Statement<'a>; 2]>> {
    let mut bodies = SmallVec::new();

    let mut current_if = if_stmt;
    loop {
        bodies.push(&current_if.consequent);

        match current_if.alternate.as_ref()? {
            Statement::IfStatement(else_if) => {
                current_if = else_if;
            }
            else_stmt => {
                bodies.push(else_stmt);
                return Some(bodies);
            }
        }
    }
}

#[test]
fn test() {
    use crate::{
        fixer::FixKind,
        tester::{ExpectFixTestCase, Tester},
    };

    let shared_suffix_references_class = r"
        function f(flag, use, C) {
            if (flag) {
                class C {}
                use(C);
            } else {
                class C { m() {} }
                use(C);
            }
        }
        ";

    let shared_prefix_references_function = r"
        function g(flag, use) {
            if (flag) {
                use(() => f());
                first();
            } else {
                use(() => f());
                function f() {}
            }
        }
        ";

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
        r"
        if (someCondition) {
            y = somethingElse;
            x = '1';
        } else if (someOtherCondition) {
            y = somethingElse;
            x = 'b';
        } else {
            viewId = accountId;
            x = 'b';
        }
        ",
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
        shared_suffix_references_class,
        shared_prefix_references_function,
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
        (shared_suffix_references_class, shared_suffix_references_class, None, FixKind::Suggestion),
        (
            r"
        if (flag) {
            const x = 1;
            use(function self() { return self; });
        } else {
            const x = 2;
            use(function self() { return self; });
        }
        ",
            r"
        if (flag) {
            const x = 1;
        } else {
            const x = 2;
        }
        use(function self() { return self; });
        ",
            None,
            FixKind::Suggestion,
        ),
        (
            shared_prefix_references_function,
            shared_prefix_references_function,
            None,
            FixKind::Suggestion,
        ),
        (
            r"
        if (flag) {
            class C {}
            use(C => C);
        } else {
            class C { m() {} }
            use(C => C);
        }
        ",
            r"
        if (flag) {
            class C {}
        } else {
            class C { m() {} }
        }
        use(C => C);
        ",
            None,
            FixKind::Suggestion,
        ),
    ];

    let mut fix: Vec<ExpectFixTestCase> = fix.into_iter().map(Into::into).collect();
    fix.push(
        (
            shared_prefix_references_function,
            shared_prefix_references_function,
            None,
            Some(std::path::PathBuf::from("branches_sharing_code.cjs")),
        )
            .into(),
    );
    fix.push(
        (
            r"
        if (flag) {
            use(typeof f);
            if (which) function f() {}
        } else {
            use(typeof f);
            other();
        }
        ",
            r"
        use(typeof f);
        if (flag) {
            if (which) function f() {}
        } else {
            other();
        }
        ",
            None,
            Some(std::path::PathBuf::from("branches_sharing_code.cjs")),
        )
            .into(),
    );

    Tester::new(BranchesSharingCode::NAME, BranchesSharingCode::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
