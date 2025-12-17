use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::operator::UnaryOperator;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::RuleFixer,
    rule::{DefaultRuleConfig, Rule},
};

fn no_unsafe_negation_diagnostic(operator: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Unexpected negation of the left operand of '{operator}' operator."
    ))
    .with_help(format!(
        "Use `()` to negate the whole expression, as '!' binds more closely than '{operator}'"
    ))
    .with_label(span)
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoUnsafeNegation {
    /// The `enforceForOrderingRelations` option determines whether negation is allowed
    /// on the left-hand side of ordering relational operators (<, >, <=, >=).
    ///
    /// The purpose is to avoid expressions such as `!a < b` (which is equivalent to `(a ? 0 : 1) < b`)
    /// when what is really intended is `!(a < b)`.
    enforce_for_ordering_relations: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows negating the left operand of relational operators to prevent logical errors
    /// caused by misunderstanding operator precedence or accidental use of negation.
    ///
    /// This rule can be disabled for TypeScript code, as the TypeScript compiler
    /// enforces this check.
    ///
    /// ### Why is this bad?
    ///
    /// Negating the left operand of relational operators can result in unexpected behavior due to
    /// operator precedence, leading to logical errors. For instance, `!a in b` may be interpreted
    /// as `(!a) in b` instead of `!(a in b)`, which is not the intended logic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// if (!key in object) {}
    ///
    /// if (!obj instanceof Ctor) {}
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// if (!(key in object)) {}
    ///
    /// if (!(obj instanceof Ctor)) {}
    /// ```
    NoUnsafeNegation,
    eslint,
    correctness,
    fix,
    config = NoUnsafeNegation,
);

impl Rule for NoUnsafeNegation {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoUnsafeNegation>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };

        let should_check = expr.operator.is_relational()
            || (self.enforce_for_ordering_relations && expr.operator.is_compare());
        if should_check {
            let Expression::UnaryExpression(left) = &expr.left else {
                return;
            };
            if left.operator == UnaryOperator::LogicalNot {
                // Diagnostic points at the unexpected negation
                let diagnostic =
                    no_unsafe_negation_diagnostic(expr.operator.as_str(), expr.left.span());

                let fix_producer = |fixer: RuleFixer<'_, 'a>| {
                    // modify `!a instance of B` to `!(a instanceof B)`
                    let modified_code = {
                        let left = ctx.source_range(left.argument.span());
                        let operator = expr.operator.as_str();
                        let right = ctx.source_range(expr.right.span());

                        format!("!({left} {operator} {right})")
                    };
                    fixer.replace(expr.span, modified_code)
                };

                ctx.diagnostic_with_fix(diagnostic, fix_producer);
            }
        }
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
        ("(y=>{if(!/s/ in(l)){}})", None),
        ("if (! a < b) {}", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("while (! a > b) {}", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("foo = ! a <= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("foo = ! a >= b;", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
        ("! a <= b", Some(serde_json::json!([{ "enforceForOrderingRelations": true }]))),
    ];

    let fix = vec![
        ("!a in b", "!(a in b)", None),
        ("(!a in b)", "(!(a in b))", None),
        ("!(a) in b", "!((a) in b)", None),
        ("!a instanceof b", "!(a instanceof b)", None),
        ("(!a instanceof b)", "(!(a instanceof b))", None),
        ("!(a) instanceof b", "!((a) instanceof b)", None),
        (
            "if (! a < b) {}",
            "if (!(a < b)) {}",
            Some(serde_json::json!([{ "enforceForOrderingRelations": true }])),
        ),
        (
            "while (! a > b) {}",
            "while (!(a > b)) {}",
            Some(serde_json::json!([{ "enforceForOrderingRelations": true }])),
        ),
        (
            "foo = ! a <= b;",
            "foo = !(a <= b);",
            Some(serde_json::json!([{ "enforceForOrderingRelations": true }])),
        ),
        (
            "foo = ! a >= b;",
            "foo = !(a >= b);",
            Some(serde_json::json!([{ "enforceForOrderingRelations": true }])),
        ),
        (
            "!a <= b",
            "!(a <= b)",
            Some(serde_json::json!([{ "enforceForOrderingRelations": true }])),
        ),
    ];

    Tester::new(NoUnsafeNegation::NAME, NoUnsafeNegation::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
