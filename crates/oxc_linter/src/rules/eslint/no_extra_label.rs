use oxc_ast::{ast::LabelIdentifier, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_extra_label_diagnostic(label: &LabelIdentifier) -> OxcDiagnostic {
    let label_name = &label.name;
    OxcDiagnostic::warn(format!("This label '{label_name}' is unnecessary"))
        .with_help(format!("Remove this label. It will have the same result because the labeled statement '{label_name}' has no nested loops or switches",))
        .with_label(label.span)
}

#[derive(Debug, Default, Clone)]
pub struct NoExtraLabel;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow unnecessary labels.
    ///
    /// ### Why is this bad?
    ///
    /// If a loop contains no nested loops or switches, labeling the loop is unnecessary.
    /// ```js
    /// A: while (a) {
    ///     break A;
    /// }
    /// ```
    /// You can achieve the same result by removing the label and using `break` or `continue` without a label.
    /// Probably those labels would confuse developers because they expect labels to jump to further.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// A: while (a) {
    ///     break A;
    /// }
    ///
    /// B: for (let i = 0; i < 10; ++i) {
    ///     break B;
    /// }
    ///
    /// C: switch (a) {
    ///     case 0:
    ///         break C;
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// while (a) {
    ///     break;
    /// }
    ///
    /// for (let i = 0; i < 10; ++i) {
    ///     break;
    /// }
    ///
    /// switch (a) {
    ///     case 0:
    ///         break;
    /// }
    ///
    /// A: {
    ///     break A;
    /// }
    ///
    /// B: while (a) {
    ///     while (b) {
    ///         break B;
    ///     }
    /// }
    ///
    /// C: switch (a) {
    ///     case 0:
    ///         while (b) {
    ///             break C;
    ///         }
    ///         break;
    /// }
    /// ```
    NoExtraLabel,
    eslint,
    style,
    fix
);

impl Rule for NoExtraLabel {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::BreakStatement(break_stmt) = node.kind() {
            if let Some(label) = &break_stmt.label {
                report_label_if_extra(label, node, ctx);
            }
        }
        if let AstKind::ContinueStatement(cont_stmt) = node.kind() {
            if let Some(label) = &cont_stmt.label {
                report_label_if_extra(label, node, ctx);
            }
        }
    }
}

fn report_label_if_extra(label: &LabelIdentifier, node: &AstNode, ctx: &LintContext) {
    let nodes = ctx.nodes();
    for ancestor_id in nodes.ancestor_ids(node.id()) {
        if !is_breakable_statement(nodes.kind(ancestor_id)) {
            continue;
        }
        let Some(AstKind::LabeledStatement(labeled_stmt)) = nodes.parent_kind(ancestor_id) else {
            return; // no need to check outer loops/switches
        };
        if labeled_stmt.label.name != label.name {
            return;
        }

        let keyword_len: u32 = match node.kind() {
            AstKind::BreakStatement(_) => 5,
            AstKind::ContinueStatement(_) => 8,
            _ => unreachable!(),
        };

        let keyword_end = node.span().start + keyword_len;
        let delete_span = Span::new(keyword_end, label.span.end);

        let diagnostic = no_extra_label_diagnostic(label);
        if ctx.comments().iter().any(|comment| delete_span.contains_inclusive(comment.span)) {
            // No autofix to avoid deleting comments between keyword and label
            // e.g. `break /* comment */ label;`
            ctx.diagnostic(diagnostic);
        } else {
            // e.g. `break label;` -> `break;`
            ctx.diagnostic_with_fix(diagnostic, |fixer| fixer.delete_range(delete_span));
        }
        return;
    }
}

