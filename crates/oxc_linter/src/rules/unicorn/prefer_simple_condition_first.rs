use crate::{AstNode, context::LintContext, fixer::RuleFixer, rule::Rule};
use oxc_ast::{
    AstKind,
    ast::{Expression, LogicalOperator, UnaryOperator},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::GetSpan;
use oxc_span::Span;

fn prefer_simple_condition_first_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Complex conditions should come after simple conditions in logical expressions",
    )
    .with_help("Swap the conditions so the simpler one comes first")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferSimpleConditionFirst;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When writing multiple conditions in a logical expression (&&, ||), simple conditions should come first.
    ///
    /// ### Why is this bad?
    ///
    /// This can improve readability and performance,
    /// since simple checks like identifiers and strict equality comparisons are cheaper to evaluate and can short-circuit before expensive operations.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    ///if (check(foo) && bar);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    ///  if(bar && check(foo));
    /// ```
    PreferSimpleConditionFirst,
    unicorn,
    style,
    fix,
    version = "next",
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckLevel {
    /// Simple expression: identifier, literal, comparison, or negation of simple
    Simple,
    /// Has side effects or can throw — not safe to reorder
    SafeOrder,
    /// Complex but safe to reorder (e.g., ternary expression)
    Pending,
}

impl Rule for PreferSimpleConditionFirst {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let condition = match node.kind() {
            AstKind::IfStatement(stmt) => &stmt.test,
            AstKind::WhileStatement(stmt) => &stmt.test,
            AstKind::DoWhileStatement(stmt) => &stmt.test,
            AstKind::ForStatement(stmt) => {
                let Some(test) = &stmt.test else { return };
                test
            }
            _ => return,
        };

        let condition = condition.get_inner_expression();
        let Expression::LogicalExpression(logical_expr) = condition else {
            return;
        };

        if logical_expr.operator == LogicalOperator::Coalesce {
            return;
        }

        let left_level = check_logical_condition(&logical_expr.left);

        if left_level == CheckLevel::Simple || left_level == CheckLevel::SafeOrder {
            return;
        }

        let right_level = check_logical_condition(&logical_expr.right);
        if right_level == CheckLevel::Simple {
            let left_span = logical_expr.left.span();
            let right_span = logical_expr.right.span();
            ctx.diagnostic_with_fix(
                prefer_simple_condition_first_diagnostic(right_span),
                |fixer: RuleFixer<'_, 'a>| {
                    let left_source = ctx.source_range(left_span);
                    let right_source = ctx.source_range(right_span);
                    let operator = logical_expr.operator.as_str();

                    let replacement = format!("{right_source} {operator} {left_source}");
                    fixer.replace(logical_expr.span, replacement)
                },
            );
        }
    }
}

/// Determine the complexity level of a condition expression.
fn check_logical_condition(expr: &Expression<'_>) -> CheckLevel {
    let expr = expr.get_inner_expression();
    match expr {
        Expression::Identifier(_)
        | Expression::BooleanLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_)
        | Expression::ThisExpression(_) => CheckLevel::Simple,

        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::LogicalNot => {
            let inner_level = check_logical_condition(&unary.argument);
            if inner_level == CheckLevel::Simple {
                CheckLevel::Simple
            } else if !is_safe_order(&unary.argument) {
                CheckLevel::SafeOrder
            } else {
                CheckLevel::Pending
            }
        }

        _ => {
            if !is_safe_order(expr) {
                CheckLevel::SafeOrder
            } else {
                CheckLevel::Pending
            }
        }
    }
}

/// Check if reordering the expression is safe (no side effects, no throws).
fn is_safe_order(expr: &Expression<'_>) -> bool {
    let expr = expr.get_inner_expression();

    match expr {
        // side effects or can throw
        Expression::AssignmentExpression(_)
        | Expression::UpdateExpression(_)
        | Expression::CallExpression(_)
        | Expression::NewExpression(_)
        | Expression::AwaitExpression(_)
        | Expression::YieldExpression(_)
        | Expression::ImportExpression(_)
        | Expression::TaggedTemplateExpression(_) => false,

        // Member access can throw if object is nullish
        Expression::StaticMemberExpression(_) => false,
        Expression::ComputedMemberExpression(_) => false,

        //  Optional chaining still produces values and can trigger getters
        Expression::ChainExpression(_) => false,

        // BinaryExpression: conservatively unsafe (type coercion, in/instanceof throw)
        Expression::BinaryExpression(_) => false,

        Expression::UnaryExpression(unary) => is_safe_order(&unary.argument),
        Expression::LogicalExpression(logical) => {
            is_safe_order(&logical.left) && is_safe_order(&logical.right)
        }

        _ => true,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "if (bar && check(foo));",
        "if (bar && foo.baz);",
        "if (bar && foo.bar.baz === 1);",
        "if (a && b);",
        "if (foo === 1 && bar === 2);",
        r#"if (a !== "hello" && b);"#,
        "if (a.b && c.d);",
        "if (foo() && bar());",
        "if (a.b.c && d.e.f);",
        "if (foo);",
        "if (!foo && bar);",
        "if (bar && !foo);",
        "if (!a || !b);",
        "if (bar || foo());",
        "if (x === -1 && y);",
        "if ((state.ready = true) && ok);",
        "if (++counter && ok);",
        "if ((foo + bar) && ready);",
        "if (check(foo) && bar);",
        "if (new Foo() && bar);",
        "while (foo() && bar) {}",
        "for (; foo() && bar; ) {}",
        "(foo() && bar) ? 1 : 0",
        "if (a() && b() && c);",
        "if (object.deep.value && ok);",
        "const x = object.deep.value || ok;",
        "if (tag`x` && ok);",
        "async function f() { if ((await foo) && bar); }",
        "function* f() { if ((yield foo) && bar); }",
        r#"if (import("foo") && bar);"#,
        "if ((a + (b = c)) && ok);",
        "if (-(++x) && ok);",
        "if (foo.bar.baz === 1 && bar === 2);",
        "const x = a.b.c && d",
        "const x = foo.bar ?? baz",
        "if (a.b && c);",
        "if (a?.b && c);",
        "if (a[b] && c);",
        "if (foo in bar && baz);",
        "if (foo instanceof bar && baz);",
        "const x = foo() || bar",
        "const x = a.b && c",
        "if (foo.bar() && baz === 1);",
        "if ((a.b || c) && d);",
        "if ((a.b) && c);",
        "if (a.b && !c);",
        "if (a.b && x === -1);",
    ];

    let fail =
        vec![("if ((foo ? bar : baz) && ready);"), ("if ((foo ? bar : baz) /* keep */ && ready);")];

    let fix = vec![("if ((foo ? bar : baz) && ready);", "if (ready && (foo ? bar : baz));")];
    Tester::new(PreferSimpleConditionFirst::NAME, PreferSimpleConditionFirst::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
