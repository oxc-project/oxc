use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de};
use serde_json::Value;

use oxc_ast::{
    AstKind,
    ast::{
        Argument, AssignmentExpression, AssignmentTarget, BinaryExpression, BinaryOperator,
        CallExpression, Expression, IfStatement, LogicalExpression, LogicalOperator,
        SimpleAssignmentTarget, Statement, UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::AssignmentOperator;

use crate::{
    AstNode,
    context::LintContext,
    rule::{Rule, TupleRuleConfig},
    utils::is_same_member_expression,
};

fn logical_assignment_operators_diagnostic(
    kind: &LogicalAssignmentOperatorsDiagnosticKind,
    operator: LogicalOperator,
    span: Span,
) -> OxcDiagnostic {
    let operator = operator.to_assignment_operator().as_str();
    let message = match kind {
        LogicalAssignmentOperatorsDiagnosticKind::Assignment => {
            format!("Assignment (=) can be replaced with operator assignment ({operator}).")
        }
        LogicalAssignmentOperatorsDiagnosticKind::Logical => {
            format!("Logical expression can be replaced with an assignment ({operator}).")
        }
        LogicalAssignmentOperatorsDiagnosticKind::If => {
            format!(
                "`if` statement can be replaced with a logical operator assignment with operator `{operator}`."
            )
        }
        LogicalAssignmentOperatorsDiagnosticKind::Unexpected => {
            format!("Unexpected logical operator assignment ({operator}) shorthand.")
        }
    };

    OxcDiagnostic::warn(message).with_label(span)
}

enum LogicalAssignmentOperatorsDiagnosticKind {
    Assignment,
    Logical,
    If,
    Unexpected,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
enum LogicalAssignmentOperatorsMode {
    #[default]
    /// This option checks for expressions that can be shortened using logical assignment operator.
    /// For example, `a = a || b` can be shortened to `a ||= b`.
    /// Expressions with associativity such as `a = a || b || c` are reported as being able to be shortened to `a ||= b || c` unless the evaluation order is explicitly defined using parentheses, such as `a = (a || b) || c`.
    Always,
    /// This option disallows logical assignment operator shorthand.
    /// For example, `a ||= b` should be written as `a = a || b`.
    Never,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct LogicalAssignmentOperatorsConfig {
    /// This option checks for additional patterns with if statements which could be expressed with the logical assignment operator.
    /// Only available if string option is set to `always`.
    ///
    /// Examples of **incorrect** code for this rule with the `["always", { enforceForIfStatements: true }]` option:
    /// ```js
    /// if (a) a = b // <=> a &&= b
    /// if (!a) a = b // <=> a ||= b
    ///
    /// if (a == null) a = b // <=> a ??= b
    /// if (a === null || a === undefined) a = b // <=> a ??= b
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `["always", { enforceForIfStatements: true }]` option:
    /// ```js
    /// if (a) b = c
    /// if (a === 0) a = b
    /// ```
    enforce_for_if_statements: bool,
}

#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
pub struct LogicalAssignmentOperators(
    LogicalAssignmentOperatorsMode,
    LogicalAssignmentOperatorsConfig,
);

impl<'de> Deserialize<'de> for LogicalAssignmentOperators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let values = Vec::<Value>::deserialize(deserializer)?;
        let mut mode = LogicalAssignmentOperatorsMode::Always;
        let mut options = LogicalAssignmentOperatorsConfig::default();

        match values.as_slice() {
            [] => {}
            [first] if first.is_string() => {
                mode = serde_json::from_value(first.clone()).map_err(de::Error::custom)?;
            }
            [first, second] if first.is_string() && second.is_object() => {
                mode = serde_json::from_value(first.clone()).map_err(de::Error::custom)?;
                if mode == LogicalAssignmentOperatorsMode::Never {
                    return Err(de::Error::custom(r#"options object is only valid with "always""#));
                }
                options = serde_json::from_value(second.clone()).map_err(de::Error::custom)?;
            }
            _ => {
                return Err(de::Error::custom(
                    r#"expected [], ["always"], ["never"], or ["always", options]"#,
                ));
            }
        }

        Ok(Self(mode, options))
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule requires or disallows logical assignment operator shorthand.
    ///
    /// ### Why is this bad?
    ///
    /// ES2021 introduces the assignment operator shorthand for the logical operators `||`, `&&` and `??`.
    /// Before, this was only allowed for mathematical operations such as `+` or `*` (see the rule `operator-assignment`).
    /// The shorthand can be used if the assignment target and the left expression of a logical expression are the same.
    /// For example `a = a || b` can be shortened to `a ||= b`.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule with the default `always` option:
    /// ```js
    /// a = a || b
    /// a = a && b
    /// a = a ?? b
    /// a || (a = b)
    /// a && (a = b)
    /// a ?? (a = b)
    /// a = a || b || c
    /// a = a && b && c
    /// a = a ?? b ?? c
    /// ```
    ///
    /// Examples of **correct** code for this rule with the default `always` option:
    /// ```js
    /// a = b
    /// a += b
    /// a ||= b
    /// a = b || c
    /// a || (b = c)
    /// if (a) a = b
    /// a = (a || b) || c
    /// ```
    ///
    /// Examples of **incorrect** code for this rule with the `never` option:
    /// ```js
    /// a ||= b
    /// a &&= b
    /// a ??= b
    /// ```
    ///
    /// Examples of **correct** code for this rule with the `never` option:
    /// ```js
    /// a = a || b
    /// a = a && b
    /// a = a ?? b
    /// ```
    LogicalAssignmentOperators,
    eslint,
    style,
    pending,
    config = LogicalAssignmentOperators,
    version = "next",
);

impl Rule for LogicalAssignmentOperators {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<TupleRuleConfig<Self>>(value).map(TupleRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::AssignmentExpression(assignment) => match self.0 {
                LogicalAssignmentOperatorsMode::Never => check_never(assignment, ctx),
                LogicalAssignmentOperatorsMode::Always => check_assignment(assignment, ctx),
            },
            AstKind::LogicalExpression(logical)
                if self.0 == LogicalAssignmentOperatorsMode::Always =>
            {
                check_logical(logical, ctx);
            }
            AstKind::IfStatement(if_statement)
                if self.0 == LogicalAssignmentOperatorsMode::Always
                    && self.1.enforce_for_if_statements =>
            {
                check_if_statement(if_statement, ctx);
            }
            _ => {}
        }
    }
}

fn check_never(assignment: &AssignmentExpression, ctx: &LintContext) {
    let Some(operator) = assignment.operator.to_logical_operator() else {
        return;
    };

    ctx.diagnostic(logical_assignment_operators_diagnostic(
        &LogicalAssignmentOperatorsDiagnosticKind::Unexpected,
        operator,
        assignment.span,
    ));
}

fn check_assignment(assignment: &AssignmentExpression, ctx: &LintContext) {
    if assignment.operator != AssignmentOperator::Assign {
        return;
    }

    let Expression::LogicalExpression(logical) = assignment.right.without_parentheses() else {
        return;
    };

    let leftmost_operand = get_leftmost_operand(logical);
    if !is_same_assignment_reference(&assignment.left, leftmost_operand, ctx) {
        return;
    }

    ctx.diagnostic(logical_assignment_operators_diagnostic(
        &LogicalAssignmentOperatorsDiagnosticKind::Assignment,
        logical.operator,
        assignment.span,
    ));
}

fn check_logical(logical: &LogicalExpression, ctx: &LintContext) {
    let Expression::AssignmentExpression(assignment) = logical.right.get_inner_expression() else {
        return;
    };

    if assignment.operator != AssignmentOperator::Assign
        || !is_reference(&logical.left)
        || !is_same_assignment_reference(&assignment.left, &logical.left, ctx)
    {
        return;
    }

    ctx.diagnostic(logical_assignment_operators_diagnostic(
        &LogicalAssignmentOperatorsDiagnosticKind::Logical,
        logical.operator,
        logical.span,
    ));
}

fn check_if_statement(if_statement: &IfStatement, ctx: &LintContext) {
    if if_statement.alternate.is_some() {
        return;
    }

    let body = match &if_statement.consequent {
        Statement::BlockStatement(block) if block.body.len() == 1 => &block.body[0],
        Statement::BlockStatement(_) => return,
        statement => statement,
    };

    let Statement::ExpressionStatement(expression_statement) = body else {
        return;
    };
    let Expression::AssignmentExpression(assignment) =
        expression_statement.expression.get_inner_expression()
    else {
        return;
    };

    if assignment.operator != AssignmentOperator::Assign {
        return;
    }

    let Some(existence) = get_existence(&if_statement.test, ctx) else {
        return;
    };

    if !is_same_assignment_reference(&assignment.left, existence.reference, ctx) {
        return;
    }

    ctx.diagnostic(logical_assignment_operators_diagnostic(
        &LogicalAssignmentOperatorsDiagnosticKind::If,
        existence.operator,
        if_statement.span,
    ));
}

fn get_leftmost_operand<'a>(logical: &'a LogicalExpression<'a>) -> &'a Expression<'a> {
    let mut left = &logical.left;

    while let Expression::LogicalExpression(left_logical) = left {
        if left_logical.operator != logical.operator {
            break;
        }
        left = &left_logical.left;
    }

    left
}

fn is_reference(expression: &Expression) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) => identifier.name != "undefined",
        expression => expression.as_member_expression().is_some(),
    }
}

