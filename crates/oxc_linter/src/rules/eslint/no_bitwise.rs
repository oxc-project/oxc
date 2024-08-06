use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use oxc_syntax::operator::BinaryOperator;

use crate::{context::LintContext, rule::Rule, utils::Set, AstNode};

fn no_bitwise_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected use of {x0:?}"))
        .with_help("bitwise operators are not allowed, maybe you mistyped `&&` or `||`")
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct NoBitwise(Box<NoBitwiseConfig>);

#[derive(Debug, Default, Clone)]
pub struct NoBitwiseConfig {
    allow: Set<CompactStr>,
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
    /// ### Example
    ///
    /// ```javascript
    /// var x = y | z;
    /// ```
    NoBitwise,
    restriction
);

impl Rule for NoBitwise {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);

        Self(Box::new(NoBitwiseConfig {
            allow: obj
                .and_then(|v| v.get("allow"))
                .and_then(|v| Set::try_from(v).ok())
                .unwrap_or_default(),
            int32_hint: obj
                .and_then(|v| v.get("int32Hint"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                let op = bin_expr.operator.as_str();

                if bin_expr.operator.is_bitwise() && !self.is_allowed(op, node) {
                    ctx.diagnostic(no_bitwise_diagnostic(op, bin_expr.span));
                }
            }
            AstKind::UnaryExpression(unary_expr) => {
                let op = unary_expr.operator.as_str();

                if unary_expr.operator.is_bitwise() && !self.is_allowed(op, node) {
                    ctx.diagnostic(no_bitwise_diagnostic(op, unary_expr.span));
                }
            }
            AstKind::AssignmentExpression(assign_expr) => {
                let op = assign_expr.operator.as_str();

                if assign_expr.operator.is_bitwise() && !self.is_allowed(op, node) {
                    ctx.diagnostic(no_bitwise_diagnostic(op, assign_expr.span));
                }
            }
            _ => {}
        }
    }
}

impl NoBitwise {
    pub(self) fn is_allowed(&self, operator: &str, node: &AstNode) -> bool {
        self.is_allowed_operator(operator) || self.is_int32_hint(node)
    }

    #[inline]
    fn is_allowed_operator(&self, operator: &str) -> bool {
        self.allow.contains_str(operator)
    }

    fn is_int32_hint(&self, node: &AstNode) -> bool {
        if !self.int32_hint {
            return false;
        }

        match node.kind() {
            AstKind::BinaryExpression(bin_expr) => {
                bin_expr.operator == BinaryOperator::BitwiseOR && bin_expr.right.is_number_0()
            }
            _ => false,
        }
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
        ("~[1, 2, 3].indexOf(1)", Some(json!([ { "allow": ["~"] }]))),
        ("~1<<2 === -8", Some(json!([ { "allow": ["~", "<<"] }]))),
        ("a|0", Some(json!([ { "int32Hint": true}]))),
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

    Tester::new(NoBitwise::NAME, pass, fail).test_and_snapshot();
}
