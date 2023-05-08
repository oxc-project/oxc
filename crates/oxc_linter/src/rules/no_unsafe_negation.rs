use oxc_ast::{
    ast::{BinaryExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_formatter::Gen;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::{BinaryOperator, UnaryOperator};

use crate::{context::LintContext, fixer::Fix, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("Unexpected logical not in the left hand side of '{0}' operator")]
#[diagnostic(
    severity(warning),
    help(
        "use parenthesis to express the negation of the whole boolean expression, as '!' binds more closely than '{0}'"
    )
)]
struct NoUnsafeNegationDiagnostic(&'static str, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUnsafeNegation {
    /// true: disallow negation of the left-hand side of ordering relational operators
    /// false: allow negation of the left-hand side of ordering relational operators (<, >, <=, >=)
    enforce_for_ordering_relations: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Disallow negating the left operand of relational operators
    ///
    /// ### Why is this bad?
    /// Just as developers might type -a + b when they mean -(a + b) for the negative of a sum,
    /// they might type !key in object by mistake when they almost certainly mean !(key in object)
    /// to test that a key is not in an object. !obj instanceof Ctor is similar.
    ///
    /// ### Example
    /// ```javascript
    /// if (!key in object) {
    ///   //operator precedence makes it equivalent to (!key) in object
    ///   //and type conversion makes it equivalent to (key ? "false" : "true") in object
    /// }
    /// ```
    NoUnsafeNegation,
    correctness
);

impl Rule for NoUnsafeNegation {
    fn from_configuration(value: serde_json::Value) -> Self {
        let enforce_for_ordering_relations = value
            .get(0)
            .and_then(|config| config.get("enforceForOrderingRelations"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or_default();
        Self { enforce_for_ordering_relations }
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.get().kind() else { return; };
        if self.should_check(expr.operator) {
            let Expression::UnaryExpression(left) = &expr.left else { return; };
            if left.operator == UnaryOperator::LogicalNot {
                Self::report_with_fix(expr, ctx);
            }
        }
    }
}

impl NoUnsafeNegation {
    fn should_check(&self, op: BinaryOperator) -> bool {
        op.is_relational() || (self.enforce_for_ordering_relations && op.is_compare())
    }

    /// Precondition:
    /// expr.left is `UnaryExpression` whose operator is '!'
    fn report_with_fix(expr: &BinaryExpression, ctx: &LintContext<'_>) {
        // Diagnostic points at the unexpected negation
        let diagnostic = NoUnsafeNegationDiagnostic(expr.operator.as_str(), expr.left.span());

        let fix_producer = || {
            // modify `!a instance of B` to `!(a instanceof B)`
            let modified_code = {
                let mut formatter = ctx.formatter();
                formatter.print(b'!');
                let Expression::UnaryExpression(left) = &expr.left else { unreachable!() };
                formatter.print(b'(');
                left.argument.gen(&mut formatter);
                expr.operator.gen(&mut formatter);
                expr.right.gen(&mut formatter);
                formatter.print(b')');
                formatter.into_code()
            };
            Fix::new(modified_code, expr.span)
        };

        ctx.diagnostic_with_fix(diagnostic, fix_producer);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("a in b", None),
        ("a in b === false", None),
        ("!(a in b)", None),
        ("(!a) in b", None),
        ("a instanceof b", None),
        ("a instanceof b === false", None),
        ("!(a instanceof b)", None),
        ("(!a) instanceof b", None),
        ("if (! a < b) {}", None),
        ("while (! a > b) {}", None),
        ("foo = ! a <= b;", None),
        ("foo = ! a >= b;", None),
        ("! a <= b", Some(serde_json::json!([{}]))),
        ("foo = ! a >= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": false }]))),
        ("foo = (!a) >= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("a <= b", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("!(a < b)", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("foo = a > b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
    ];

    let fail = vec![
        ("!a in b", None),
        ("(!a in b)", None),
        ("!(a) in b", None),
        ("!a instanceof b", None),
        ("(!a instanceof b)", None),
        ("!(a) instanceof b", None),
        ("if (! a < b) {}", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("while (! a > b) {}", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("foo = ! a <= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("foo = ! a >= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("! a <= b", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
    ];

    Tester::new(NoUnsafeNegation::NAME, pass, fail).test_and_snapshot();
}