fn is_same_assignment_reference(
    left: &AssignmentTarget,
    right: &Expression,
    ctx: &LintContext,
) -> bool {
    let Some(left) = left.as_simple_assignment_target() else {
        return false;
    };

    is_same_simple_assignment_reference(left, right, ctx)
}

fn is_same_simple_assignment_reference(
    left: &SimpleAssignmentTarget,
    right: &Expression,
    ctx: &LintContext,
) -> bool {
    let right = right.get_inner_expression();

    if let SimpleAssignmentTarget::AssignmentTargetIdentifier(left) = left {
        return matches!(right, Expression::Identifier(right) if left.name == right.name);
    }

    if let Some(left_expression) = left.get_expression() {
        return is_same_expression_reference(left_expression, right, ctx);
    }

    let Some(left_member) = left.as_member_expression() else {
        return false;
    };
    let Some(right_member) = right.as_member_expression() else {
        return false;
    };

    is_same_member_expression(left_member, right_member, ctx)
}

fn is_same_expression_reference(left: &Expression, right: &Expression, ctx: &LintContext) -> bool {
    let left = left.get_inner_expression();
    let right = right.get_inner_expression();

    match (left, right) {
        (Expression::Identifier(left), Expression::Identifier(right)) => left.name == right.name,
        (left, right) => {
            let Some(left_member) = left.as_member_expression() else {
                return false;
            };
            let Some(right_member) = right.as_member_expression() else {
                return false;
            };
            is_same_member_expression(left_member, right_member, ctx)
        }
    }
}

