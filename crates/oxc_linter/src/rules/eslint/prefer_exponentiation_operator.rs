use oxc_ast::{AstKind, ast::Expression, match_member_expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::precedence::{GetPrecedence, Precedence};

use crate::{
    AstNode, ast_util::is_method_call, context::LintContext, globals::GLOBAL_OBJECT_NAMES,
    rule::Rule, utils::get_precedence,
};

#[derive(Debug, Default, Clone)]
pub struct PreferExponentiationOperator;

fn prefer_exponentian_operator_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer `**` over `Math.pow`.")
        .with_help("Replace `Math.pow(a, b)` with `a ** b`.")
        .with_label(span)
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of Math.pow in favor of the ** operator
    ///
    /// ### Why is this bad?
    ///
    /// Introduced in ES2016, the infix exponentiation operator ** is an alternative for the
    /// standard Math.pow function. Infix notation is considered to be more readable and thus more
    /// preferable than the function notation.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// Math.pow(a, b)
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// a ** b
    /// ```
    PreferExponentiationOperator,
    eslint,
    style,
    fix,
);

impl Rule for PreferExponentiationOperator {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        if !is_method_call(call_expr, None, Some(&["pow"]), Some(2), Some(2)) {
            return;
        }

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

        let member_expor_obj = member_expr.object();

        match member_expor_obj {
            Expression::Identifier(ident) => {
                if ident.name.as_str() != "Math" || !ctx.is_reference_to_global_variable(ident) {
                    return;
                }
            }
            match_member_expression!(Expression) => {
                let member_expr = member_expor_obj.to_member_expression();
                let Some(static_prop_name) = member_expr.static_property_name() else {
                    return;
                };
                if static_prop_name != "Math" {
                    return;
                }

                if let Expression::Identifier(ident) = member_expr.object().without_parentheses()
                    && GLOBAL_OBJECT_NAMES.contains(&ident.name.as_str())
                    && ctx.is_reference_to_global_variable(ident)
                {
                } else {
                    return;
                }
            }
            _ => return,
        }

        ctx.diagnostic_with_fix(prefer_exponentian_operator_diagnostic(call_expr.span), |fixer| {
            if let AstKind::CallExpression(call_expr) = node.kind()
                && !ctx.has_comments_between(call_expr.span)
                && call_expr.arguments.len() == 2
                && let Some(base) = call_expr.arguments[0].as_expression()
                && let Some(exponent) = call_expr.arguments[1].as_expression()
            {
                let base_text = if does_base_need_parens(base) {
                    format!("({})", base.span().source_text(ctx.source_text()))
                } else {
                    base.span().source_text(ctx.source_text()).to_string()
                };
                let exponent_text =
                    get_exponent_text(fixer.source_range(exponent.span()), exponent);

                let replacement = format!("{base_text} ** {exponent_text}");

                // Check if we need to wrap the entire expression in parentheses based on parent context
                let replacement = if needs_parens_for_parent(node, ctx) {
                    format!("({replacement})")
                } else {
                    replacement
                };

                fixer.replace(call_expr.span, replacement)
            } else {
                fixer.noop()
            }
        });
    }
}

fn does_base_need_parens(expr: &Expression) -> bool {
    let expr = expr.without_parentheses();
    if matches!(expr, Expression::UnaryExpression(_) | Expression::AwaitExpression(_)) {
        return true;
    }
    if let Some(prec) = get_precedence(expr) {
        return prec <= Precedence::Exponentiation;
    }
    false
}

/// Determines if the exponent expression needs parentheses when used in `base ** exponent`.
///
/// Parentheses are needed only when the exponent has lower precedence than exponentiation.
/// Since `**` is right-associative, `a ** b ** c` equals `a ** (b ** c)`,
/// so `Math.pow(a, b ** c)` can safely become `a ** b ** c`.
fn does_exponent_need_parens(expr: &Expression) -> bool {
    let expr = expr.without_parentheses();

    if let Some(prec) = get_precedence(expr) {
        return prec < Precedence::Exponentiation;
    }

    false
}

/// Gets the text for the exponent, adding parentheses if needed.
fn get_exponent_text(source_text: &str, expr: &Expression) -> String {
    if does_exponent_need_parens(expr) {
        format!("({source_text})")
    } else {
        source_text.to_string()
    }
}

