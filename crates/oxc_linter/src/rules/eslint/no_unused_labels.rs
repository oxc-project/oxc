use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_unused_labels_diagnostic(label_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{label_name}:' is defined but never used.")).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoUnusedLabels;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unused labels
    ///
    ///
    /// ### Why is this bad?
    ///
    /// Labels that are declared and not used anywhere in the code are most likely an error due to incomplete refactoring.
    ///
    /// ### Example
    /// ```javascript
    /// OUTER_LOOP:
    /// for (const student of students) {
    ///     if (checkScores(student.scores)) {
    ///         continue;
    ///     }
    ///     doSomething(student);
    /// }
    /// ```
    NoUnusedLabels,
    correctness,
    fix
);

impl Rule for NoUnusedLabels {
    fn run_once(&self, ctx: &LintContext) {
        for id in ctx.semantic().unused_labels() {
            let node = ctx.semantic().nodes().get_node(*id);
            let AstKind::LabeledStatement(stmt) = node.kind() else {
                continue;
            };
            ctx.diagnostic_with_fix(
                no_unused_labels_diagnostic(stmt.label.name.as_str(), stmt.label.span),
                |fixer| fixer.replace_with(stmt, &stmt.body),
            );
        }
    }

    fn should_run(&self, ctx: &crate::context::ContextHost) -> bool {
        ctx.file_path().extension().is_some_and(|ext| ext != "svelte")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("A: break A;", None),
        ("A: { foo(); break A; bar(); }", None),
        ("A: if (a) { foo(); if (b) break A; bar(); }", None),
        ("A: for (var i = 0; i < 10; ++i) { foo(); if (a) break A; bar(); }", None),
        ("A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue A; bar(); }", None),
        (
            "A: { B: break B; C: for (var i = 0; i < 10; ++i) { foo(); if (a) break A; if (c) continue C; bar(); } }",
            None,
        ),
        ("A: { var A = 0; console.log(A); break A; console.log(A); }", None),
    ];

    let fail = vec![
        ("A: var foo = 0;", None),
        ("A: { foo(); bar(); }", None),
        ("A: if (a) { foo(); bar(); }", None),
        ("A: for (var i = 0; i < 10; ++i) { foo(); if (a) break; bar(); }", None),
        ("A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue; bar(); }", None),
        ("A: for (var i = 0; i < 10; ++i) { B: break A; }", None),
        ("A: { var A = 0; console.log(A); }", None),
        ("A: /* comment */ foo", None),
        ("A /* comment */: foo", None),
    ];
    let fix = vec![
        ("A: var foo = 0;", "var foo = 0;", None),
        ("A: /* comment */ foo", "foo", None),
        ("A /* comment */: foo", "foo", None),
        (
            "A: for (var i = 0; i < 10; ++i) { B: break A; }",
            "A: for (var i = 0; i < 10; ++i) { break A; }",
            None,
        ),
    ];

    Tester::new(NoUnusedLabels::NAME, pass, fail).expect_fix(fix).test_and_snapshot();
}
