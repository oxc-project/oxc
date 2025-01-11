use oxc_ast::{
    ast::{LabelIdentifier, Statement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_labels_diagnostic(message: &'static str, label_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(message).with_label(label_span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLabels {
    allow_loop: bool,
    allow_switch: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow labeled statements.
    ///
    /// ### Why is this bad?
    ///
    /// Labeled statements in JavaScript are used in conjunction with `break` and `continue` to control flow around multiple loops. For example:
    /// ```js
    /// outer:
    ///     while (true) {
    ///         while (true) {
    ///            break outer;
    ///         }
    ///     }
    /// ```
    /// The `break outer` statement ensures that this code will not result in an infinite loop because control is returned to the next statement after the `outer` label was applied. If this statement was changed to be just `break`, control would flow back to the outer `while` statement and an infinite loop would result.
    /// While convenient in some cases, labels tend to be used only rarely and are frowned upon by some as a remedial form of flow control that is more error prone and harder to understand.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// label:
    ///     while(true) {
    ///         // ...
    ///     }
    ///
    /// label:
    ///     while(true) {
    ///         break label;
    ///     }
    ///
    /// label:
    ///     while(true) {
    ///         continue label;
    ///     }
    ///
    /// label:
    ///     switch (a) {
    ///     case 0:
    ///         break label;
    ///     }
    ///
    /// label:
    ///     {
    ///         break label;
    ///     }
    ///
    /// label:
    ///     if (a) {
    ///         break label;
    ///     }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var f = {
    ///     label: "foo"
    /// };
    ///
    /// while (true) {
    ///     break;
    /// }
    ///
    /// while (true) {
    ///     continue;
    /// }
    /// ```
    ///
    /// ### Options
    ///
    /// The options allow labels with loop or switch statements:
    /// * `"allowLoop"` (`boolean`, default is `false`) - If this option was set `true`, this rule ignores labels which are sticking to loop statements.
    /// * `"allowSwitch"` (`boolean`, default is `false`) - If this option was set `true`, this rule ignores labels which are sticking to switch statements.
    ///
    /// Actually labeled statements in JavaScript can be used with other than loop and switch statements.
    /// However, this way is ultra rare, not well-known, so this would be confusing developers.
    ///
    /// #### allowLoop
    ///
    /// Examples of **correct** code for the `{ "allowLoop": true }` option:
    /// ```js
    /// label:
    ///     while (true) {
    ///         break label;
    ///     }
    /// ```
    ///
    /// #### allowSwitch
    ///
    /// Examples of **correct** code for the `{ "allowSwitch": true }` option:
    /// ```js
    /// label:
    ///     switch (a) {
    ///         case 0:
    ///             break label;
    ///     }
    /// ```
    NoLabels,
    eslint,
    style,
);

impl Rule for NoLabels {
    fn from_configuration(value: serde_json::Value) -> Self {
        let allow_loop = value
            .get(0)
            .and_then(|config| config.get("allowLoop"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let allow_switch = value
            .get(0)
            .and_then(|config| config.get("allowSwitch"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { allow_loop, allow_switch }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::LabeledStatement(labeled_stmt) = node.kind() {
            if !self.is_allowed(&labeled_stmt.body) {
                let label_span = labeled_stmt.label.span;
                ctx.diagnostic(no_labels_diagnostic(
                    "Labeled statement is not allowed",
                    label_span,
                ));
            }
        }

        if let AstKind::BreakStatement(break_stmt) = node.kind() {
            let Some(label) = &break_stmt.label else { return };

            if !self.is_allowed_in_break_or_continue(label, node.id(), ctx) {
                ctx.diagnostic(no_labels_diagnostic(
                    "Label in break statement is not allowed",
                    label.span,
                ));
            }
        }

        if let AstKind::ContinueStatement(cont_stmt) = node.kind() {
            let Some(label) = &cont_stmt.label else { return };

            if !self.is_allowed_in_break_or_continue(label, node.id(), ctx) {
                ctx.diagnostic(no_labels_diagnostic(
                    "Label in continue statement is not allowed",
                    label.span,
                ));
            }
        }
    }
}

impl NoLabels {
    fn is_allowed(&self, stmt: &Statement) -> bool {
        match stmt {
            stmt if stmt.is_iteration_statement() => self.allow_loop,
            Statement::SwitchStatement(_) => self.allow_switch,
            _ => false,
        }
    }

    /// Whether the `label` in break/continue statement is allowed.
    fn is_allowed_in_break_or_continue<'a>(
        &self,
        label: &LabelIdentifier<'a>,
        stmt_node_id: NodeId,
        ctx: &LintContext<'a>,
    ) -> bool {
        let nodes = ctx.nodes();
        for ancestor_kind in nodes.ancestor_kinds(stmt_node_id) {
            if let AstKind::LabeledStatement(labeled_stmt) = ancestor_kind {
                if label.name == labeled_stmt.label.name {
                    return self.is_allowed(&labeled_stmt.body);
                }
            }
        }
        false
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var f = { label: foo ()}", None),
        ("while (true) {}", None),
        ("while (true) { break; }", None),
        ("while (true) { continue; }", None),
        ("A: while (a) { break A; }", Some(serde_json::json!([{ "allowLoop": true }]))),
        (
            "A: do { if (b) { break A; } } while (a);",
            Some(serde_json::json!([{ "allowLoop": true }])),
        ),
        (
            "A: for (var a in obj) { for (;;) { switch (a) { case 0: continue A; } } }",
            Some(serde_json::json!([{ "allowLoop": true }])),
        ),
        ("A: switch (a) { case 0: break A; }", Some(serde_json::json!([{ "allowSwitch": true }]))),
    ];

    let fail = vec![
        ("label: while(true) {}", None),
        ("label: while (true) { break label; }", None),
        ("label: while (true) { continue label; }", None),
        ("A: var foo = 0;", None),
        ("A: break A;", None),
        ("A: { if (foo()) { break A; } bar(); };", None),
        ("A: if (a) { if (foo()) { break A; } bar(); };", None),
        ("A: switch (a) { case 0: break A; default: break; };", None),
        ("A: switch (a) { case 0: B: { break A; } default: break; };", None),
        ("A: var foo = 0;", Some(serde_json::json!([{ "allowLoop": true }]))),
        ("A: break A;", Some(serde_json::json!([{ "allowLoop": true }]))),
        (
            "A: { if (foo()) { break A; } bar(); };",
            Some(serde_json::json!([{ "allowLoop": true }])),
        ),
        (
            "A: if (a) { if (foo()) { break A; } bar(); };",
            Some(serde_json::json!([{ "allowLoop": true }])),
        ),
        (
            "A: switch (a) { case 0: break A; default: break; };",
            Some(serde_json::json!([{ "allowLoop": true }])),
        ),
        ("A: var foo = 0;", Some(serde_json::json!([{ "allowSwitch": true }]))),
        ("A: break A;", Some(serde_json::json!([{ "allowSwitch": true }]))),
        (
            "A: { if (foo()) { break A; } bar(); };",
            Some(serde_json::json!([{ "allowSwitch": true }])),
        ),
        (
            "A: if (a) { if (foo()) { break A; } bar(); };",
            Some(serde_json::json!([{ "allowSwitch": true }])),
        ),
        ("A: while (a) { break A; }", Some(serde_json::json!([{ "allowSwitch": true }]))),
        (
            "A: do { if (b) { break A; } } while (a);",
            Some(serde_json::json!([{ "allowSwitch": true }])),
        ),
        (
            "A: for (var a in obj) { for (;;) { switch (a) { case 0: break A; } } }",
            Some(serde_json::json!([{ "allowSwitch": true }])),
        ),
    ];

    Tester::new(NoLabels::NAME, NoLabels::PLUGIN, pass, fail).test_and_snapshot();
}