struct Existence<'a> {
    reference: &'a Expression<'a>,
    operator: LogicalOperator,
}

fn get_existence<'a>(expression: &'a Expression<'a>, ctx: &LintContext) -> Option<Existence<'a>> {
    let expression = expression.get_inner_expression();
    let (is_negated, base) = if let Expression::UnaryExpression(unary) = expression
        && unary.operator.is_not()
    {
        (true, unary.argument.get_inner_expression())
    } else {
        (false, expression)
    };

    if is_reference(base) {
        return Some(Existence {
            reference: base,
            operator: if is_negated { LogicalOperator::Or } else { LogicalOperator::And },
        });
    }

    if let Expression::UnaryExpression(unary) = base
        && unary.operator.is_not()
        && is_reference(&unary.argument)
    {
        return Some(Existence {
            reference: unary.argument.get_inner_expression(),
            operator: LogicalOperator::And,
        });
    }

    if let Expression::CallExpression(call_expression) = base
        && is_boolean_cast(call_expression, ctx)
        && let Some(reference) = call_expression.arguments.first().and_then(Argument::as_expression)
        && is_reference(reference)
    {
        return Some(Existence {
            reference: reference.get_inner_expression(),
            operator: if is_negated { LogicalOperator::Or } else { LogicalOperator::And },
        });
    }

    if let Some(reference) = is_implicit_nullish_comparison(expression, ctx) {
        return Some(Existence { reference, operator: LogicalOperator::Coalesce });
    }

    if let Some(reference) = is_explicit_nullish_comparison(expression, ctx) {
        return Some(Existence { reference, operator: LogicalOperator::Coalesce });
    }

    None
}

fn is_boolean_cast(call_expression: &CallExpression, ctx: &LintContext) -> bool {
    matches!(
        call_expression.callee.get_inner_expression(),
        Expression::Identifier(identifier)
            if identifier.name == "Boolean" && ctx.is_reference_to_global_variable(identifier)
    ) && call_expression.arguments.len() == 1
}

fn is_implicit_nullish_comparison<'a>(
    expression: &'a Expression<'a>,
    ctx: &LintContext,
) -> Option<&'a Expression<'a>> {
    let Expression::BinaryExpression(binary) = expression else {
        return None;
    };
    if binary.operator != BinaryOperator::Equality {
        return None;
    }

    nullish_comparison_parts(binary, ctx).map(|(reference, _)| reference)
}

fn is_explicit_nullish_comparison<'a>(
    expression: &'a Expression<'a>,
    ctx: &LintContext,
) -> Option<&'a Expression<'a>> {
    let Expression::LogicalExpression(logical) = expression else {
        return None;
    };
    if !logical.operator.is_or() {
        return None;
    }

    let Expression::BinaryExpression(left) = logical.left.get_inner_expression() else {
        return None;
    };
    let Expression::BinaryExpression(right) = logical.right.get_inner_expression() else {
        return None;
    };
    if left.operator != BinaryOperator::StrictEquality
        || right.operator != BinaryOperator::StrictEquality
    {
        return None;
    }

    let (left_reference, left_nullish) = nullish_comparison_parts(left, ctx)?;
    let (right_reference, right_nullish) = nullish_comparison_parts(right, ctx)?;

    if !is_same_expression_reference(left_reference, right_reference, ctx) {
        return None;
    }

    let left_is_null = is_null_literal(left_nullish);
    let right_is_null = is_null_literal(right_nullish);
    let left_is_undefined = is_undefined(left_nullish, ctx);
    let right_is_undefined = is_undefined(right_nullish, ctx);

    if (left_is_null && right_is_undefined) || (left_is_undefined && right_is_null) {
        Some(left_reference)
    } else {
        None
    }
}

