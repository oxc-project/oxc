use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule};

fn no_unused_labels_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("eslint(no-unused-labels): Disallow unused labels")
        .with_help(format!("'{x0}:' is defined but never used."))
        .with_label(span1)
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
    correctness
);

impl Rule for NoUnusedLabels {
    fn run_once(&self, ctx: &LintContext) {
        if ctx.file_path().extension().is_some_and(|ext| ext == "svelte") {
            return;
        }
        for id in ctx.semantic().unused_labels() {
            let node = ctx.semantic().nodes().get_node(*id);
            if let AstKind::LabeledStatement(stmt) = node.kind() {
                // TODO: Ignore fix where comments exist between label and statement
                // e.g. A: /* Comment */ function foo(){}
                ctx.diagnostic_with_fix(
                    no_unused_labels_diagnostic(stmt.label.name.as_str(), stmt.label.span),
                    |fixer| fixer.delete_range(stmt.label.span),
                );
            }
        }
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

    Tester::new(NoUnusedLabels::NAME, pass, fail).test_and_snapshot();
}
