use oxc_ast::{
    ast::{AssignmentExpression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_cond_assign_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Expected a conditional expression and instead saw an assignment")
        .with_help("Consider wrapping the assignment in additional parentheses")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoCondAssign {
    config: NoCondAssignConfig,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
enum NoCondAssignConfig {
    #[default]
    ExceptParens,
    Always,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow assignment operators in conditional expressions
    ///
    /// ### Why is this bad?
    ///
    /// In conditional statements, it is very easy to mistype a comparison
    /// operator (such as `==`) as an assignment operator (such as `=`).
    ///
    /// There are valid reasons to use assignment operators in conditional
    /// statements. However, it can be difficult to tell whether a specific
    /// assignment was intentional.
    ///
    /// ### Example
    ///
    /// ```js
    /// // Check the user's job title
    /// if (user.jobTitle = "manager") {
    ///     // user.jobTitle is now incorrect
    /// }
    /// ```
    NoCondAssign,
    correctness
);

impl Rule for NoCondAssign {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config = value.get(0).and_then(serde_json::Value::as_str).map_or_else(
            NoCondAssignConfig::default,
            |value| match value {
                "always" => NoCondAssignConfig::Always,
                _ => NoCondAssignConfig::ExceptParens,
            },
        );
        Self { config }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::IfStatement(stmt) => self.check_expression(ctx, &stmt.test),
            AstKind::WhileStatement(stmt) => self.check_expression(ctx, &stmt.test),
            AstKind::DoWhileStatement(stmt) => self.check_expression(ctx, &stmt.test),
            AstKind::ForStatement(stmt) => {
                if let Some(expr) = &stmt.test {
                    self.check_expression(ctx, expr);
                }
            }
            AstKind::ConditionalExpression(expr) => {
                self.check_expression(ctx, expr.test.get_inner_expression());
            }
            AstKind::AssignmentExpression(expr) if self.config == NoCondAssignConfig::Always => {
                let mut spans = vec![];
                for ancestor in ctx.nodes().iter_parents(node.id()).skip(1) {
                    match ancestor.kind() {
                        AstKind::IfStatement(if_stmt) => {
                            spans.push(if_stmt.test.span());
                        }
                        AstKind::WhileStatement(while_stmt) => {
                            spans.push(while_stmt.test.span());
                        }
                        AstKind::DoWhileStatement(do_while_stmt) => {
                            spans.push(do_while_stmt.test.span());
                        }
                        AstKind::ForStatement(for_stmt) => {
                            if let Some(test) = &for_stmt.test {
                                spans.push(test.span());
                            }
                            if let Some(update) = &for_stmt.update {
                                spans.push(update.span());
                            }
                            if let Some(update) = &for_stmt.update {
                                spans.push(update.span());
                            }
                        }
                        AstKind::ConditionalExpression(cond_expr) => {
                            spans.push(cond_expr.span());
                        }
                        AstKind::Function(_)
                        | AstKind::ArrowFunctionExpression(_)
                        | AstKind::Program(_)
                        | AstKind::BlockStatement(_) => {
                            break;
                        }
                        _ => {}
                    };
                }

                // Only report the diagnostic if the assignment is in a span where it should not be.
                // For example, report `if (a = b) { ...}`, not `if (...) { a = b }`
                if spans.iter().any(|span| span.contains_inclusive(node.span())) {
                    Self::emit_diagnostic(ctx, expr);
                }
            }
            _ => {}
        }
    }
}

impl NoCondAssign {
    #[allow(clippy::cast_possible_truncation)]
    fn emit_diagnostic(ctx: &LintContext<'_>, expr: &AssignmentExpression<'_>) {
        let mut operator_span = Span::new(expr.left.span().end, expr.right.span().start);
        let start =
            operator_span.source_text(ctx.source_text()).find(expr.operator.as_str()).unwrap_or(0)
                as u32;
        operator_span.start += start;
        operator_span.end = operator_span.start + expr.operator.as_str().len() as u32;

        ctx.diagnostic(no_cond_assign_diagnostic(operator_span));
    }

    fn check_expression(&self, ctx: &LintContext<'_>, expr: &Expression<'_>) {
        let mut expr = expr;
        if self.config == NoCondAssignConfig::Always {
            expr = expr.get_inner_expression();
        }
        if let Expression::AssignmentExpression(expr) = expr {
            Self::emit_diagnostic(ctx, expr);
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("var x = 0; if (x == 0) { var b = 1; }", None),
        ("var x = 0; if (x == 0) { var b = 1; }", Some(serde_json::json!(["always"]))),
        ("var x = 5; while (x < 5) { x = x + 1; }", None),
        ("if ((someNode = someNode.parentNode) !== null) { }", None),
        (
            "if ((someNode = someNode.parentNode) !== null) { }",
            Some(serde_json::json!(["except-parens"])),
        ),
        ("if ((a = b));", None),
        ("while ((a = b));", None),
        ("do {} while ((a = b));", None),
        ("for (;(a = b););", None),
        ("for (;;) {}", None),
        ("if (someNode || (someNode = parentNode)) { }", None),
        ("while (someNode || (someNode = parentNode)) { }", None),
        ("do { } while (someNode || (someNode = parentNode));", None),
        ("for (;someNode || (someNode = parentNode););", None),
        (
            "if ((function(node) { return node = parentNode; })(someNode)) { }",
            Some(serde_json::json!(["except-parens"])),
        ),
        (
            "if ((function(node) { return node = parentNode; })(someNode)) { }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "if ((node => node = parentNode)(someNode)) { }",
            Some(serde_json::json!(["except-parens"])),
        ),
        ("if ((node => node = parentNode)(someNode)) { }", Some(serde_json::json!(["always"]))),
        (
            "if (function(node) { return node = parentNode; }) { }",
            Some(serde_json::json!(["except-parens"])),
        ),
        (
            "if (function(node) { return node = parentNode; }) { }",
            Some(serde_json::json!(["always"])),
        ),
        ("x = 0;", Some(serde_json::json!(["always"]))),
        ("var x; var b = (x === 0) ? 1 : 0;", None),
        ("switch (foo) { case a = b: bar(); }", Some(serde_json::json!(["except-parens"]))),
        ("switch (foo) { case a = b: bar(); }", Some(serde_json::json!(["always"]))),
        ("switch (foo) { case baz + (a = b): bar(); }", Some(serde_json::json!(["always"]))),
        // not in condition
        ("if (obj.key) { (obj.key=false) }", Some(serde_json::json!(["always"]))),
        ("for (;;) { (obj.key=false) }", Some(serde_json::json!(["always"]))),
        ("while (obj.key) { (obj.key=false) }", Some(serde_json::json!(["always"]))),
        ("do { (obj.key=false) } while (obj.key)", Some(serde_json::json!(["always"]))),
        // https://github.com/oxc-project/oxc/issues/6656
        (
            "
            if (['a', 'b', 'c', 'd'].includes(value)) newValue = value;
            else newValue = 'default';
            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
            while(true) newValue = value;
            ",
            Some(serde_json::json!(["always"])),
        ),
        (
            "
            for(;;) newValue = value;
            ",
            Some(serde_json::json!(["always"])),
        ),
        ("for (; (typeof l === 'undefined' ? (l = 0) : l); i++) { }", None),
        ("for (x = 0;x<10;x++) { x = 0 }", None),
        ("for (x = 0;x<10;(x = x + 1)) { x = 0 }", None),
    ];

    let fail = vec![
        ("var x; if (x = 0) { var b = 1; }", None),
        ("var x; while (x = 0) { var b = 1; }", None),
        ("var x = 0, y; do { y = x; } while (x = x + 1);", None),
        ("var x; for(; x+=1 ;){};", None),
        ("var x; if ((x) = (0));", None),
        ("if (someNode || (someNode = parentNode)) { }", Some(serde_json::json!(["always"]))),
        ("while (someNode || (someNode = parentNode)) { }", Some(serde_json::json!(["always"]))),
        (
            "do { } while (someNode || (someNode = parentNode));",
            Some(serde_json::json!(["always"])),
        ),
        (
            "for (; (typeof l === 'undefined' ? (l = 0) : l); i++) { }",
            Some(serde_json::json!(["always"])),
        ),
        ("if (x = 0) { }", Some(serde_json::json!(["always"]))),
        ("while (x = 0) { }", Some(serde_json::json!(["always"]))),
        ("do { } while (x = x + 1);", Some(serde_json::json!(["always"]))),
        ("for(; x = y; ) { }", Some(serde_json::json!(["always"]))),
        ("if ((x = 0)) { }", Some(serde_json::json!(["always"]))),
        ("while ((x = 0)) { }", Some(serde_json::json!(["always"]))),
        ("do { } while ((x = x + 1));", Some(serde_json::json!(["always"]))),
        ("for(; (x = y); ) { }", Some(serde_json::json!(["always"]))),
        ("var x; var b = (x = 0) ? 1 : 0;", None),
        ("var x; var b = x && (y = 0) ? 1 : 0;", Some(serde_json::json!(["always"]))),
        ("(((3496.29)).bkufyydt = 2e308) ? foo : bar;", None),
    ];

    Tester::new(NoCondAssign::NAME, pass, fail).test_and_snapshot();
}
