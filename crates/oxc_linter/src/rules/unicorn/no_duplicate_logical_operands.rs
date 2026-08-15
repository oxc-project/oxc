use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::LogicalOperator;

use crate::{AstNode, context::LintContext, rule::Rule, utils::is_same_expression};

fn no_duplicate_logical_operands_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("This operand duplicates the left operand.")
        .with_help("Remove the duplicate operand.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDuplicateLogicalOperands;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow adjacent duplicate operands in `&&` and `||` expressions.
    ///
    /// ### Why is this bad?
    ///
    /// Repeating an operand next to itself has no effect on the result, so the
    /// duplicate is either dead code or a typo hiding the operand that was meant
    /// to be there.
    ///
    /// Only side-effect-free references are compared, because repeating an
    /// expression that can have side effects, such as `getValue() && getValue()`,
    /// is not necessarily redundant.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// foo && foo
    /// foo || bar || bar
    /// foo.bar && foo.bar
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// foo && bar
    /// foo && bar && foo
    /// getValue() && getValue()
    /// foo ?? foo
    /// ```
    NoDuplicateLogicalOperands,
    unicorn,
    suspicious,
    fix_conditional,
    version = "next",
    short_description = "Disallow adjacent duplicate operands in logical expressions.",
);

/// A computed property that is stable enough to compare, e.g. the `bar` in `foo[bar]`.
fn is_simple_computed_property(expr: &Expression) -> bool {
    matches!(
        expr.get_inner_expression(),
        Expression::Identifier(_)
            | Expression::ThisExpression(_)
            | Expression::StringLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

/// A reference that can be repeated without side effects, e.g. `foo`, `this.foo`, `foo[bar].baz`.
///
/// Optional chains are excluded: `foo?.bar` short-circuits, so the second operand is not
/// guaranteed to evaluate the same way as the first.
fn is_simple_reference(expr: &Expression) -> bool {
    match expr.get_inner_expression() {
        Expression::Identifier(_) | Expression::ThisExpression(_) | Expression::Super(_) => true,
        Expression::StaticMemberExpression(member) => {
            !member.optional && is_simple_reference(&member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            !member.optional
                && is_simple_reference(&member.object)
                && is_simple_computed_property(&member.expression)
        }
        Expression::PrivateFieldExpression(member) => {
            !member.optional && is_simple_reference(&member.object)
        }
        _ => false,
    }
}

/// Replacing the whole expression with its left operand is only safe when it cannot change
/// what a bare identifier resolves to, so bail out inside `with`.
fn is_inside_with_statement(node: &AstNode, ctx: &LintContext) -> bool {
    ctx.nodes()
        .ancestors(node.id())
        .any(|ancestor| matches!(ancestor.kind(), AstKind::WithStatement(_)))
}

impl Rule for NoDuplicateLogicalOperands {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::LogicalExpression(logical_expr) = node.kind() else {
            return;
        };

        if !matches!(logical_expr.operator, LogicalOperator::And | LogicalOperator::Or) {
            return;
        }

        // `a && b && c` parses as `(a && b) && c`, so the operand adjacent to `c` is `b`.
        // Only the adjacent pair is compared, which is why `foo && bar && foo` is allowed.
        let adjacent_left_operand = match logical_expr.left.without_parentheses() {
            Expression::LogicalExpression(left_logical_expr)
                if left_logical_expr.operator == logical_expr.operator =>
            {
                &left_logical_expr.right
            }
            _ => &logical_expr.left,
        };

        if !is_simple_reference(adjacent_left_operand)
            || !is_simple_reference(&logical_expr.right)
            || !is_same_expression(
                adjacent_left_operand.get_inner_expression(),
                logical_expr.right.get_inner_expression(),
                ctx,
            )
            || is_inside_with_statement(node, ctx)
        {
            return;
        }

        let diagnostic = no_duplicate_logical_operands_diagnostic(logical_expr.right.span());

        // The fix drops everything but the left operand, so every comment in the expression
        // has to already live inside that operand or it would be deleted with the duplicate.
        let comments_in_expression =
            ctx.comments_range(logical_expr.span.start..logical_expr.span.end).count();
        let comments_in_left_operand = ctx
            .comments_range(logical_expr.left.span().start..logical_expr.left.span().end)
            .count();

        if comments_in_expression != comments_in_left_operand
            || !is_safely_autofixable(adjacent_left_operand)
            || !is_safely_autofixable(&logical_expr.right)
        {
            ctx.diagnostic(diagnostic);
            return;
        }

        ctx.diagnostic_with_fix(diagnostic, |fixer| {
            fixer.replace_with(&logical_expr.span, &logical_expr.left.span())
        });
    }
}

/// Member expressions are left alone because a getter can make `foo.bar` observable, so
/// deleting the second read is a behaviour change rather than a cleanup.
fn is_safely_autofixable(expr: &Expression) -> bool {
    matches!(expr.without_parentheses(), Expression::Identifier(_) | Expression::ThisExpression(_))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo && bar",
        "foo || bar",
        "foo && bar && foo",
        "foo && (foo && bar)",
        "foo ?? foo",
        "getValue() && getValue()",
        "getValue() || getValue()",
        "foo.bar() && foo.bar()",
        "foo + bar && foo + bar",
        "await foo && await foo",
        "foo++ && foo++",
        "(foo = bar) && (foo = bar)",
        "foo[bar()] && foo[bar()]",
        "foo[bar + baz] || foo[bar + baz]",
        "foo?.bar && foo?.bar",
        "foo?.[bar] || foo?.[bar]",
        "true && true",
        "0 || 0",
        "with (scope) {foo && foo;}", // {"sourceType": "script"}
    ];

    let fail = vec![
        "foo && foo",
        "foo || foo",
        "foo && bar && bar",
        "foo || bar || bar",
        "(foo) && (foo)",
        "this && this",
        "foo.bar && foo.bar",
        "foo.bar || foo.bar",
        "foo.bar.baz && foo.bar.baz",
        "foo && bar.baz && bar.baz",
        "this.foo && this.foo",
        "class Foo {#foo; method() {return this.#foo && this.#foo;}}",
        "class Foo extends Bar {method() {return super.foo && super.foo;}}",
        "foo[bar] && foo[bar]",
        r#"foo["bar"] || foo.bar"#,
        "(foo as boolean) && (foo as boolean)", // {"parser": parsers.typescript},
        "(foo as Foo) && (foo as Bar)",         // {"parser": parsers.typescript},
        "foo && (foo as Foo)",                  // {"parser": parsers.typescript},
        "(<boolean>foo) && (<boolean>foo)",     // {"parser": parsers.typescript},
        "foo! || foo!",                         // {"parser": parsers.typescript},
        "foo! && foo",                          // {"parser": parsers.typescript},
        "foo && foo!",                          // {"parser": parsers.typescript},
        "(foo satisfies boolean) && (foo satisfies boolean)", // {"parser": parsers.typescript},
        "foo && (foo satisfies Foo)",           // {"parser": parsers.typescript},
        "foo /* keep */ && foo",
        "foo && /* keep */ foo",
        "foo && bar /* keep */ && bar",
        "foo && (foo /* keep */)",
        "(foo /* keep */) && foo",
    ];

    let fix = vec![
        ("foo && foo", "foo"),
        ("foo || foo", "foo"),
        ("foo && bar && bar", "foo && bar"),
        ("(foo) && (foo)", "(foo)"),
        ("this && this", "this"),
        // Member expressions are reported but not fixed, a getter could be observable.
        ("foo.bar && foo.bar", "foo.bar && foo.bar"),
        // Comments outside the left operand would be dropped by the fix.
        ("foo && /* keep */ foo", "foo && /* keep */ foo"),
        ("foo /* keep */ && foo", "foo /* keep */ && foo"),
    ];

    Tester::new(NoDuplicateLogicalOperands::NAME, NoDuplicateLogicalOperands::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