fn nullish_comparison_parts<'a>(
    binary: &'a BinaryExpression<'a>,
    ctx: &LintContext,
) -> Option<(&'a Expression<'a>, &'a Expression<'a>)> {
    let left = binary.left.get_inner_expression();
    let right = binary.right.get_inner_expression();

    if is_reference(left) {
        Some((left, right)).filter(|(_, nullish)| is_nullish(nullish, ctx))
    } else if is_reference(right) {
        Some((right, left)).filter(|(_, nullish)| is_nullish(nullish, ctx))
    } else {
        None
    }
}

fn is_nullish(expression: &Expression, ctx: &LintContext) -> bool {
    is_null_literal(expression) || is_undefined(expression, ctx)
}

fn is_null_literal(expression: &Expression) -> bool {
    expression.get_inner_expression().is_null()
}

/// Check manually instead of `expr.is_undefined()` for cases like
/// `const undefined = 0; if (a == undefined) a = b`
fn is_undefined(expression: &Expression, ctx: &LintContext) -> bool {
    match expression.get_inner_expression() {
        Expression::Identifier(identifier) if identifier.name == "undefined" => {
            ctx.is_reference_to_global_variable(identifier)
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::Void => {
            unary.argument.get_inner_expression().is_number_0()
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("a || b", None),
        ("a && b", None),
        ("a ?? b", None),
        ("a || a || b", None),
        ("var a = a || b", None),
        ("a === undefined ? a : b", None),
        ("while (a) a = b", None),
        ("a ||= b", None),
        ("a &&= b", None),
        ("a ??= b", None),
        ("a += a || b", None),
        ("a *= a || b", None),
        ("a ||= a || b", None),
        ("a &&= a || b", None),
        ("a = a", None),
        ("a = b", None),
        ("a = a === b", None),
        ("a = a + b", None),
        ("a = a / b", None),
        ("a = fn(a) || b", None),
        ("a = false || c", None),
        ("a = f() || g()", None),
        ("a = b || c", None),
        ("a = b || a", None),
        ("object.a = object.b || c", None),
        ("[a] = a || b", None),
        ("({ a } = a || b)", None),
        ("(a = b) || a", None),
        ("a + (a = b)", None),
        ("a || (b ||= c)", None),
        ("a || (b &&= c)", None),
        ("a || b === 0", None),
        ("a || fn()", None),
        ("a || (b && c)", None),
        ("a || (b ?? c)", None),
        ("a || (b = c)", None),
        ("a || (a ||= b)", None),
        ("fn() || (a = b)", None),
        ("a.b || (a = b)", None),
        ("a?.b || (a.b = b)", None),
        ("class Class { #prop; constructor() { this.#prop || (this.prop = value) } }", None), // { "ecmaVersion": 2022 },
        ("class Class { #prop; constructor() { this.prop || (this.#prop = value) } }", None), // { "ecmaVersion": 2022 },
        ("if (a) a = b", None),
        ("if (a) a = b", Some(serde_json::json!(["always", { "enforceForIfStatements": false }]))),
        (
            "if (a) { a = b } else {}",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b } else if (a) {}",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b; else {}",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b; else if (unrelated) {}",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if (a) {}", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (a) { before; a = b }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b; after }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) throw new Error()",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if (a) a", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) a ||= b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) b = a", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) { a() }", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (a) { a += a || b }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (true) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (predicate(a)) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a?.b) a.b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (!a?.b) a.b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === b) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a != null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null && a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === 0 || a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === 1) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == null || a == undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === !0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === +0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined || a === void 0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === void void 0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === void 'string') a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === void fn()) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == b) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null == null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined == undefined) undefined = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null == x) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null == fn()) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || a === 0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (0 === a || null === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (1 === a || a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === a || 1 === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === b) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (b === undefined || a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || b === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === null || undefined === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === null || a === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === undefined || a === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === undefined || a === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{
               const undefined = 0;
               if (a == undefined) a = b
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "(() => {
               const undefined = 0;
               if (condition) {
                   if (a == undefined) a = b
               }
            })()",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{
               if (a == undefined) a = b
            }
            var undefined = 0;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{
               const undefined = 0;
               if (undefined == null) undefined = b
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{
               const undefined = 0;
               if (a === undefined || a === null) a = b
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{
               const undefined = 0;
               if (undefined === a || null === a) a = b
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if (a) b = c", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (!a) b = c", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (!!a) b = c", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (a == null) b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === undefined) b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || b === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || b === undefined) b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (Boolean(a)) b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "function fn(Boolean) {
               if (Boolean(a)) a = b
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("a = a || b", Some(serde_json::json!(["never"]))),
        ("a = a && b", Some(serde_json::json!(["never"]))),
        ("a = a ?? b", Some(serde_json::json!(["never"]))),
        ("a = b", Some(serde_json::json!(["never"]))),
        ("a += b", Some(serde_json::json!(["never"]))),
        ("a -= b", Some(serde_json::json!(["never"]))),
        ("a.b = a.b || c", Some(serde_json::json!(["never"]))),
        ("a = a && b || c", Some(serde_json::json!(["always"]))),
        ("a = a && b && c || d", Some(serde_json::json!(["always"]))),
        ("a = (a || b) || c", Some(serde_json::json!(["always"]))),
        ("a = (a && b) && c", Some(serde_json::json!(["always"]))),
        ("a = (a ?? b) ?? c", Some(serde_json::json!(["always"]))),
    ];

    let fail = vec![
        ("a = a || b", None),
        ("a = a && b", None),
        ("a = a ?? b", None),
        ("foo = foo || bar", None),
        ("a = a || fn()", None),
        ("a = a || b && c", None),
        ("a = a || (b || c)", None),
        ("a = a || (b ? c : d)", None),
        ("/* before */ a = a || b", None),
        ("a = a || b // after", None),
        ("a /* between */ = a || b", None),
        ("a = /** @type */ a || b", None),
        ("a = a || /* between */ b", None),
        ("(a) = a || b", None),
        ("a = (a) || b", None),
        ("a = a || (b)", None),
        ("a = a || ((b))", None),
        ("(a = a || b)", None),
        ("a = a || (f(), b)", None),
        ("a.b = a.b ?? c", None),
        ("a.b.c = a.b.c ?? d", None),
        ("a[b] = a[b] ?? c", None),
        ("a['b'] = a['b'] ?? c", None),
        ("a.b = a['b'] ?? c", None),
        ("a['b'] = a.b ?? c", None),
        ("this.prop = this.prop ?? {}", None),
        ("with (object) a = a || b", None),
        ("with (object) { a = a || b }", None),
        ("with (object) { if (condition) a = a || b }", None),
        ("with (a = a || b) {}", None),
        ("with (object) {} a = a || b", None),
        ("a = a || b; with (object) {}", None),
        ("if (condition) a = a || b", None),
        (
            r#"with (object) {
              "use strict";
               a = a || b
            }"#,
            None,
        ),
        ("fn(a = a || b)", None),
        ("fn((a = a || b))", None),
        ("(a = a || b) ? c : d", None),
        ("a = b = b || c", None),
        ("a || (a = b)", None),
        ("a && (a = b)", None),
        ("a ?? (a = b)", None),
        ("foo ?? (foo = bar)", None),
        ("a || (a = 0)", None),
        ("a || (a = fn())", None),
        ("a || (a = (b || c))", None),
        ("(a) || (a = b)", None),
        ("a || ((a) = b)", None),
        ("a || (a = (b))", None),
        ("a || ((a = b))", None),
        ("a || (((a = b)))", None),
        ("a || ( ( a = b ) )", None),
        ("/* before */ a || (a = b)", None),
        ("a || (a = b) // after", None),
        ("a /* between */ || (a = b)", None),
        ("a || /* between */ (a = b)", None),
        ("a.b || (a.b = c)", None),
        ("class Class { #prop; constructor() { this.#prop || (this.#prop = value) } }", None), // { "ecmaVersion": 2022 },
        ("a['b'] || (a['b'] = c)", None),
        ("a[0] || (a[0] = b)", None),
        ("a[this] || (a[this] = b)", None),
        ("foo.bar || (foo.bar = baz)", None),
        ("a.b.c || (a.b.c = d)", None),
        ("a[b.c] || (a[b.c] = d)", None),
        ("a[b?.c] || (a[b?.c] = d)", None),
        ("with (object) a.b || (a.b = c)", None),
        ("a = a.b || (a.b = {})", None),
        ("a || (a = 0) || b", None),
        ("(a || (a = 0)) || b", None),
        ("a || (b || (b = 0))", None),
        ("a = b || (b = c)", None),
        ("a || (a = 0) ? b : c", None),
        ("fn(a || (a = 0))", None),
        ("if (a) a = b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (Boolean(a)) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if (!!a) a = b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (!a) a = b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (!Boolean(a)) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined || a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === void 0) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === void 0 || a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b; }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{ const undefined = 0; }
            if (a == undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == undefined) a = b
            { const undefined = 0; }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null == a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined == a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === a || a === null) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined || null === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === a || null === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || a === undefined) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || undefined === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || undefined === a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if ((a)) a = b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) (a) = b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) a = (b)", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        ("if (a) (a = b)", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            ";if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{ if (a) (a) = b }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "fn();if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "fn()
            if (a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "id
            if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "object.prop
            if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "object[computed]
            if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "fn()
            if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = b; fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b; }
            fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b }
            fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b } fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b
            } fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("if (a) a  =  b", Some(serde_json::json!(["always", { "enforceForIfStatements": true }]))),
        (
            "if (a)
             a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) {
             a = b; 
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "/* before */ if (a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = b /* after */",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) /* between */ a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = /* between */ b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a.b) a.b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a[b]) a[b] = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a['b']) a['b'] = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (this.prop) this.prop = value",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "(class extends SuperClass { method() { if (super.prop) super.prop = value } })",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "with (object) if (a) a = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a.b === undefined || a.b === null) a.b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a.b.c) a.b.c = d",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a.b.c.d) a.b.c.d = e",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a[b].c) a[b].c = d",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "with (object) if (a.b) a.b = c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) {} else if (b) {} else if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else
            if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {
            }
            else if (a) {
            a = b;
            }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) statement; else if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) id
            else if (a) (a) = b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b; else if (c) c = d",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) { /* body */ } else if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} /* before else */ else if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else // Line
            if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else /* Block */ if (a) a = b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (array) array = array.filter(predicate)",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("a ||= b", Some(serde_json::json!(["never"]))),
        ("a &&= b", Some(serde_json::json!(["never"]))),
        ("a ??= b", Some(serde_json::json!(["never"]))),
        ("foo ||= bar", Some(serde_json::json!(["never"]))),
        ("a.b ||= c", Some(serde_json::json!(["never"]))),
        ("a[b] ||= c", Some(serde_json::json!(["never"]))),
        ("a['b'] ||= c", Some(serde_json::json!(["never"]))),
        ("this.prop ||= 0", Some(serde_json::json!(["never"]))),
        ("with (object) a ||= b", Some(serde_json::json!(["never"]))),
        ("(a) ||= b", Some(serde_json::json!(["never"]))),
        ("a ||= (b)", Some(serde_json::json!(["never"]))),
        ("(a ||= b)", Some(serde_json::json!(["never"]))),
        ("/* before */ a ||= b", Some(serde_json::json!(["never"]))),
        ("a ||= b // after", Some(serde_json::json!(["never"]))),
        ("a /* before */ ||= b", Some(serde_json::json!(["never"]))),
        ("a ||= /* after */ b", Some(serde_json::json!(["never"]))),
        ("a ||= b && c", Some(serde_json::json!(["never"]))),
        ("a &&= b || c", Some(serde_json::json!(["never"]))),
        ("a ||= b || c", Some(serde_json::json!(["never"]))),
        ("a &&= b && c", Some(serde_json::json!(["never"]))),
        ("a ??= b || c", Some(serde_json::json!(["never"]))),
        ("a ??= b && c", Some(serde_json::json!(["never"]))),
        ("a ??= b ?? c", Some(serde_json::json!(["never"]))),
        ("a ??= (b || c)", Some(serde_json::json!(["never"]))),
        ("a ??= b + c", Some(serde_json::json!(["never"]))),
        ("a ||= b as number;", Some(serde_json::json!(["never"]))), // { "parser": require( parser( "typescript-parsers/logical-assignment-with-assertion", ), ), },
        ("a.b.c || (a.b.c = d as number)", None), // { "parser": require( parser( "typescript-parsers/logical-with-assignment-with-assertion-1", ), ), },
        ("a.b.c || (a.b.c = (d as number))", None), // { "parser": require( parser( "typescript-parsers/logical-with-assignment-with-assertion-2", ), ), },
        ("(a.b.c || (a.b.c = d)) as number", None), // { "parser": require( parser( "typescript-parsers/logical-with-assignment-with-assertion-3", ), ), },
        ("a = a || b || c", Some(serde_json::json!(["always"]))),
        ("a = a && b && c", Some(serde_json::json!(["always"]))),
        ("a = a ?? b ?? c", Some(serde_json::json!(["always"]))),
        ("a = a || b && c", Some(serde_json::json!(["always"]))),
        ("a = a || b || c || d", Some(serde_json::json!(["always"]))),
        ("a = a && b && c && d", Some(serde_json::json!(["always"]))),
        ("a = a ?? b ?? c ?? d", Some(serde_json::json!(["always"]))),
        ("a = a || b || c && d", Some(serde_json::json!(["always"]))),
        ("a = a || b && c || d", Some(serde_json::json!(["always"]))),
        ("a = (a) || b || c", Some(serde_json::json!(["always"]))),
        ("a = a || (b || c) || d", Some(serde_json::json!(["always"]))),
        ("a = (a || b || c)", Some(serde_json::json!(["always"]))),
        ("a = ((a) || (b || c) || d)", Some(serde_json::json!(["always"]))),
    ];

    // TODO: Implement a suggestion for this rule
    let _fix = vec![
        ("a = a || b", "a ||= b", None),
        ("a = a && b", "a &&= b", None),
        ("a = a ?? b", "a ??= b", None),
        ("foo = foo || bar", "foo ||= bar", None),
        ("a = a || fn()", "a ||= fn()", None),
        ("a = a || b && c", "a ||= b && c", None),
        ("a = a || (b || c)", "a ||= (b || c)", None),
        ("a = a || (b ? c : d)", "a ||= (b ? c : d)", None),
        ("/* before */ a = a || b", "/* before */ a ||= b", None),
        ("a = a || b // after", "a ||= b // after", None),
        ("(a) = a || b", "(a) ||= b", None),
        ("a = (a) || b", "a ||= b", None),
        ("a = a || (b)", "a ||= (b)", None),
        ("a = a || ((b))", "a ||= ((b))", None),
        ("(a = a || b)", "(a ||= b)", None),
        ("a = a || (f(), b)", "a ||= (f(), b)", None),
        ("with (a = a || b) {}", "with (a ||= b) {}", None),
        ("with (object) {} a = a || b", "with (object) {} a ||= b", None),
        ("a = a || b; with (object) {}", "a ||= b; with (object) {}", None),
        ("if (condition) a = a || b", "if (condition) a ||= b", None),
        ("fn(a = a || b)", "fn(a ||= b)", None),
        ("fn((a = a || b))", "fn((a ||= b))", None),
        ("(a = a || b) ? c : d", "(a ||= b) ? c : d", None),
        ("a = b = b || c", "a = b ||= c", None),
        ("a || (a = b)", "a ||= b", None),
        ("a && (a = b)", "a &&= b", None),
        ("a ?? (a = b)", "a ??= b", None),
        ("foo ?? (foo = bar)", "foo ??= bar", None),
        ("a || (a = 0)", "a ||= 0", None),
        ("a || (a = fn())", "a ||= fn()", None),
        ("a || (a = (b || c))", "a ||= (b || c)", None),
        ("(a) || (a = b)", "a ||= b", None),
        ("a || ((a) = b)", "(a) ||= b", None),
        ("a || (a = (b))", "a ||= (b)", None),
        ("a || ((a = b))", "a ||= b", None),
        ("a || (((a = b)))", "a ||= b", None),
        ("a || ( ( a = b ) )", "a ||= b", None),
        ("/* before */ a || (a = b)", "/* before */ a ||= b", None),
        ("a || (a = b) // after", "a ||= b // after", None),
        ("a.b || (a.b = c)", "a.b ||= c", None),
        (
            "class Class { #prop; constructor() { this.#prop || (this.#prop = value) } }",
            "class Class { #prop; constructor() { this.#prop ||= value } }",
            None,
        ),
        ("a['b'] || (a['b'] = c)", "a['b'] ||= c", None),
        ("a[0] || (a[0] = b)", "a[0] ||= b", None),
        ("a[this] || (a[this] = b)", "a[this] ||= b", None),
        ("foo.bar || (foo.bar = baz)", "foo.bar ||= baz", None),
        ("a = a.b || (a.b = {})", "a = a.b ||= {}", None),
        ("a || (a = 0) || b", "(a ||= 0) || b", None),
        ("(a || (a = 0)) || b", "(a ||= 0) || b", None),
        ("a || (b || (b = 0))", "a || (b ||= 0)", None),
        ("a = b || (b = c)", "a = b ||= c", None),
        ("a || (a = 0) ? b : c", "(a ||= 0) ? b : c", None),
        ("fn(a || (a = 0))", "fn(a ||= 0)", None),
        (
            "if (a) a = b",
            "a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (Boolean(a)) a = b",
            "a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (!!a) a = b",
            "a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (!a) a = b",
            "a ||= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (!Boolean(a)) a = b",
            "a ||= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == undefined) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a == null) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === undefined) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined || a === null) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || a === void 0) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === void 0 || a === null) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b; }",
            "a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null == a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined == a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === a || a === null) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === undefined || null === a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (undefined === a || null === a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || a === undefined) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a === null || undefined === a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (null === a || undefined === a) a = b",
            "a ??= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if ((a)) a = b",
            "a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) (a) = b",
            "(a) &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = (b)",
            "a &&= (b)",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) (a = b)",
            "(a &&= b)",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            ";if (a) (a) = b",
            ";(a) &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "{ if (a) (a) = b }",
            "{ (a) &&= b }",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "fn();if (a) (a) = b",
            "fn();(a) &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "fn()
            if (a) a = b",
            "fn()
            a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = b; fn();",
            "a &&= b; fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b }",
            "a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b; }
            fn();",
            "a &&= b;
            fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b }
            fn();",
            "a &&= b;
            fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b } fn();",
            "a &&= b; fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) { a = b
            } fn();",
            "a &&= b; fn();",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a  =  b",
            "a  &&=  b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a)
             a = b",
            "a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) {
             a = b; 
            }",
            "a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "/* before */ if (a) a = b",
            "/* before */ a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) a = b /* after */",
            "a &&= b /* after */",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a.b) a.b = c",
            "a.b &&= c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a[b]) a[b] = c",
            "a[b] &&= c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a['b']) a['b'] = c",
            "a['b'] &&= c",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (this.prop) this.prop = value",
            "this.prop &&= value",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "(class extends SuperClass { method() { if (super.prop) super.prop = value } })",
            "(class extends SuperClass { method() { super.prop &&= value } })",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "with (object) if (a) a = b",
            "with (object) a &&= b",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b;",
            "if (unrelated) {} else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (a) {} else if (b) {} else if (a) a = b;",
            "if (a) {} else if (b) {} else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else
            if (a) a = b;",
            "if (unrelated) {} else
            a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {
            }
            else if (a) {
            a = b;
            }",
            "if (unrelated) {
            }
            else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) statement; else if (a) a = b;",
            "if (unrelated) statement; else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else if (a) a = b; else if (c) c = d",
            "if (unrelated) {} else if (a) a = b; else c &&= d",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) { /* body */ } else if (a) a = b;",
            "if (unrelated) { /* body */ } else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} /* before else */ else if (a) a = b;",
            "if (unrelated) {} /* before else */ else a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else // Line
            if (a) a = b;",
            "if (unrelated) {} else // Line
            a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (unrelated) {} else /* Block */ if (a) a = b;",
            "if (unrelated) {} else /* Block */ a &&= b;",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        (
            "if (array) array = array.filter(predicate)",
            "array &&= array.filter(predicate)",
            Some(serde_json::json!(["always", { "enforceForIfStatements": true }])),
        ),
        ("a ||= b", "a = a || b", Some(serde_json::json!(["never"]))),
        ("a &&= b", "a = a && b", Some(serde_json::json!(["never"]))),
        ("a ??= b", "a = a ?? b", Some(serde_json::json!(["never"]))),
        ("foo ||= bar", "foo = foo || bar", Some(serde_json::json!(["never"]))),
        ("(a) ||= b", "(a) = a || b", Some(serde_json::json!(["never"]))),
        ("a ||= (b)", "a = a || (b)", Some(serde_json::json!(["never"]))),
        ("(a ||= b)", "(a = a || b)", Some(serde_json::json!(["never"]))),
        ("/* before */ a ||= b", "/* before */ a = a || b", Some(serde_json::json!(["never"]))),
        ("a ||= b // after", "a = a || b // after", Some(serde_json::json!(["never"]))),
        ("a ||= b && c", "a = a || b && c", Some(serde_json::json!(["never"]))),
        ("a &&= b || c", "a = a && (b || c)", Some(serde_json::json!(["never"]))),
        ("a ||= b || c", "a = a || (b || c)", Some(serde_json::json!(["never"]))),
        ("a &&= b && c", "a = a && (b && c)", Some(serde_json::json!(["never"]))),
        ("a ??= b || c", "a = a ?? (b || c)", Some(serde_json::json!(["never"]))),
        ("a ??= b && c", "a = a ?? (b && c)", Some(serde_json::json!(["never"]))),
        ("a ??= b ?? c", "a = a ?? (b ?? c)", Some(serde_json::json!(["never"]))),
        ("a ??= (b || c)", "a = a ?? (b || c)", Some(serde_json::json!(["never"]))),
        ("a ??= b + c", "a = a ?? b + c", Some(serde_json::json!(["never"]))),
        ("a ||= b as number;", "a = a || (b as number);", Some(serde_json::json!(["never"]))),
        ("a = a || b || c", "a ||= b || c", Some(serde_json::json!(["always"]))),
        ("a = a && b && c", "a &&= b && c", Some(serde_json::json!(["always"]))),
        ("a = a ?? b ?? c", "a ??= b ?? c", Some(serde_json::json!(["always"]))),
        ("a = a || b && c", "a ||= b && c", Some(serde_json::json!(["always"]))),
        ("a = a || b || c || d", "a ||= b || c || d", Some(serde_json::json!(["always"]))),
        ("a = a && b && c && d", "a &&= b && c && d", Some(serde_json::json!(["always"]))),
        ("a = a ?? b ?? c ?? d", "a ??= b ?? c ?? d", Some(serde_json::json!(["always"]))),
        ("a = a || b || c && d", "a ||= b || c && d", Some(serde_json::json!(["always"]))),
        ("a = a || b && c || d", "a ||= b && c || d", Some(serde_json::json!(["always"]))),
        ("a = (a) || b || c", "a ||= b || c", Some(serde_json::json!(["always"]))),
        ("a = a || (b || c) || d", "a ||= (b || c) || d", Some(serde_json::json!(["always"]))),
        ("a = (a || b || c)", "a ||= (b || c)", Some(serde_json::json!(["always"]))),
        (
            "a = ((a) || (b || c) || d)",
            "a ||= ((b || c) || d)",
            Some(serde_json::json!(["always"])),
        ),
    ];

    Tester::new(LogicalAssignmentOperators::NAME, LogicalAssignmentOperators::PLUGIN, pass, fail)
        .test_and_snapshot();
}
