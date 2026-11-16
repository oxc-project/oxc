use oxc_ast::{
    AstKind,
    ast::{Expression, LabeledStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{AstNode, context::LintContext, rule::Rule};

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
    conditional_fix,
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
            ctx.diagnostic_with_fix(
                no_unused_labels_diagnostic(stmt.label.name.as_str(), stmt.label.span),
                |fixer| {
                    if can_remove_label(node, stmt, ctx) {
                        fixer.replace_with(stmt, &stmt.body)
                    } else {
                        fixer.noop()
                    }
                },
            );
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext != "svelte")
    }
}

fn can_remove_label(node: &AstNode, stmt: &LabeledStatement, ctx: &LintContext) -> bool {
    let Statement::ExpressionStatement(body) = &stmt.body else {
        return true;
    };

    // These tokens can continue the previous statement once the label is gone.
    if ctx.source_range(body.span).starts_with(['[', '(', '`', '/', '+', '-', '<']) {
        return false;
    }

    if !matches!(body.expression, Expression::StringLiteral(_)) {
        return true;
    }

    // A labeled string terminates the directive prologue. Removing even a
    // non-"use strict" string's label can make a later "use strict" a directive.
    let mut outer_span = stmt.span;
    for ancestor in ctx.nodes().ancestors(node.id()) {
        match ancestor.kind() {
            AstKind::LabeledStatement(outer) => outer_span = outer.span,
            AstKind::Program(program) => {
                return program.body.first().is_none_or(|first| first.span() != outer_span);
            }
            AstKind::FunctionBody(body) => {
                return body.statements.first().is_none_or(|first| first.span() != outer_span);
            }
            _ => return true,
        }
    }
    true
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
        (r#"A: "use strict""#, r#"A: "use strict""#),
        (
            r#"function f(a = 1) { unused: "use strict"; return a; }"#,
            r#"function f(a = 1) { unused: "use strict"; return a; }"#,
        ),
        (
            r#"function f() { unused: "hello"; "use strict"; return this; }"#,
            r#"function f() { unused: "hello"; "use strict"; return this; }"#,
        ),
        (
            r#"function f() { foo(); unused: "use strict"; }"#,
            r#"function f() { foo(); "use strict"; }"#,
        ),
        (
            "const a = [10, 20]\nunused: [0].forEach(console.log)",
            "const a = [10, 20]\nunused: [0].forEach(console.log)",
        ),
        ("foo()\nunused: (bar)()", "foo()\nunused: (bar)()"),
        ("foo()\nunused: `bar`", "foo()\nunused: `bar`"),
        ("foo()\nunused: /bar/.test(value)", "foo()\nunused: /bar/.test(value)"),
        ("foo()\nunused: +bar", "foo()\nunused: +bar"),
        ("foo()\nunused: -bar", "foo()\nunused: -bar"),
    ];

    Tester::new(NoUnusedLabels::NAME, NoUnusedLabels::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