fn needs_parens_for_parent(node: &AstNode, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());

    match parent.kind() {
        AstKind::BinaryExpression(bin_expr) => {
            if bin_expr.operator == oxc_ast::ast::BinaryOperator::Exponential {
                let AstKind::CallExpression(call_expr) = node.kind() else {
                    return true;
                };
                bin_expr.right.span() != call_expr.span
            } else {
                bin_expr.precedence() >= Precedence::Exponentiation
            }
        }
        AstKind::UnaryExpression(_)
        | AstKind::AwaitExpression(_)
        | AstKind::TSAsExpression(_)
        | AstKind::TSSatisfiesExpression(_) => true,
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Object.pow(a, b)",
        "Math.max(a, b)",
        "Math",
        "Math(a, b)",
        "pow",
        "pow(a, b)",
        "Math.pow",
        "Math.Pow(a, b)",
        "math.pow(a, b)",
        "foo.Math.pow(a, b)",
        "new Math.pow(a, b)",
        "Math[pow](a, b)",
        "globalThis.Object.pow(a, b)",
        "globalThis.Math.max(a, b)",
        // "/* globals Math:off*/ Math.pow(a, b)",
        "let Math; Math.pow(a, b);",
        "if (foo) { const Math = 1; Math.pow(a, b); }",
        "var x = function Math() { Math.pow(a, b); }",
        "function foo(Math) { Math.pow(a, b); }",
        "function foo() { Math.pow(a, b); var Math; }",
        "
			                var globalThis = bar;
			                globalThis.Math.pow(a, b)
			            ",
        "class C { #pow; foo() { Math.#pow(a, b); } }",
    ];

    let fail = vec![
        "globalThis.Math.pow(a, b)",
        "globalThis.Math['pow'](a, b)",
        "Math.pow(a, b) + Math.pow(c,
			 d)",
        "Math.pow(Math.pow(a, b), Math.pow(c, d))",
        "Math.pow(a, b)**Math.pow(c, d)",
        "Math.pow(a, b as any)",
        "Math.pow(a as any, b)",
        "Math.pow(a, b) as any",
        // With comments - no fix should be applied
        "Math.pow(a, b) + Math.pow(c, /* comment */ d)",
    ];

    let fix = vec![
        ("globalThis.Math.pow(a, b)", "a ** b"),
        ("globalThis.Math['pow'](a, b)", "a ** b"),
        // Nested Math.pow - only fixes outer call first (inner calls remain as arguments)
        ("Math.pow(Math.pow(a, b), Math.pow(c, d))", "Math.pow(a, b) ** Math.pow(c, d)"),
        // When Math.pow is the left operand of **, wrap in parens
        ("Math.pow(a, b)**Math.pow(c, d)", "(a ** b)**c ** d"),
        // TypeScript: exponent with type assertion needs parens
        ("Math.pow(a, b as any)", "a ** (b as any)"),
        // TypeScript: base with type assertion needs parens
        ("Math.pow(a as any, b)", "(a as any) ** b"),
        // TypeScript: entire expression cast needs parens around the exponentiation
        ("Math.pow(a, b) as any", "(a ** b) as any"),
        // Additional test cases
        ("Math.pow(a, b)", "a ** b"),
        ("Math.pow(2, 3)", "2 ** 3"),
        // Unary expressions in base need parens
        ("Math.pow(-a, b)", "(-a) ** b"),
        ("Math.pow(+a, b)", "(+a) ** b"),
        ("Math.pow(!a, b)", "(!a) ** b"),
        ("Math.pow(~a, b)", "(~a) ** b"),
        // Binary expressions with lower precedence need parens
        ("Math.pow(a + b, c)", "(a + b) ** c"),
        ("Math.pow(a * b, c)", "(a * b) ** c"),
        ("Math.pow(a, b + c)", "a ** (b + c)"),
        ("Math.pow(a, b * c)", "a ** (b * c)"),
        // Exponentiation in base needs parens (right-associativity)
        ("Math.pow(a ** b, c)", "(a ** b) ** c"),
        // Exponentiation in exponent doesn't need parens
        ("Math.pow(a, b ** c)", "a ** b ** c"),
        // Identifiers don't need parens
        ("Math.pow(foo, bar)", "foo ** bar"),
        // Member expressions don't need parens
        ("Math.pow(a.b, c.d)", "a.b ** c.d"),
        // Call expressions don't need parens
        ("Math.pow(f(), g())", "f() ** g()"),
    ];

    Tester::new(
        PreferExponentiationOperator::NAME,
        PreferExponentiationOperator::PLUGIN,
        pass,
        fail,
    )
    .expect_fix(fix)
    .test_and_snapshot();
}
