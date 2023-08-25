use oxc_ast::AstKind;
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-mixed-operators): Unexpected mix of {0} with {1}")]
#[diagnostic(
    severity(warning),
    help("Use parentheses to clarify the intended order of operations.")
)]
struct NoMixedOperatorsDiagnostic(
    &'static str,      /*Node Operator */
    &'static str,      /*Parent Operator */
    #[label] pub Span, /*Span of the node operator */
    #[label] pub Span, /*Span of the parent operator */
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoMixedOperators {
    /// Disallow Mixed operators within one group.
    groups: Vec<Vec<&'static str>>,
    /// Allow operators of the same precedence to be mixed.
    allow_same_precedence: bool,
}

impl Default for NoMixedOperators {
    fn default() -> Self {
        Self { groups: default_groups(), allow_same_precedence: true }
    }
}

declare_oxc_lint! {
  /// ### What it does
  /// Disallow mixed binary operators.
  ///
  /// ### Why is this bad?
  /// Enclosing complex expressions by parentheses clarifies the developer’s intention,
  /// which makes the code more readable. This rule warns when different operators
  /// are used consecutively without parentheses in an expression.
  ///
  /// ### Examples
  /// ```javascript
  /// var foo = a && b || c || d;    /*BAD: Unexpected mix of '&&' and '||'.*/
  /// var foo = (a && b) || c || d;  /*GOOD*/
  /// var foo = a && (b || c || d);  /*GOOD*/
  /// ```
  NoMixedOperators,
  pedantic,
}

impl Rule for NoMixedOperators {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let node_kind = node.kind();
        if !matches!(node_kind, AstKind::BinaryExpression(_) | AstKind::LogicalExpression(_)) {
            return;
        }

        let Some(parent_kind) = ctx.nodes().parent_kind(node.id()) else { return };

        if !matches!(
            parent_kind,
            AstKind::BinaryExpression(_)
                | AstKind::LogicalExpression(_)
                | AstKind::ConditionalExpression(_)
        ) {
            return;
        }

        // Now we know that node is a BinaryExpression or LogicalExpression, and parent
        // is a BinaryExpression or LogicalExpression or ConditionalExpression.
        if Self::is_mixed_with_parent(node_kind, parent_kind) {
            self.report(node_kind, parent_kind, ctx);
        }
    }

    fn from_configuration(value: serde_json::Value) -> Self {
        Self::try_from_configuration(&value).unwrap_or_default()
    }
}

impl NoMixedOperators {
    pub fn try_from_configuration(value: &serde_json::Value) -> Option<Self> {
        let config = value.get(0)?;

        let mut groups = vec![];
        if let Some(groups_config) = config.get("groups") {
            if let Some(groups_config) = groups_config.as_array() {
                'outer: for group_config in groups_config {
                    // Parse current group configuration. On failure fall through to next group.
                    if let Some(group_config) = group_config.as_array() {
                        let mut group = vec![];
                        for val in group_config {
                            let Some(val) = val.as_str() else { continue 'outer };
                            let Some((operator, _)) = operator_and_precedence(val) else {
                                continue 'outer;
                            };
                            group.push(operator);
                        }
                        groups.push(group);
                    }
                }
            }
        }

        if groups.is_empty() {
            groups = default_groups();
        }

        let allow_same_precedence =
            config.get("allowSamePrecedence").map_or(true, |val| val.as_bool().unwrap_or_default());

        Some(Self { groups, allow_same_precedence })
    }

    fn is_mixed_with_parent(node: AstKind, parent: AstKind) -> bool {
        match (node, parent) {
            (AstKind::BinaryExpression(node), AstKind::BinaryExpression(parent)) => {
                node.operator != parent.operator
            }
            (AstKind::LogicalExpression(node), AstKind::LogicalExpression(parent)) => {
                node.operator != parent.operator
            }
            _ => true,
        }
        // Note that there is not need to check for parenthesis explicitly because if an
        // expression is parenthesized, its parent node is a ParenthesizedExpression and will
        // never enter the code path.
    }

    /// Report mixed operator pare between node and parent corresponding to configuration.
    fn report(&self, node: AstKind, parent: AstKind, ctx: &LintContext<'_>) {
        let (node_operator, node_left_span, node_right_span) = match node {
            AstKind::BinaryExpression(expr) => {
                (expr.operator.as_str(), expr.left.span(), expr.right.span())
            }
            AstKind::LogicalExpression(expr) => {
                (expr.operator.as_str(), expr.left.span(), expr.right.span())
            }
            _ => unreachable!(),
        };
        // Since we don't store the exact span of the operators, approximate that span to be between the lhs
        // and rhs of the expression.
        let node_operator_span = Span::new(node_left_span.end + 1, node_right_span.start - 1);

        let (parent_operator, parent_left_span, parent_right_span) = match parent {
            AstKind::BinaryExpression(expr) => {
                (expr.operator.as_str(), expr.left.span(), expr.right.span())
            }
            AstKind::LogicalExpression(expr) => {
                (expr.operator.as_str(), expr.left.span(), expr.right.span())
            }
            AstKind::ConditionalExpression(expr) => {
                // For conditional operators, the span covers both ? and :
                ("?:", expr.test.span(), expr.alternate.span())
            }
            _ => unreachable!(),
        };
        let parent_operator_span = Span::new(parent_left_span.end + 1, parent_right_span.start - 1);

        let (node_operator, node_precedence) = operator_and_precedence(node_operator).unwrap();
        let (parent_operator, parent_precedence) =
            operator_and_precedence(parent_operator).unwrap();
        if !(self.allow_same_precedence && node_precedence == parent_precedence)
            && self.in_the_same_group(node_operator, parent_operator)
        {
            // Report error at both operators
            ctx.diagnostic(NoMixedOperatorsDiagnostic(
                node_operator,
                parent_operator,
                node_operator_span,
                parent_operator_span,
            ));
        }
    }

