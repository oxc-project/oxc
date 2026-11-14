use oxc_ast::{
    AstKind,
    ast::{Expression, LabeledStatement, Statement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::{GetSpan, Span};

use crate::{ast_util, context::LintContext, rule::Rule};

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
            let label_name = stmt.label.name.as_str();
            let label_span = stmt.label.span;
            if is_fixable(stmt, node.id(), ctx) {
                ctx.diagnostic_with_fix(
                    no_unused_labels_diagnostic(label_name, label_span),
                    |fixer| fixer.replace_with(stmt, &stmt.body),
                );
            } else {
                ctx.diagnostic(no_unused_labels_diagnostic(label_name, label_span));
            }
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_extension().is_some_and(|ext| ext != "svelte")
    }
}

/// Returns `true` when removing the label would be safe.
fn is_fixable(stmt: &LabeledStatement, node_id: NodeId, ctx: &LintContext) -> bool {
    !removing_label_creates_directive(stmt, node_id, ctx)
        && !removing_label_creates_asi_hazard(stmt, ctx)
}

/// Removing a label around a string/template statement that sits at the start
/// of a script, module, or function body can turn the statement into a
/// directive prologue (e.g. `"use strict"`). ESLint skips the fix in that
/// situation; we mirror that behavior here.
fn removing_label_creates_directive(
    stmt: &LabeledStatement,
    node_id: NodeId,
    ctx: &LintContext,
) -> bool {
    let parent = ctx.nodes().parent_kind(node_id);
    let is_directive_container = matches!(parent, AstKind::Program(_) | AstKind::FunctionBody(_));
    if !is_directive_container {
        return false;
    }
    let Statement::ExpressionStatement(expr_stmt) = &stmt.body else {
        return false;
    };
    match &expr_stmt.expression {
        Expression::StringLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}

/// Removing a label can move the body to the start of a new line. If the
/// previous line ends without a semicolon (or other safe separator) and the body
/// starts with a token like `[`, `(`, `/`, `+`, `-`, or `` ` ``, ASI will join
/// the body onto the previous expression. ESLint skips the fix here too.
fn removing_label_creates_asi_hazard(stmt: &LabeledStatement, ctx: &LintContext) -> bool {
    let source_text = ctx.source_text();
    let Some(prev_char) = ast_util::find_last_meaningful_char(source_text, stmt.span.start, ctx)
    else {
        return false;
    };
    if matches!(prev_char, ':' | ';' | '{') {
        return false;
    }
    let Some(first_char) = source_text[stmt.body.span().start as usize..].chars().next() else {
        return false;
    };
    matches!(first_char, '[' | '(' | '/' | '+' | '-' | '`')
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
        // Directive hazards: removing the label would create a directive prologue.
        r#"function f() { A: "use strict"; }"#,
        r#"function f(a = 1) { A: "use strict"; }"#,
        "function f() { A: `use strict`; }",
        // ASI hazards: removing the label joins the body onto the previous line.
        "foo()\nA: [0].forEach(x => x)",
        "const a = [10, 20]\nunused: [0].forEach((i) => console.log(\"item\", i))",
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
        // Safe: the previous line ends with a semicolon, so there is no ASI hazard.
        ("foo();\nA: [0].forEach(x => x)", "foo();\n[0].forEach(x => x)"),
    ];

    Tester::new(NoUnusedLabels::NAME, NoUnusedLabels::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