fn is_breakable_statement(kind: AstKind) -> bool {
    match kind {
        kind if kind.is_iteration_statement() => true,
        AstKind::SwitchStatement(_) => true,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "A: break A;",
        "A: { if (a) break A; }",
        "A: { while (b) { break A; } }",
        "A: { switch (b) { case 0: break A; } }",
        "A: while (a) { while (b) { break; } break; }",
        "A: while (a) { while (b) { break A; } }",
        "A: while (a) { while (b) { continue A; } }",
        "A: while (a) { switch (b) { case 0: break A; } }",
        "A: while (a) { switch (b) { case 0: continue A; } }",
        "A: switch (a) { case 0: while (b) { break A; } }",
        "A: switch (a) { case 0: switch (b) { case 0: break A; } }",
        "A: for (;;) { while (b) { break A; } }",
        "A: do { switch (b) { case 0: break A; break; } } while (a);",
        "A: for (a in obj) { while (b) { break A; } }",
        "A: for (a of ary) { switch (b) { case 0: break A; } }", // { "ecmaVersion": 6 }
    ];

    let fail = vec![
        "A: while (a) break A;",
        "A: while (a) { B: { continue A; } }",
        "X: while (x) { A: while (a) { B: { break A; break B; continue X; } } }",
        "A: do { break A; } while (a);",
        "A: for (;;) { break A; }",
        "A: for (a in obj) { break A; }",
        "A: for (a of ary) { break A; }", // { "ecmaVersion": 6 },
        "A: switch (a) { case 0: break A; }",
        "X: while (x) { A: switch (a) { case 0: break A; } }",
        "X: switch (a) { case 0: A: while (b) break A; }",
        "                
        A: while (true) {
            break A;
            while (true) {
                break A;
            }
        }
        ",
        "A: while(true) { /*comment*/break A; }",
        "A: while(true) { break/**/ A; }",
        "A: while(true) { continue /**/ A; }",
        "A: while(true) { break /**/A; }",
        "A: while(true) { continue/**/A; }",
        "A: while(true) { continue A/*comment*/; }",
        "A: while(true) { break A//comment
			 }",
        "A: while(true) { break A/*comment*/
			foo() }",
    ];

    let fix = vec![
        ("A: while (a) break A;", "A: while (a) break;", None),
        ("A: while (a) { B: { continue A; } }", "A: while (a) { B: { continue; } }", None),
        (
            "X: while (x) { A: while (a) { B: { break A; break B; continue X; } } }",
            "X: while (x) { A: while (a) { B: { break; break B; continue X; } } }",
            None,
        ),
        ("A: do { break A; } while (a);", "A: do { break; } while (a);", None),
        ("A: for (;;) { break A; }", "A: for (;;) { break; }", None),
        ("A: for (a in obj) { break A; }", "A: for (a in obj) { break; }", None),
        ("A: for (a of ary) { break A; }", "A: for (a of ary) { break; }", None),
        ("A: switch (a) { case 0: break A; }", "A: switch (a) { case 0: break; }", None),
        (
            "X: while (x) { A: switch (a) { case 0: break A; } }",
            "X: while (x) { A: switch (a) { case 0: break; } }",
            None,
        ),
        (
            "X: switch (a) { case 0: A: while (b) break A; }",
            "X: switch (a) { case 0: A: while (b) break; }",
            None,
        ),
        (
            "
            A: while (true) {
                break A;
                while (true) {
                    break A;
                }
            }
            ",
            "
            A: while (true) {
                break;
                while (true) {
                    break A;
                }
            }
            ",
            None,
        ),
        ("A: while(true) { /*comment*/break A; }", "A: while(true) { /*comment*/break; }", None),
        (
            "A: while(true) { continue A/*comment*/; }",
            "A: while(true) { continue/*comment*/; }",
            None,
        ),
        (
            "A: while(true) { break A//comment
			 }",
            "A: while(true) { break//comment
			 }",
            None,
        ),
        (
            "A: while(true) { break A/*comment*/
			foo() }",
            "A: while(true) { break/*comment*/
			foo() }",
            None,
        ),
        // Do not fix if a comment sits between break/continue and label
        (
            r"A: while(true) { break /*comment*/ A; }",
            r"A: while(true) { break /*comment*/ A; }",
            None,
        ),
        (
            r"A: while(true) { continue /*comment*/ A; }",
            r"A: while(true) { continue /*comment*/ A; }",
            None,
        ),
    ];
    Tester::new(NoExtraLabel::NAME, NoExtraLabel::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
