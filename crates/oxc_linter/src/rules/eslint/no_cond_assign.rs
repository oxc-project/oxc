use oxc_ast::{
    ast::{
        ConditionalExpression, DoWhileStatement, Expression, ForStatement, IfStatement,
        WhileStatement,
    },
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-cond-assign): Expected a conditional expression and instead saw an assignment")]
#[diagnostic(severity(warning), help("Consider wrapping the assignment in additional parentheses"))]
struct NoCondAssignDiagnostic(#[label] pub Span);

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
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
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
            AstKind::IfStatement(stmt) if self.is_assignment_expression(&stmt.test) => {
                ctx.diagnostic(NoCondAssignDiagnostic(stmt.test.span()));
            }
            AstKind::WhileStatement(stmt) if self.is_assignment_expression(&stmt.test) => {
                ctx.diagnostic(NoCondAssignDiagnostic(stmt.test.span()));
            }
            AstKind::DoWhileStatement(stmt) if self.is_assignment_expression(&stmt.test) => {
                ctx.diagnostic(NoCondAssignDiagnostic(stmt.test.span()));
            }
            AstKind::ForStatement(stmt) => {
                if let Some(expr) = &stmt.test {
                    if self.is_assignment_expression(expr) {
                        ctx.diagnostic(NoCondAssignDiagnostic(expr.span()));
                    }
                }
            }
            AstKind::ConditionalExpression(expr) if self.is_assignment_expression(expr.test.get_inner_expression()) => {
                ctx.diagnostic(NoCondAssignDiagnostic(expr.test.span()));
            }
            AstKind::AssignmentExpression(_) if self.config == NoCondAssignConfig::Always => {
                for node_id in ctx.nodes().ancestors(node.id()).skip(1) {
                    match ctx.nodes().kind(node_id) {
                        AstKind::IfStatement(IfStatement { test, .. })
                        | AstKind::WhileStatement(WhileStatement { test, .. })
                        | AstKind::DoWhileStatement(DoWhileStatement { test, .. })
                        | AstKind::ForStatement(ForStatement { test: Some(test), .. })
                        | AstKind::ConditionalExpression(ConditionalExpression { test, .. }) => {
                            ctx.diagnostic(NoCondAssignDiagnostic(test.span()));
                            return
                        }
                        AstKind::Function(_) | AstKind::ArrowExpression(_)| AstKind::Program(_) => break,
                        _ => {},
                    }
                }
            }
            _ => {}
        }
    }
}

impl NoCondAssign {
    fn is_assignment_expression(&self, expr: &Expression<'_>) -> bool {
        let mut expr = expr;
        if self.config == NoCondAssignConfig::Always {
            expr = expr.get_inner_expression();
        }
        matches!(expr, Expression::AssignmentExpression(_))
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
