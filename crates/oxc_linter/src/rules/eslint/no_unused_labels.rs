use oxc_ast::{
    AstKind,
    ast::{Expression, LabeledStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{ast_util::find_last_meaningful_char, context::LintContext, rule::Rule};

fn no_unused_labels_diagnostic(label_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{label_name}:' is defined but never used.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedLabels;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unused labels.
    ///
    /// ### Why is this bad?
    ///
    /// Labels that are declared and not used anywhere in the code are most likely an error due to incomplete refactoring.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// OUTER_LOOP:
    /// for (const student of students) {
    ///     if (checkScores(student.scores)) {
    ///         continue;
    ///     }
    ///     doSomething(student);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// for (const student of students) {
    ///     if (checkScores(student.scores)) {
    ///         continue;
    ///     }
    ///     doSomething(student);
    /// }
    /// ```
    NoUnusedLabels,
    eslint,
    correctness,
    recommended,
    fix,
    version = "0.0.3",
    short_description = "Disallow unused labels.",
);

impl Rule for NoUnusedLabels {
    fn run_once(&self, ctx: &LintContext) {
        for id in ctx.unused_labels() {
            let node = ctx.nodes().get_node(*id);
            let AstKind::LabeledStatement(stmt) = node.kind() else {
                continue;
            };
            let diagnostic = no_unused_labels_diagnostic(stmt.label.name.as_str(), stmt.label.span);
            if is_fixable(stmt, *id, ctx) {
                ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.replace_with(stmt, &stmt.body));
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext != "svelte")
    }
}

/// Can the label be removed without changing how the body is parsed?
///
/// Ported from `isFixable` in ESLint's `no-unused-labels`.
fn is_fixable(stmt: &LabeledStatement, id: NodeId, ctx: &LintContext) -> bool {
    // `A: "use strict"` -> `"use strict"` would become a directive
    if let Statement::ExpressionStatement(expr_stmt) = &stmt.body
        && match expr_stmt.expression.without_parentheses() {
            Expression::StringLiteral(_) => true,
            Expression::TemplateLiteral(tpl) => tpl.is_no_substitution_template(),
            _ => false,
        }
        && ctx
            .nodes()
            .ancestor_kinds(id)
            .find(|kind| !matches!(kind, AstKind::LabeledStatement(_)))
            .is_some_and(|kind| matches!(kind, AstKind::Program(_) | AstKind::FunctionBody(_)))
    {
        return false;
    }

    // `foo()\nA: [1].forEach(f)` -> `foo()\n[1].forEach(f)` would continue the previous line
    let source_text = ctx.source_text();
    source_text[stmt.body.span().start as usize..]
        .chars()
        .next()
        .is_none_or(|c| !matches!(c, '(' | '[' | '-' | '+' | '/' | '`'))
        || find_last_meaningful_char(source_text, stmt.span.start, ctx)
            .is_none_or(|c| matches!(c, ':' | ';' | '{'))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "A: break A;",
        "A: { foo(); break A; bar(); }",
        "A: if (a) { foo(); if (b) break A; bar(); }",
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) break A; bar(); }",
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue A; bar(); }",
        "A: { B: break B; C: for (var i = 0; i < 10; ++i) { foo(); if (a) break A; if (c) continue C; bar(); } }",
        "A: { var A = 0; console.log(A); break A; console.log(A); }",
        "label: while (true) { f = function() { label: while (true) { break label; } }; break label; }",
        "outer: { function f() { inner: { break inner; } } break outer; }",
        "A: { const f = () => { B: { break B; } }; break A; }",
    ];

    let fail = vec![
        "A: var foo = 0;",
        "A: { foo(); bar(); }",
        "A: if (a) { foo(); bar(); }",
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) break; bar(); }",
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue; bar(); }",
        "A: for (var i = 0; i < 10; ++i) { B: break A; }",
        "A: { var A = 0; console.log(A); }",
        "A: /* comment */ foo",
        "A /* comment */: foo",
        r#"A: "use strict""#,
        r#""use strict"; foo: "bar""#,
        r#"A: ("use strict")"#,
        "A: `use strict`", // { "ecmaVersion": 6 },
        "if (foo) { bar: 'baz' }",
        "A: B: 'foo'",
        "A: B: C: 'foo'",
        "A: B: C: D: 'foo'",
        "A: B: C: D: E: 'foo'",
        "A: 42",
        // Ensure inner label in function is still marked as unused when not used
        "A: { function f() { B: { } } break A; }",
        "label: while (true) { (() => { label: while (false) {} })(); }",
        "run()\nunused: [1].forEach(log)",
        "function f() { A: 'use strict' }",
    ];

    let fix = vec![
        ("A: var foo = 0;", "var foo = 0;"),
        ("A: /* comment */ foo", "foo"),
        ("A /* comment */: foo", "foo"),
        ("A: { foo(); bar(); }", "{ foo(); bar(); }"),
        ("A: if (a) { foo(); bar(); }", "if (a) { foo(); bar(); }"),
        (
            "A: for (var i = 0; i < 10; ++i) { foo(); if (a) break; bar(); }",
            "for (var i = 0; i < 10; ++i) { foo(); if (a) break; bar(); }",
        ),
        (
            "A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue; bar(); }",
            "for (var i = 0; i < 10; ++i) { foo(); if (a) continue; bar(); }",
        ),
        (
            "A: for (var i = 0; i < 10; ++i) { B: break A; }",
            "A: for (var i = 0; i < 10; ++i) { break A; }",
        ),
        ("A: { var A = 0; console.log(A); }", "{ var A = 0; console.log(A); }"),
        ("if (foo) { bar: 'baz' }", "if (foo) { 'baz' }"),
        ("A: B: 'foo'", "B: 'foo'"),
        ("A: B: C: 'foo'", "B: C: 'foo'"),
        // TODO: Fix the rule fixer to allow these.
        // ("A: B: C: D: 'foo'", "B: D: 'foo'"),
        // ("A: B: C: D: E: 'foo'", "B: D: E: 'foo'"),
        ("A: 42", "42"),
        // removing the label would create a directive
        (r#"A: "use strict""#, r#"A: "use strict""#),
        (r#""use strict"; foo: "bar""#, r#""use strict"; foo: "bar""#),
        (r#"A: ("use strict")"#, r#"A: ("use strict")"#),
        ("A: `use strict`", "A: `use strict`"),
        ("function f() { A: 'use strict' }", "function f() { A: 'use strict' }"),
        ("() => { A: 'use strict' }", "() => { A: 'use strict' }"),
        ("{ A: 'use strict' }", "{ 'use strict' }"),
        // removing the label would continue the previous statement
        ("run()\nunused: [1].forEach(log)", "run()\nunused: [1].forEach(log)"),
        ("run()\nA: (0, f)()", "run()\nA: (0, f)()"),
        ("run()\nA: `x`", "run()\nA: `x`"),
        ("run()\nA: /x/.test(s)", "run()\nA: /x/.test(s)"),
        ("run()\nA: -x", "run()\nA: -x"),
        ("run() /* c */\nA: +x", "run() /* c */\nA: +x"),
        ("run();\nunused: [1].forEach(log)", "run();\n[1].forEach(log)"),
        ("{ A: [1].forEach(log) }", "{ [1].forEach(log) }"),
        ("unused: [1].forEach(log)", "[1].forEach(log)"),
        ("run()\nA: foo()", "run()\nfoo()"),
    ];

    Tester::new(NoUnusedLabels::NAME, NoUnusedLabels::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
