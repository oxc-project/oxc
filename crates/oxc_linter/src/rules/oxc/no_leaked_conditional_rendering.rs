use oxc_ast::AstKind;
use oxc_ast::ast::Expression;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::LogicalOperator;

use crate::{AstNode, context::LintContext, rule::Rule};

fn no_leaked_conditional_rendering_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Potential leaked value in JSX conditional rendering")
        .with_help("When using `&&` for conditional rendering, falsy values like `0` or `NaN` will be rendered as text. Use a ternary expression or `Boolean()` to ensure only boolean values are checked.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoLeakedConditionalRendering;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prevents problematic leaked values in JSX expressions when using `&&`
    /// for conditional rendering.
    ///
    /// ### Why is this bad?
    ///
    /// In React, when using `{value && <Component />}`, if `value` is a falsy
    /// number like `0` or `NaN`, it will be rendered as text in the output
    /// instead of rendering nothing. This is a common bug.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// {count && <Component />}
    /// {data.length && <List items={data} />}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// {count > 0 && <Component />}
    /// {Boolean(count) && <Component />}
    /// {count ? <Component /> : null}
    /// {!!count && <Component />}
    /// ```
    NoLeakedConditionalRendering,
    oxc,
    correctness,
    none
);

/// Check if an expression could be a non-boolean falsy value (number, string, etc.)
fn is_potentially_leaky(expr: &Expression<'_>) -> bool {
    match expr {
        // These are safe — they produce booleans
        Expression::BooleanLiteral(_) => false,
        Expression::UnaryExpression(unary)
            if unary.operator == oxc_syntax::operator::UnaryOperator::LogicalNot =>
        {
            false
        }
        Expression::CallExpression(call) => {
            // Boolean(x) is safe
            if let Expression::Identifier(callee) = &call.callee
                && callee.name == "Boolean"
            {
                return false;
            }
            true
        }
        Expression::BinaryExpression(bin) => {
            // Comparison operators produce booleans
            matches!(
                bin.operator,
                oxc_syntax::operator::BinaryOperator::Equality
                    | oxc_syntax::operator::BinaryOperator::Inequality
                    | oxc_syntax::operator::BinaryOperator::StrictEquality
                    | oxc_syntax::operator::BinaryOperator::StrictInequality
                    | oxc_syntax::operator::BinaryOperator::LessThan
                    | oxc_syntax::operator::BinaryOperator::LessEqualThan
                    | oxc_syntax::operator::BinaryOperator::GreaterThan
                    | oxc_syntax::operator::BinaryOperator::GreaterEqualThan
                    | oxc_syntax::operator::BinaryOperator::Instanceof
                    | oxc_syntax::operator::BinaryOperator::In
            )
            .not()
        }
        // Everything else (string/number literals, identifiers, member expressions, etc.)
        // could potentially leak non-boolean values
        _ => true,
    }
}

use std::ops::Not;

impl Rule for NoLeakedConditionalRendering {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical) = node.kind() else {
            return;
        };

        if logical.operator != LogicalOperator::And {
            return;
        }

        // Check if we're inside a JSX expression container
        let parent = ctx.nodes().parent_node(node.id());
        if !matches!(parent.kind(), AstKind::JSXExpressionContainer(_)) {
            return;
        }

        // The right side should be JSX for this to be conditional rendering
        let is_jsx_right =
            matches!(&logical.right, Expression::JSXElement(_) | Expression::JSXFragment(_));

        if !is_jsx_right {
            return;
        }

        if is_potentially_leaky(&logical.left) {
            ctx.diagnostic(no_leaked_conditional_rendering_diagnostic(logical.left.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<div>{count > 0 && <Component />}</div>"#,
        r#"<div>{Boolean(count) && <Component />}</div>"#,
        r#"<div>{!!count && <Component />}</div>"#,
        r#"<div>{count ? <Component /> : null}</div>"#,
        r#"<div>{true && <Component />}</div>"#,
    ];

    let fail =
        vec![r#"<div>{count && <Component />}</div>"#, r#"<div>{data.length && <List />}</div>"#];

    Tester::new(
        NoLeakedConditionalRendering::NAME,
        NoLeakedConditionalRendering::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path_extension("tsx")
    .test_and_snapshot();
}
