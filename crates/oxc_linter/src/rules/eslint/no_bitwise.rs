use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::operator::BinaryOperator;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_bitwise_diagnostic(operator: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected use of `{operator:?}`."))
        .with_help("bitwise operators are not allowed, maybe you mistyped `&&` or `||`?")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoBitwise(Box<NoBitwiseConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NoBitwiseConfig {
    /// The `allow` option permits the given list of bitwise operators to be used
    /// as exceptions to this rule.
    ///
    /// For example `{ "allow": ["~"] }` would allow the use of the bitwise operator
    /// `~` without restriction. Such as in the following:
    ///
    /// ```javascript
    /// ~[1,2,3].indexOf(1) === -1;
    /// ```
    allow: Vec<CompactStr>,
    /// When set to `true` the `int32Hint` option allows the use of bitwise OR in |0
    /// pattern for type casting.
    ///
    /// For example with `{ "int32Hint": true }` the following is permitted:
    ///
    /// ```javascript
    /// const b = a|0;
    /// ```
    int32_hint: bool,
}

impl std::ops::Deref for NoBitwise {
    type Target = NoBitwiseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow bitwise operators
    ///
    /// ### Why is this bad?
    ///
    /// The use of bitwise operators in JavaScript is very rare and often `&` or `|` is simply a mistyped `&&` or `||`,
    /// which will lead to unexpected behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var x = y | z;
    /// ```
    ///
    /// ```javascript
    /// var x = y ^ z;
    /// ```
    ///
    /// ```javascript
    /// var x = y >> z;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var x = y || z;
    /// ```
    ///
    /// ```javascript
    /// var x = y && z;
    /// ```
    ///
    /// ```javascript
    /// var x = y > z;
    /// ```
    NoBitwise,
    eslint,
    restriction,
    config = NoBitwiseConfig,
);

impl Rule for NoBitwise {
    fn from_configuration(value: Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<NoBitwise>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                let op = bin_expr.operator.as_str();

                if bin_expr.operator.is_bitwise()
                    && !allowed_operator(&self.allow, op)
                    && !is_int32_hint(self.int32_hint, node)
                {
                    ctx.diagnostic(no_bitwise_diagnostic(op, bin_expr.span));
                }
            }
            AstKind::UnaryExpression(unary_expr) => {
                let op = unary_expr.operator.as_str();

                if unary_expr.operator.is_bitwise()
                    && !allowed_operator(&self.allow, op)
                    && !is_int32_hint(self.int32_hint, node)
                {
                    ctx.diagnostic(no_bitwise_diagnostic(op, unary_expr.span));
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                let op = assign_expr.operator.as_str();

                if assign_expr.operator.is_bitwise()
                    && !allowed_operator(&self.allow, op)
                    && !is_int32_hint(self.int32_hint, node)
                {
                    ctx.diagnostic(no_bitwise_diagnostic(op, assign_expr.span));
                }
            }
            _ => {}
        }
    }
}

fn allowed_operator(allow: &[CompactStr], operator: &str) -> bool {
    allow.iter().any(|s| s == operator)
}

fn is_int32_hint(int32_hint: bool, node: &AstNode) -> bool {
    if !int32_hint {
        return false;
    }

    match node.kind() {
        AstKind::BinaryExpression(bin_expr) => {
            bin_expr.operator == BinaryOperator::BitwiseOR && bin_expr.right.is_number_0()
        }
        _ => false,
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("a + b", None),
        ("!a", None),
        ("a && b", None),
        ("a || b", None),
        ("a += b", None),
        ("a &&= b", None),
        ("a ||= b", None),
        ("a ??= b", None),
        ("a > b", None),
        ("a < b", None),
        ("~[1, 2, 3].indexOf(1)", Some(json!([ { "allow": ["~"] }]))),
        ("~1<<2 === -8", Some(json!([ { "allow": ["~", "<<"] }]))),
        ("a|0", Some(json!([ { "int32Hint": true }]))),
        ("a|0", Some(json!([ { "int32Hint": false, "allow": ["|"] }]))),
    ];

    let fail = vec![
        ("a ^ b", None),
        ("a | b", None),
        ("a & b", None),
        ("a << b", None),
        ("a >> b", None),
        ("a >>> b", None),
        ("~a", None),
        ("a ^= b", None),
        ("a |= b", None),
        ("a &= b", None),
        ("a <<= b", None),
        ("a >>= b", None),
        ("a >>>= b", None),
    ];

    Tester::new(NoBitwise::NAME, NoBitwise::PLUGIN, pass, fail).test_and_snapshot();
}
