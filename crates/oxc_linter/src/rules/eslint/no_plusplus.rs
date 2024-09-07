use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_plusplus_diagnostic(span: Span, operator: &str) -> OxcDiagnostic {
    let diagnostic =
        OxcDiagnostic::warn(format!("Unary operator '{}' used.", operator)).with_label(span);

    if operator == "++" {
        return diagnostic.with_help("Use the assignment operator `+=` instead.");
    } else if operator == "--" {
        return diagnostic.with_help("Use the assignment operator `-=` instead.");
    }

    diagnostic
}

#[derive(Debug, Default, Clone)]
pub struct NoPlusplus {
    /// Whether to allow `++` and `--` in for loop afterthoughts.
    allow_for_loop_afterthoughts: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the unary operators `++`` and `--`.
    ///
    /// ### Why is this bad?
    ///
    /// Because the unary `++` and `--` operators are subject to automatic semicolon insertion, differences in whitespace
    /// can change the semantics of source code. For example, these two code blocks are not equivalent:
    ///
    /// ```js
    /// var i = 10;
    /// var j = 20;
    ///
    /// i ++
    /// j
    /// // => i = 11, j = 20
    /// ```
    ///
    /// ```js
    /// var i = 10;
    /// var j = 20;
    ///
    /// i
    /// ++
    /// j
    /// // => i = 10, j = 21
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var x = 0; x++;
    /// var y = 0; y--;
    /// for (i = 0; i < l; i++) {
    ///     doSomething(i);
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// var x = 0; x += 1;
    /// var y = 0; y -= 1;
    /// for (i = 0; i < l; i += 1) {
    ///    doSomething(i);
    /// }
    /// ```
    NoPlusplus,
    restriction,
);

impl Rule for NoPlusplus {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_for_loop_afterthoughts: obj
                .and_then(|v| v.get("allowForLoopAfterthoughts"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::UpdateExpression(expr) = node.kind() else {
            return;
        };

        if self.allow_for_loop_afterthoughts && is_for_loop_afterthought(node, ctx) {
            return;
        }

        ctx.diagnostic(no_plusplus_diagnostic(expr.span, expr.operator.as_str()));
    }
}

/// Whether the given AST node is a ++ or -- inside of a for-loop update.
fn is_for_statement_update(node: &AstNode, ctx: &LintContext) -> bool {
    let Some(parent) = ctx.nodes().parent_node(node.id()) else {
        return false;
    };
    let AstKind::ForStatement(for_stmt) = parent.kind() else {
        return false;
    };

    for_stmt.update.as_ref().is_some_and(|update| is_eq_node_expr(node, update))
}

/// Checks if the given node is equivalent to the given expression (i.e., they have the same span).
fn is_eq_node_expr(node: &AstNode, expr: &Expression) -> bool {
    // TODO: This logic should be moved to somewhere more general and shared across rules and expanded
    // to cover all expressions and node types
    let node_span = match node.kind() {
        AstKind::UpdateExpression(expr) => expr.span,
        AstKind::SequenceExpression(expr) => expr.span,
        _ => return false,
    };
    let expr_span = match expr {
        Expression::UpdateExpression(expr) => expr.span,
        Expression::SequenceExpression(expr) => expr.span,
        _ => return false,
    };
    node_span == expr_span
}

/// Determines whether the given node is considered to be a for loop "afterthought" by the logic of this rule.
/// In particular, it returns `true` if the given node is either:
///   - The update node of a `ForStatement`: for (;; i++) {}
///   - An operand of a sequence expression that is the update node: for (;; foo(), i++) {}
///   - An operand of a sequence expression that is child of another sequence expression, etc.,
///     up to the sequence expression that is the update node: for (;; foo(), (bar(), (baz(), i++))) {}
fn is_for_loop_afterthought(node: &AstNode, ctx: &LintContext) -> bool {
    if let Some(parent) = ctx.nodes().parent_node(node.id()) {
        match parent.kind() {
            AstKind::SequenceExpression(_) => return is_for_loop_afterthought(parent, ctx),
            AstKind::ParenthesizedExpression(_) => return is_for_loop_afterthought(parent, ctx),
            _ => (),
        }
    }

    is_for_statement_update(node, ctx)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var foo = 0; foo=+1;", None),
        ("var foo = 0; foo+=1;", None),
        ("var foo = 0; foo-=1;", None),
        ("var foo = 0; foo=+1;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { console.log(i); }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (var i = 0, j = i + 1; j < example.length; i++, j++) {}",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (;; i--, foo());", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; foo(), --i);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo(), ++i, bar);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; i++, (++j, k--));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; foo(), (bar(), i++), baz());",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; (--i, j += 2), bar = j + 1);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        (
            "for (;; a, (i--, (b, ++j, c)), d);",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    let fail = vec![
        ("var foo = 0; foo++;", None),
        ("var foo = 0; foo--;", None),
        ("var foo = 0; --foo;", None),
        ("var foo = 0; ++foo;", None),
        ("for (i = 0; i < l; i++) { console.log(i); }", None),
        ("for (i = 0; i < l; foo, i++) { console.log(i); }", None),
        ("var foo = 0; foo++;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (i = 0; i < l; i++) { v++; }",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
        ("for (i++;;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;--i;);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;;) ++i;", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i = j++);", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        ("for (;; i++, f(--j));", Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }]))),
        (
            "for (;; foo + (i++, bar));",
            Some(serde_json::json!([{ "allowForLoopAfterthoughts": true }])),
        ),
    ];

    Tester::new(NoPlusplus::NAME, pass, fail).test_and_snapshot();
}