    fn in_the_same_group(&self, op1: &str, op2: &str) -> bool {
        self.groups.iter().any(|group| {
            let mut contains_op1 = false;
            let mut contains_op2 = false;
            for &op in group {
                if op == op1 {
                    contains_op1 = true;
                }
                if op == op2 {
                    contains_op2 = true;
                }
            }
            contains_op1 && contains_op2
        })
    }
}

#[rustfmt::skip]
static OPERATORS: [&str; 27] = [
  "+", "-", "*", "/", "%", "**",                  /* Arithmetic operator: 6 */
  "&", "|", "^", "~", "<<", ">>", ">>>",          /*Bitwise operator: 13 */
  "==", "!=", "===", "!==", ">", ">=", "<", "<=", /*Compare operator: 21 */
  "&&", "||",                                     /*Logical operator: 23 */
  "in", "instanceof",                             /*Relational operator: 25 */
  "?:",                                           /*Conditional operator */
  "??",                                           /*Coalesce operator */
];

/// `https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Operator_Precedence#table`
#[rustfmt::skip]
const PRECEDENCES: [u8; 27] = [
  11, 11, 12, 12, 12, 13,
  7, 5, 6, 14, 10, 10, 10,
  8, 8, 8, 8, 9, 9, 9, 9,
  4, 3,
  9, 9,
  2,
  3
];

#[inline]
fn default_groups() -> Vec<Vec<&'static str>> {
    let arithmetic: &[&str] = &OPERATORS[..6];
    let bitwise: &[&str] = &OPERATORS[6..13];
    let compare: &[&str] = &OPERATORS[13..21];
    let logical: &[&str] = &OPERATORS[21..23];
    let relational: &[&str] = &OPERATORS[23..25];
    let default_operators: [&[&str]; 5] = [arithmetic, bitwise, compare, logical, relational];
    default_operators.iter().map(|operators| operators.to_vec()).collect()
}

#[inline]
fn operator_and_precedence(operator: &str) -> Option<(&'static str, u8)> {
    OPERATORS.iter().position(|op| *op == operator).map(|idx| (OPERATORS[idx], PRECEDENCES[idx]))
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        ("a && b && c && d", None),
        ("a || b || c || d", None),
        ("(a || b) && c && d", None),
        ("a || (b && c && d)", None),
        ("(a || b || c) && d", None),
        ("a || b || (c && d)", None),
        ("a + b + c + d", None),
        ("a * b * c * d", None),
        ("a == 0 && b == 1", None),
        ("a == 0 || b == 1", None),
        ("(a == 0) && (b == 1)", Some(json!([{"groups": [["&&", "=="]]}]))),
        ("a + b - c * d / e", Some(json!([{ "groups": [["&&", "||"]] }]))),
        ("a + b - c", None),
        ("a * b / c", None),
        ("a + b - c", Some(json!([{ "allowSamePrecedence": true }]))),
        ("a * b / c", Some(json!([{ "allowSamePrecedence": true }]))),
        ("(a || b) ? c : d", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a ? (b || c) : d", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a ? b : (c || d)", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a || (b ? c : d)", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("(a ? b : c) || d", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a || (b ? c : d)", None),
        ("(a || b) ? c : d", None),
        ("a || b ? c : d", None),
        ("a ? (b || c) : d", None),
        ("a ? b || c : d", None),
        ("a ? b : (c || d)", None),
        ("a ? b : c || d", None),
    ];

    let fail = vec![
        ("a && b || c", None),
        ("a && b > 0 || c", Some(json!([{ "groups": [["&&", "||", ">"]] }]))),
        ("a && b > 0 || c", Some(json!([{ "groups": [["&&", "||"]] }]))),
        (
            "a && b + c - d / e || f",
            Some(json!([{ "groups": [["&&", "||"], ["+", "-", "*", "/"]] }])),
        ),
        (
            "a && b + c - d / e || f",
            Some(
                json!([{ "groups": [["&&", "||"], ["+", "-", "*", "/"]], "allowSamePrecedence": true }]),
            ),
        ),
        ("a + b - c", Some(json!([{ "allowSamePrecedence": false }]))),
        ("a * b / c", Some(json!([{ "allowSamePrecedence": false }]))),
        ("a || b ? c : d", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a && b ? 1 : 2", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("x ? a && b : 0", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("x ? 0 : a && b", Some(json!([{ "groups": [["&&", "||", "?:"]] }]))),
        ("a + b ?? c", Some(json!([{ "groups": [["+", "??"]] }]))),
        ("a in b ?? c", Some(json!([{ "groups": [["in", "??"]] }]))),
    ];

    Tester::new(NoMixedOperators::NAME, pass, fail).test_and_snapshot();
}

#[cfg(test)]
mod internal_tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_from_configuration() {
        let config = json!([{
            "groups": [
                ["+", "-", "*", "/", "%", "**"],
                ["&", "|", "^", "~", "<<", ">>", ">>>"],
                ["==", "!=", "===", "!==", ">", ">=", "<", "<="],
                ["&&", "||"],
                ["in", "instanceof"]
            ],
            "allowSamePrecedence": true
        }]);
        let rule = NoMixedOperators::try_from_configuration(&config);
        assert_eq!(Some(NoMixedOperators::default()), rule);
    }

    #[test]
    fn test_nornmalize_configuration() {
        let config = json!([
          { "allowSamePrecedence": false }
        ]);
        let rule = NoMixedOperators::try_from_configuration(&config);
        // missing groups should fall back to default
        let expected = NoMixedOperators { groups: default_groups(), allow_same_precedence: false };
        assert_eq!(Some(expected), rule);
    }
}
