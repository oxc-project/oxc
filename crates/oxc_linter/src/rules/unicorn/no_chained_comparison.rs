use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::UnaryOperator;

use crate::{AstNode, ast_util::get_declaration_of_variable, context::LintContext, rule::Rule};

fn no_chained_comparison_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Comparison operators cannot be chained")
        .with_help(
            "The inner comparison evaluates to a boolean, which is then compared instead of the operands. Compare each pair of operands separately with `&&`.",
        )
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoChainedComparison;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow chained comparisons such as `a < b < c`.
    ///
    /// ### Why is this bad?
    ///
    /// Unlike in math (or Python), chained comparisons in JavaScript do not
    /// check a range. Comparison operators are binary and left-associative, so
    /// `a < b < c` parses as `(a < b) < c`. The first comparison evaluates to a
    /// boolean, which is then coerced to `0` or `1` and compared with `c`. This
    /// is almost always a bug.
    ///
    /// Use `&&` to compare each pair of operands separately.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// if (a < b < c) {}
    /// if (a === b === c) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// if (a < b && b < c) {}
    /// if (a === b && b === c) {}
    /// // Intentionally comparing two boolean results.
    /// if ((a > 0) === (b > 0)) {}
    /// ```
    NoChainedComparison,
    unicorn,
    correctness,
    version = "next",
    short_description = "Disallow chained comparisons such as `a < b < c`.",
);

impl Rule for NoChainedComparison {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(binary) = node.kind() else {
            return;
        };

        if !is_comparison_operator(binary) {
            return;
        }

        // One operand must itself be a comparison: `(a OP b) OP c` or `a OP (b OP c)`.
        let sibling = if is_comparison(&binary.left) {
            &binary.right
        } else if is_comparison(&binary.right) {
            &binary.left
        } else {
            return;
        };

        // For equality operators, comparing a comparison result against another boolean value
        // (`(a > 0) === (b > 0)`, `(a < b) === true`, `(a > 0) !== Boolean(b)`) is an intentional
        // boolean comparison, not a chained comparison.
        if binary.operator.is_equality() && is_boolean(sibling, ctx) {
            return;
        }

        ctx.diagnostic(no_chained_comparison_diagnostic(binary.span));
    }
}

fn is_comparison_operator(binary: &oxc_ast::ast::BinaryExpression) -> bool {
    binary.operator.is_compare() || binary.operator.is_equality()
}

/// A comparison is an ordering (`<`, `>`, `<=`, `>=`) or equality
/// (`==`, `!=`, `===`, `!==`) binary expression.
fn is_comparison(expr: &Expression) -> bool {
    matches!(expr.without_parentheses(), Expression::BinaryExpression(binary) if is_comparison_operator(binary))
}

/// Whether `expr` is known to evaluate to a boolean.
fn is_boolean<'a>(expr: &Expression<'a>, ctx: &LintContext<'a>) -> bool {
    match expr.without_parentheses() {
        Expression::BooleanLiteral(_) => true,
        Expression::BinaryExpression(binary) => {
            binary.operator.is_compare()
                || binary.operator.is_equality()
                || binary.operator.is_relational()
        }
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::LogicalNot | UnaryOperator::Delete)
        }
        Expression::LogicalExpression(logical) => {
            is_boolean(&logical.left, ctx) && is_boolean(&logical.right, ctx)
        }
        Expression::ConditionalExpression(conditional) => {
            is_boolean(&conditional.consequent, ctx) && is_boolean(&conditional.alternate, ctx)
        }
        // `Boolean(…)`
        Expression::CallExpression(call) => {
            call.arguments.len() == 1
                && matches!(&call.callee, Expression::Identifier(ident) if ident.name == "Boolean")
        }
        // A `const` binding whose initializer is itself a boolean.
        Expression::Identifier(ident) => {
            let Some(declaration) = get_declaration_of_variable(ident, ctx) else {
                return false;
            };
            let Some(declarator) = declaration.kind().as_variable_declarator() else {
                return false;
            };
            let is_const = matches!(
                ctx.nodes().parent_kind(declaration.id()),
                AstKind::VariableDeclaration(declaration) if declaration.kind.is_const()
            );
            is_const && declarator.init.as_ref().is_some_and(|init| is_boolean(init, ctx))
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "a < b",
        "a === b",
        "a < b && b < c",
        "a < b && b < c && c < d",
        "(a > 0) === (b > 0)",
        "(a > 0) !== (b > 0)",
        "(a > 0) == (b > 0)",
        "(a < b) === true",
        "(a < b) !== false",
        "(a < b) == true",
        "true === (a < b)",
        "a == b == true",
        "(a > 0) !== Boolean(b)",
        "(a < b) === !c",
        "(a < b) === (c ? true : false)",
        "const d = a > b; (a < b) === d;",
        "a < b || c < d",
        "!(a < b)",
        "foo(a < b, c)",
        "a < b ? c : d",
        "const x = a < b;",
        "f(a < b) === g(c < d)",
        "a instanceof b instanceof c",
        "a in b in c",
        "a + b + c",
        "(a < b) + c",
        "(a < b) & c",
        "typeof a < b",
        "a << b < c",
        "(a as number) < b", // {"parser": parsers.typescript}
    ];

    let fail = vec![
        "a < b < c",
        "a > b > c",
        "a <= b <= c",
        "a >= b >= c",
        "a < b <= c",
        "a < b > c",
        "a == b == c",
        "a === b === c",
        "a != b != c",
        "a !== b !== c",
        "a <= b >= c",
        "a === b !== c",
        "(a < b) < c",
        "(a == b) == c",
        "0 < x < 10",
        "min <= value <= max",
        "-1 < x < 1",
        "a < b.c < d",
        "a < b[c] < d",
        "a < b?.c < d",
        "a < b + c < d",
        "a < b === c",
        "a === b < c",
        "(a > 0) == 0",
        "(a > 0) === isEnabled",
        "a == b < c",
        "a === b > c",
        "a < f() < c",
        "a < (b = c) < d",
        "a < b++ < c",
        "a < /* comment */ b < c",
        "(a < b) < true",
        "a < b < false",
        "(a < b) < (c < d)",
        "a < b < c < d",
        "foo === a < b < c",
        "a < b < c === foo",
        "x & a < b < c",
        "a ?? b < c < d",
        "foo === (a < b < c)",
        "a && b < c < d",
        "a || b < c < d",
        "a < b < c ? x : y",
        "a < (b ?? 0) < c",
        "a < b < c",             // {"parser": parsers.typescript},
        "a < b! < c",            // {"parser": parsers.typescript},
        "a < (b as number) < c", // {"parser": parsers.typescript}
    ];

    Tester::new(NoChainedComparison::NAME, NoChainedComparison::PLUGIN, pass, fail)
        .test_and_snapshot();
}
