use oxc_ast::{
    AstKind,
    ast::{
        BinaryExpression, BinaryOperator, Expression, LogicalExpression, LogicalOperator,
        UnaryOperator,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToBigInt, WithoutGlobalReferenceInformation};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::is_same_expression,
};

fn yoda_diagnostic(span: Span, never: bool, operator: &str) -> OxcDiagnostic {
    let expected_side = if never { "right" } else { "left" };
    OxcDiagnostic::warn("Require or disallow \"Yoda\" conditions")
        .with_help(format!("Expected literal to be on the {expected_side} side of {operator}."))
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Yoda(AllowYoda, YodaOptions);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct YodaOptions {
    /// If the `"exceptRange"` property is `true`, the rule *allows* yoda conditions
    /// in range comparisons which are wrapped directly in parentheses, including the
    /// parentheses of an `if` or `while` condition.
    /// A *range* comparison tests whether a variable is inside or outside the range
    /// between two literal values.
    except_range: bool,
    /// If the `"onlyEquality"` property is `true`, the rule reports yoda
    /// conditions *only* for the equality operators `==` and `===`. The `onlyEquality`
    /// option allows a superset of the exceptions which `exceptRange` allows, thus
    /// both options are not useful together.
    only_equality: bool,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum AllowYoda {
    /// The default `"never"` option can have exception options in an object literal, via `exceptRange` and `onlyEquality`.
    #[default]
    Never,
    /// The `"always"` option requires that literal values must always come first in comparisons.
    Always,
}

impl Default for Yoda {
    fn default() -> Self {
        Self(AllowYoda::Never, YodaOptions { except_range: false, only_equality: false })
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Require or disallow "Yoda" conditions.
    /// This rule aims to enforce consistent style of conditions which compare a variable to a literal value.
    ///
    /// ### Why is this bad?
    ///
    /// Yoda conditions are so named because the literal value of the condition comes first while the variable comes second. For example, the following is a Yoda condition:
    /// ```js
    /// if ("red" === color) {
    //     // ...
    /// }
    /// ```
    ///
    /// This is called a Yoda condition because it reads as, "if red equals the color", similar to the way the Star Wars character Yoda speaks. Compare to the other way of arranging the operands:
    ///
    /// ```js
    /// if (color === "red") {
    ///     // ...
    /// }
    /// ```
    ///
    /// This typically reads, "if the color equals red", which is arguably a more natural way to describe the comparison.
    /// Proponents of Yoda conditions highlight that it is impossible to mistakenly use `=` instead of `==` because you cannot assign to a literal value. Doing so will cause a syntax error and you will be informed of the mistake early on. This practice was therefore very common in early programming where tools were not yet available.
    /// Opponents of Yoda conditions point out that tooling has made us better programmers because tools will catch the mistaken use of `=` instead of `==` (ESLint will catch this for you). Therefore, they argue, the utility of the pattern doesn't outweigh the readability hit the code takes while using Yoda conditions.
    ///
    /// ### Examples
    ///
    /// #### never
    ///
    /// Examples of **incorrect** code for the default `"never"` option:
    /// ```js
    /// if ("red" === color) {
    ///     // ...
    /// }
    /// if (`red` === color) {
    ///      // ...
    /// }
    /// if (`red` === `${color}`) {
    ///     // ...
    /// }
    ///
    /// if (true == flag) {
    ///    // ...
    /// }
    ///
    // if (5 > count) {
    //     // ...
    // }
    ///
    // if (-1 < str.indexOf(substr)) {
    //     // ...
    // }
    ///
    /// if (0 <= x && x < 1) {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of **correct** code for the default `"never"` option:
    /// ```js
    /// if (5 & value) {
    ///     // ...
    /// }
    ///
    /// if (value === "red") {
    ///     // ...
    /// }
    ///
    /// if (value === `red`) {
    ///     // ...
    /// }
    ///
    /// if (`${value}` === `red`) {
    ///
    /// }
    /// ```
    ///
    /// #### exceptRange
    ///
    /// Examples of **correct** code for the `"never", { "exceptRange": true }` options:
    ///
    /// ```js
    /// function isReddish(color) {
    ///     return (color.hue < 60 || 300 < color.hue);
    /// }
    ///
    /// if (x < -1 || 1 < x) {
    ///     // ...
    /// }
    ///
    /// if (count < 10 && (0 <= rand && rand < 1)) {
    ///     // ...
    /// }
    ///
    /// if (`blue` < x && x < `green`) {
    ///     // ...
    /// }
    ///
    /// function howLong(arr) {
    ///     return (0 <= arr.length && arr.length < 10) ? "short" : "long";
    /// }
    /// ```
    ///
    /// #### onlyEquality
    ///
    /// Examples of **correct** code for the `"never", { "onlyEquality": true }` options:
    /// ```js
    /// if (x < -1 || 9 < x) {
    /// }
    ///
    /// if (x !== 'foo' && 'bar' != x) {
    /// }
    ///
    /// if (x !== `foo` && `bar` != x) {
    /// }
    /// ```
    ///
    /// #### always
    ///
    /// Examples of **incorrect** code for the `"always"` option:
    /// ```js
    /// if (color == "blue") {
    ///     // ...
    /// }
    ///
    /// if (color == `blue`) {
    ///     // ...
    /// }
    /// ```
    ///
    /// Examples of **correct** code for the `"always"` option:
    /// ```js
    /// if ("blue" == value) {
    ///     // ...
    /// }
    ///
    /// if (`blue` == value) {
    ///     // ...
    /// }
    ///
    /// if (`blue` == `${value}`) {
    ///     // ...
    /// }
    ///
    /// if (-1 < str.indexOf(substr)) {
    ///     // ...
    /// }
    /// ```
    Yoda,
    eslint,
    style,
    fix,
    config = Yoda,
);

impl Rule for Yoda {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<Yoda>>(value).unwrap_or_default().into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let Yoda(allow_yoda, options) = self;

        let AstKind::BinaryExpression(expr) = node.kind() else {
            return;
        };

        let parent_node = ctx.nodes().parent_node(node.id());
        if let AstKind::LogicalExpression(logical_expr) = parent_node.kind() {
            let parent_logical_expr = ctx.nodes().parent_node(parent_node.id());

            if options.except_range
                && is_parenthesized(parent_logical_expr)
                && is_range(logical_expr, ctx)
            {
                return;
            }
        }

        if !expr.operator.is_equality() && !expr.operator.is_compare() {
            return;
        }

        if options.only_equality && !is_equality(expr) {
            return;
        }

        // never
        if allow_yoda == &AllowYoda::Never && is_yoda(expr) {
            do_diagnostic_with_fix(expr, ctx, true);
        }

        // always
        if allow_yoda == &AllowYoda::Always && is_not_yoda(expr) {
            do_diagnostic_with_fix(expr, ctx, false);
        }
    }
}

fn is_yoda(expr: &BinaryExpression) -> bool {
    is_literal_or_simple_template_literal(expr.left.get_inner_expression())
        && !is_literal_or_simple_template_literal(expr.right.get_inner_expression())
}

fn is_not_yoda(expr: &BinaryExpression) -> bool {
    !is_literal_or_simple_template_literal(expr.left.get_inner_expression())
        && is_literal_or_simple_template_literal(expr.right.get_inner_expression())
}

#[expect(clippy::cast_possible_truncation)]
fn do_diagnostic_with_fix(expr: &BinaryExpression, ctx: &LintContext, never: bool) {
    ctx.diagnostic_with_fix(yoda_diagnostic(expr.span, never, expr.operator.as_str()), |fix| {
        let left_span = expr.left.span();
        let right_span = expr.right.span();

        let operator_str = expr.operator.as_str();
        let str_between_left_and_right = ctx.source_range(
            Span::new(left_span.end, right_span.start)
        );

        let (operator_start, operator_end) = str_between_left_and_right
            .as_bytes()
            .windows(operator_str.len())
            .enumerate()
            .find_map(|(index, chunk)| {
                if chunk == operator_str.as_bytes() {
                    let pos_start = index as u32 + left_span.end;
                    let pos_end = pos_start + operator_str.len() as u32;
                    if !ctx.comments().iter().any(|comment| comment.span.start <= pos_start && pos_end <= comment.span.end) {
                        return Some((pos_start, pos_end));
                    }
                }
                None
            })
            .unwrap();

        let str_between_left_and_operator = ctx.source_range(Span::new(left_span.end, operator_start));
        let str_between_operator_and_right = ctx.source_range(Span::new(operator_end, right_span.start));

        let left_prev_token = if left_span.start > 0 && (expr.right.is_literal() || expr.right.is_identifier_reference() || is_keyword_expression(&expr.right)) {
            let tokens = ctx.source_range(Span::new(0, left_span.start));
            let token = tokens.chars().last();
            match_token(token)
        } else {
            false
        };

        let source_size = u32::try_from(ctx.source_text().len()).unwrap();
        let right_next_token = if right_span.end < source_size && (expr.left.is_literal() || expr.left.is_identifier_reference()) {
            let tokens = ctx.source_range(Span::new(right_span.end, source_size));
            let token = tokens.chars().next();
            match_token(token)
        } else {
            false
        };

        let left_str = ctx.source_range(left_span);
        let right_str = ctx.source_range(right_span);
        let flipped_operator_str = flip_operator(expr.operator).as_str();
        let replacement = format!(
            "{}{right_str}{str_between_left_and_operator}{flipped_operator_str}{str_between_operator_and_right}{left_str}{}",
            if left_prev_token { " " } else { "" },
            if right_next_token { " " } else { "" }
        );

        fix.replace(expr.span, replacement)
    });
}

fn match_token(token: Option<char>) -> bool {
    !matches!(token, Some(' ' | '(' | ')' | '{' | '}' | '/' | '=' | ';'))
}

/// Returns `true` if the expression starts with a keyword (typeof, void, delete,
/// await, yield, new) that needs a space separator.
fn is_keyword_expression(expr: &Expression) -> bool {
    matches!(
        expr,
        Expression::UnaryExpression(unary)
            if matches!(unary.operator, UnaryOperator::Typeof | UnaryOperator::Void | UnaryOperator::Delete)
    ) || matches!(
        expr,
        Expression::AwaitExpression(_)
            | Expression::YieldExpression(_)
            | Expression::NewExpression(_)
    )
}

fn flip_operator(operator: BinaryOperator) -> BinaryOperator {
    match operator {
        BinaryOperator::LessThan => BinaryOperator::GreaterThan,
        BinaryOperator::LessEqualThan => BinaryOperator::GreaterEqualThan,
        BinaryOperator::GreaterThan => BinaryOperator::LessThan,
        BinaryOperator::GreaterEqualThan => BinaryOperator::LessEqualThan,
        _ => operator,
    }
}

fn is_equality(expr: &BinaryExpression) -> bool {
    expr.operator == BinaryOperator::Equality || expr.operator == BinaryOperator::StrictEquality
}

fn is_parenthesized(parent_logical_expr: &AstNode) -> bool {
    let kind = parent_logical_expr.kind();

    matches!(kind, AstKind::ParenthesizedExpression(_))
        || matches!(kind, AstKind::IfStatement(_))
        || matches!(kind, AstKind::WhileStatement(_))
        || matches!(kind, AstKind::DoWhileStatement(_))
}

fn is_range(expr: &LogicalExpression, ctx: &LintContext) -> bool {
    let Expression::BinaryExpression(left) = &expr.left else {
        return false;
    };
    let Expression::BinaryExpression(right) = &expr.right else {
        return false;
    };

    match left.operator {
        BinaryOperator::LessThan | BinaryOperator::LessEqualThan => {}
        _ => return false,
    }

    match right.operator {
        BinaryOperator::LessThan | BinaryOperator::LessEqualThan => {}
        _ => return false,
    }

    if expr.operator == LogicalOperator::And {
        if !is_same_expression(&left.right, &right.left, ctx) {
            return false;
        }

        let left_left = &left.left;
        let right_right = &right.right;

        let is_left_left_target_literal = is_target_literal(left_left);
        let is_right_right_target_literal = is_target_literal(right_right);

        if !is_left_left_target_literal && !is_right_right_target_literal {
            return false;
        }

        if !is_left_left_target_literal || !is_right_right_target_literal {
            return true;
        }

        if let (Some(left_left), Some(right_right)) =
            (get_string_literal(left_left), get_string_literal(right_right))
        {
            return left_left <= right_right;
        }

        if let (Some(left_left), Some(right_right)) =
            (get_number(left_left), get_number(right_right))
        {
            return left_left <= right_right;
        }

        return false;
    }

    if expr.operator == LogicalOperator::Or {
        if !is_same_expression(&left.left, &right.right, ctx) {
            return false;
        }

        let left_right = &left.right;
        let right_left = &right.left;

        let is_left_right_target_literal = is_target_literal(left_right);
        let is_right_left_target_literal = is_target_literal(right_left);

        if !is_left_right_target_literal && !is_right_left_target_literal {
            return false;
        }

        if !is_left_right_target_literal || !is_right_left_target_literal {
            return true;
        }

        if let (Some(left_right), Some(right_left)) =
            (get_string_literal(left_right), get_string_literal(right_left))
        {
            return left_right <= right_left;
        }

        if let (Some(left_right), Some(right_left)) =
            (get_number(left_right), get_number(right_left))
        {
            return left_right <= right_left;
        }

        return false;
    }

    false
}

fn is_simple_template_literal(expr: &Expression) -> bool {
    match expr {
        Expression::TemplateLiteral(template) => template.quasis.len() == 1,
        _ => false,
    }
}

fn is_literal_or_simple_template_literal(expr: &Expression) -> bool {
    expr.is_literal() || is_number(expr) || is_simple_template_literal(expr)
}

fn is_target_literal(expr: &Expression) -> bool {
    get_string_literal(expr).is_some() || is_number(expr)
}

fn get_string_literal<'a>(expr: &'a Expression) -> Option<&'a str> {
    match expr {
        Expression::StringLiteral(string) => Some(&string.value),
        Expression::TemplateLiteral(template) => {
            if template.quasis.len() != 1 {
                return None;
            }

            template.quasis.first().map(|e| e.value.raw.as_str())
        }
        _ => None,
    }
}

fn is_number(expr: &Expression) -> bool {
    match expr {
        Expression::NumericLiteral(_) | Expression::BigIntLiteral(_) => true,
        Expression::UnaryExpression(unary) => {
            if unary.operator == UnaryOperator::UnaryNegation {
                return is_number(&unary.argument);
            }
            false
        }
        _ => false,
    }
}

fn get_number(expr: &Expression) -> Option<f64> {
    match expr {
        Expression::NumericLiteral(numeric) => Some(numeric.value),
        Expression::BigIntLiteral(big_int) => {
            let big_int = big_int.to_big_int(&WithoutGlobalReferenceInformation {})?;

            let Ok(big_int) = big_int.to_string().parse::<f64>() else {
                return None;
            };

            Some(big_int)
        }
        Expression::UnaryExpression(unary) => {
            if unary.operator == UnaryOperator::UnaryNegation {
                return get_number(&unary.argument).map(|num| -num);
            }

            None
        }
        _ => None,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"if (value === "red") {}"#, Some(serde_json::json!(["never"]))),
        ("if (value === value) {}", Some(serde_json::json!(["never"]))),
        ("if (value != 5) {}", Some(serde_json::json!(["never"]))),
        ("if (5 & foo) {}", Some(serde_json::json!(["never"]))),
        ("if (5 === 4) {}", Some(serde_json::json!(["never"]))),
        ("if (value === `red`) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("if (`red` === `red`) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("if (`${foo}` === `red`) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (r#"if (`${""}` === `red`) {}"#, Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (r#"if (`${"red"}` === foo) {}"#, Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("if (b > `a` && b > `a`) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (r#"if (`b` > `a` && "b" > "a") {}"#, Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (r#"if ("blue" === value) {}"#, Some(serde_json::json!(["always"]))),
        ("if (value === value) {}", Some(serde_json::json!(["always"]))),
        ("if (4 != value) {}", Some(serde_json::json!(["always"]))),
        ("if (foo & 4) {}", Some(serde_json::json!(["always"]))),
        ("if (5 === 4) {}", Some(serde_json::json!(["always"]))),
        ("if (`red` === value) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("if (`red` === `red`) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("if (`red` === `${foo}`) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (r#"if (`red` === `${""}`) {}"#, Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (r#"if (foo === `${"red"}`) {}"#, Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("if (`a` > b && `a` > b) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (r#"if (`b` > `a` && "b" > "a") {}"#, Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (
            r#"if ("a" < x && x < MAX ) {}"#,
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("if (1 < x && x < MAX ) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        (
            "if ('a' < x && x < MAX ) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (x < `x` || `x` <= x) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        ("if (0 < x && x <= 1) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("if (0 <= x && x < 1) {}", Some(serde_json::json!(["always", { "exceptRange": true }]))),
        (
            "if ('blue' < x.y && x.y < 'green') {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 < x[``] && x[``] < 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 < x[''] && x[``] < 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (a < 4 || (b[c[0]].d['e'] < 0 || 1 <= b[c[0]].d['e'])) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= x['y'] && x['y'] <= 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (a < 0 && (0 < b && b < 1)) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if ((0 < a && a < 1) && b < 0) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("if (-1 < x && x < 0) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        (
            "if (0 <= this.prop && this.prop <= 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= index && index < list.length) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (ZERO <= index && index < 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (value <= MIN || 10 < value) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (value <= 0 || MAX < value) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            r#"if (0 <= a.b && a["b"] <= 100) {}"#,
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a.b && a[`b`] <= 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        ("if (-1n < x && x <= 1n) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))), // { "ecmaVersion": 2020 },
        (
            "if (-1n <= x && x < 1n) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "if (x < `1` || `1` < x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2020 },
        (
            "if (1 <= a['/(?<zero>0)/'] && a[/(?<zero>0)/] <= 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2018 },
        (
            "if (x <= `bar` || `foo` < x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if ('a' < x && x < MAX ) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        ("if ('a' < x && x < MAX ) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (
            "if (MIN < x && x < 'a' ) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        ("if (MIN < x && x < 'a' ) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (
            "if (`blue` < x.y && x.y < `green`) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 <= x[`y`] && x[`y`] <= 100) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            r#"if (0 <= x[`y`] && x["y"] <= 100) {}"#,
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if ('a' <= x && x < 'b') {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("if (x < -1n || 1n <= x) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))), // { "ecmaVersion": 2020 },
        (
            "if (x < -1n || 1n <= x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2020 },
        ("if (1 < a && a <= 2) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("if (x < -1 || 1 < x) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        (
            "if (x <= 'bar' || 'foo' < x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        ("if (x < 0 || 1 <= x) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("if('a' <= x && x < MAX) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        (
            "if (0 <= obj?.a && obj?.a < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2020 },
        ("if (0 < x && x <= 1) {}", Some(serde_json::json!(["never", { "onlyEquality": true }]))),
        (
            "if (x !== 'foo' && 'foo' !== x) {}",
            Some(serde_json::json!(["never", { "onlyEquality": true }])),
        ),
        (
            "if (x < 2 && x !== -3) {}",
            Some(serde_json::json!(["always", { "onlyEquality": true }])),
        ),
        (
            "if (x !== `foo` && `foo` !== x) {}",
            Some(serde_json::json!(["never", { "onlyEquality": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (x < `2` && x !== `-3`) {}",
            Some(serde_json::json!(["always", { "onlyEquality": true }])),
        ), // { "ecmaVersion": 2015 }
    ];

    let fail = vec![
        (
            "if (x <= 'foo' || 'bar' < x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (r#"if ("red" == value) {}"#, Some(serde_json::json!(["never"]))),
        ("if (true === value) {}", Some(serde_json::json!(["never"]))),
        ("if (5 != value) {}", Some(serde_json::json!(["never"]))),
        ("if (5n != value) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2020 },
        ("if (null !== value) {}", Some(serde_json::json!(["never"]))),
        (r#"if ("red" <= value) {}"#, Some(serde_json::json!(["never"]))),
        ("if (`red` <= value) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("if (`red` <= `${foo}`) {}", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        (r#"if (`red` <= `${"red"}`) {}"#, Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("if (true >= value) {}", Some(serde_json::json!(["never"]))),
        ("var foo = (5 < value) ? true : false", Some(serde_json::json!(["never"]))),
        ("function foo() { return (null > value); }", Some(serde_json::json!(["never"]))),
        ("if (-1 < str.indexOf(substr)) {}", Some(serde_json::json!(["never"]))),
        (r#"if (value == "red") {}"#, Some(serde_json::json!(["always"]))),
        ("if (value == `red`) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("if (value === true) {}", Some(serde_json::json!(["always"]))),
        ("if (value === 5n) {}", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2020 },
        (r#"if (`${"red"}` <= `red`) {}"#, Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        (
            "if (a < 0 && 0 <= b && b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a && a < 1 && b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("if (1 < a && a < 0) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("0 < a && a < 1", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("var a = b < 0 || 1 <= b;", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("if (0 <= x && x < -1) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        (
            "var a = (b < 0 && 0 <= b);",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (
            "var a = (b < `0` && `0` <= b);",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (`green` < x.y && x.y < `blue`) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 <= a[b] && a['b'] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[b] && a[`b`] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (`0` <= a[b] && a[`b`] < `1`) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 <= a[b] && a.b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a.b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[' '] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[null] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[``] && a[null] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 <= a[''] && a[b] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[``] && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "if (0 <= a[b()] && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a.null && a[/(?<zero>0)/] <= 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2018 },
        ("if (3 == a) {}", Some(serde_json::json!(["never", { "onlyEquality": true }]))),
        ("foo(3 === a);", Some(serde_json::json!(["never", { "onlyEquality": true }]))),
        ("foo(a === 3);", Some(serde_json::json!(["always", { "onlyEquality": true }]))),
        ("foo(a === `3`);", Some(serde_json::json!(["always", { "onlyEquality": true }]))), // { "ecmaVersion": 2015 },
        ("if (0 <= x && x < 1) {}", None),
        ("if ( /* a */ 0 /* b */ < /* c */ foo /* d */ ) {}", Some(serde_json::json!(["never"]))),
        ("if ( /* a */ foo /* b */ > /* c */ 0 /* d */ ) {}", Some(serde_json::json!(["always"]))),
        ("if (foo()===1) {}", Some(serde_json::json!(["always"]))),
        ("if (foo()     === 1) {}", Some(serde_json::json!(["always"]))),
        ("while (0 === (a));", Some(serde_json::json!(["never"]))),
        ("while (0 === (a = b));", Some(serde_json::json!(["never"]))),
        ("while ((a) === 0);", Some(serde_json::json!(["always"]))),
        ("while ((a = b) === 0);", Some(serde_json::json!(["always"]))),
        ("if (((((((((((foo)))))))))) === ((((((5)))))));", Some(serde_json::json!(["always"]))),
        ("function *foo() { yield(1) < a }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield((1)) < a }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield 1 < a }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield/**/1 < a }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield(1) < ++a }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield(1) < (a) }", Some(serde_json::json!(["never"]))), // { "ecmaVersion": 2015 },
        ("x=1 < a", Some(serde_json::json!(["never"]))),
        ("function *foo() { yield++a < 1 }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield(a) < 1 }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield a < 1 }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield/**/a < 1 }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("function *foo() { yield++a < (1) }", Some(serde_json::json!(["always"]))), // { "ecmaVersion": 2015 },
        ("x=a < 1", Some(serde_json::json!(["always"]))),
        ("0 < f()in obj", None),
        ("1 > x++instanceof foo", Some(serde_json::json!(["never"]))),
        ("x < ('foo')in bar", Some(serde_json::json!(["always"]))),
        ("false <= ((x))in foo", Some(serde_json::json!(["never"]))),
        ("x >= (1)instanceof foo", Some(serde_json::json!(["always"]))),
        ("false <= ((x)) in foo", Some(serde_json::json!(["never"]))),
        ("x >= 1 instanceof foo", Some(serde_json::json!(["always"]))),
        ("x >= 1/**/instanceof foo", Some(serde_json::json!(["always"]))),
        ("(x >= 1)instanceof foo", Some(serde_json::json!(["always"]))),
        ("(x) >= (1)instanceof foo", Some(serde_json::json!(["always"]))),
        ("1 > x===foo", Some(serde_json::json!(["never"]))),
        ("1 > x", Some(serde_json::json!(["never"]))),
        (
            "if (`green` < x.y && x.y < `blue`) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ), // { "ecmaVersion": 2015 },
        ("if('a' <= x && x < 'b') {}", Some(serde_json::json!(["always"]))),
        (
            "if ('b' <= x && x < 'a') {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("if('a' <= x && x < 1) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        ("if (0 < a && b < max) {}", Some(serde_json::json!(["never", { "exceptRange": true }]))),
        // Issue: <https://github.com/oxc-project/oxc/issues/7714>
        ("{( t=='' )}", Some(serde_json::json!(["always", { "onlyEquality": true }]))),
    ];

    let fix = vec![
        (
            "if (x <= 'foo' || 'bar' < x) {}",
            "if ('foo' >= x || 'bar' < x) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (
            r#"if ("red" == value) {}"#,
            r#"if (value == "red") {}"#,
            Some(serde_json::json!(["never"])),
        ),
        ("if (true === value) {}", "if (value === true) {}", Some(serde_json::json!(["never"]))),
        ("if (5 != value) {}", "if (value != 5) {}", Some(serde_json::json!(["never"]))),
        ("if (5n != value) {}", "if (value != 5n) {}", Some(serde_json::json!(["never"]))),
        ("if (null !== value) {}", "if (value !== null) {}", Some(serde_json::json!(["never"]))),
        (
            r#"if ("red" <= value) {}"#,
            r#"if (value >= "red") {}"#,
            Some(serde_json::json!(["never"])),
        ),
        ("if (`red` <= value) {}", "if (value >= `red`) {}", Some(serde_json::json!(["never"]))),
        (
            "if (`red` <= `${foo}`) {}",
            "if (`${foo}` >= `red`) {}",
            Some(serde_json::json!(["never"])),
        ),
        (
            r#"if (`red` <= `${"red"}`) {}"#,
            r#"if (`${"red"}` >= `red`) {}"#,
            Some(serde_json::json!(["never"])),
        ),
        ("if (true >= value) {}", "if (value <= true) {}", Some(serde_json::json!(["never"]))),
        (
            "var foo = (5 < value) ? true : false",
            "var foo = (value > 5) ? true : false",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function foo() { return (null > value); }",
            "function foo() { return (value < null); }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "if (-1 < str.indexOf(substr)) {}",
            "if (str.indexOf(substr) > -1) {}",
            Some(serde_json::json!(["never"])),
        ),
        (
            r#"if (value == "red") {}"#,
            r#"if ("red" == value) {}"#,
            Some(serde_json::json!(["always"])),
        ),
        ("if (value == `red`) {}", "if (`red` == value) {}", Some(serde_json::json!(["always"]))),
        ("if (value === true) {}", "if (true === value) {}", Some(serde_json::json!(["always"]))),
        ("if (value === 5n) {}", "if (5n === value) {}", Some(serde_json::json!(["always"]))),
        (
            r#"if (`${"red"}` <= `red`) {}"#,
            r#"if (`red` >= `${"red"}`) {}"#,
            Some(serde_json::json!(["always"])),
        ),
        (
            "if (a < 0 && 0 <= b && b < 1) {}",
            "if (a < 0 && b >= 0 && b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a && a < 1 && b < 1) {}",
            "if (a >= 0 && a < 1 && b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (1 < a && a < 0) {}",
            "if (a > 1 && a < 0) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "0 < a && a < 1",
            "a > 0 && a < 1",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "var a = b < 0 || 1 <= b;",
            "var a = b < 0 || b >= 1;",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= x && x < -1) {}",
            "if (x >= 0 && x < -1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "var a = (b < 0 && 0 <= b);",
            "var a = (0 > b && 0 <= b);",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (
            "var a = (b < `0` && `0` <= b);",
            "var a = (`0` > b && `0` <= b);",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (
            "if (`green` < x.y && x.y < `blue`) {}",
            "if (x.y > `green` && x.y < `blue`) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[b] && a['b'] < 1) {}",
            "if (a[b] >= 0 && a['b'] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[b] && a[`b`] < 1) {}",
            "if (a[b] >= 0 && a[`b`] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (`0` <= a[b] && a[`b`] < `1`) {}",
            "if (a[b] >= `0` && a[`b`] < `1`) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[b] && a.b < 1) {}",
            "if (a[b] >= 0 && a.b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a.b < 1) {}",
            "if (a[''] >= 0 && a.b < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[' '] < 1) {}",
            "if (a[''] >= 0 && a[' '] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[null] < 1) {}",
            "if (a[''] >= 0 && a[null] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[``] && a[null] < 1) {}",
            "if (a[``] >= 0 && a[null] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[b] < 1) {}",
            "if (a[''] >= 0 && a[b] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[''] && a[b()] < 1) {}",
            "if (a[''] >= 0 && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[``] && a[b()] < 1) {}",
            "if (a[``] >= 0 && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a[b()] && a[b()] < 1) {}",
            "if (a[b()] >= 0 && a[b()] < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 <= a.null && a[/(?<zero>0)/] <= 1) {}",
            "if (a.null >= 0 && a[/(?<zero>0)/] <= 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (3 == a) {}",
            "if (a == 3) {}",
            Some(serde_json::json!(["never", { "onlyEquality": true }])),
        ),
        (
            "foo(3 === a);",
            "foo(a === 3);",
            Some(serde_json::json!(["never", { "onlyEquality": true }])),
        ),
        (
            "foo(a === 3);",
            "foo(3 === a);",
            Some(serde_json::json!(["always", { "onlyEquality": true }])),
        ),
        (
            "foo(a === `3`);",
            "foo(`3` === a);",
            Some(serde_json::json!(["always", { "onlyEquality": true }])),
        ),
        ("if (0 <= x && x < 1) {}", "if (x >= 0 && x < 1) {}", None),
        (
            "if ( /* a */ 0 /* b */ < /* c */ foo /* d */ ) {}",
            "if ( /* a */ foo /* b */ > /* c */ 0 /* d */ ) {}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "if ( /* a */ 0 /* < */ < /* < */ foo /* d */ ) {}",
            "if ( /* a */ foo /* < */ > /* < */ 0 /* d */ ) {}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "if ( /* a */ foo /* b */ > /* c */ 0 /* d */ ) {}",
            "if ( /* a */ 0 /* b */ < /* c */ foo /* d */ ) {}",
            Some(serde_json::json!(["always"])),
        ),
        ("if (foo()===1) {}", "if (1===foo()) {}", Some(serde_json::json!(["always"]))),
        ("if (foo()     === 1) {}", "if (1     === foo()) {}", Some(serde_json::json!(["always"]))),
        ("while (0 === (a));", "while ((a) === 0);", Some(serde_json::json!(["never"]))),
        ("while (0 === (a = b));", "while ((a = b) === 0);", Some(serde_json::json!(["never"]))),
        ("while ((a) === 0);", "while (0 === (a));", Some(serde_json::json!(["always"]))),
        ("while ((a = b) === 0);", "while (0 === (a = b));", Some(serde_json::json!(["always"]))),
        (
            "if (((((((((((foo)))))))))) === ((((((5)))))));",
            "if (((((((5)))))) === ((((((((((foo)))))))))));",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function *foo() { yield(1) < a }",
            "function *foo() { yield a > (1) }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function *foo() { yield((1)) < a }",
            "function *foo() { yield a > ((1)) }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function *foo() { yield 1 < a }",
            "function *foo() { yield a > 1 }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function *foo() { yield/**/1 < a }",
            "function *foo() { yield/**/a > 1 }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function *foo() { yield(1) < ++a }",
            "function *foo() { yield++a > (1) }",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function *foo() { yield(1) < (a) }",
            "function *foo() { yield(a) > (1) }",
            Some(serde_json::json!(["never"])),
        ),
        ("x=1 < a", "x=a > 1", Some(serde_json::json!(["never"]))),
        (
            "function *foo() { yield++a < 1 }",
            "function *foo() { yield 1 > ++a }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function *foo() { yield(a) < 1 }",
            "function *foo() { yield 1 > (a) }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function *foo() { yield a < 1 }",
            "function *foo() { yield 1 > a }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function *foo() { yield/**/a < 1 }",
            "function *foo() { yield/**/1 > a }",
            Some(serde_json::json!(["always"])),
        ),
        (
            "function *foo() { yield++a < (1) }",
            "function *foo() { yield(1) > ++a }",
            Some(serde_json::json!(["always"])),
        ),
        ("x=a < 1", "x=1 > a", Some(serde_json::json!(["always"]))),
        ("0 < f()in obj", "f() > 0 in obj", None),
        ("1 > x++instanceof foo", "x++ < 1 instanceof foo", Some(serde_json::json!(["never"]))),
        ("x < ('foo')in bar", "('foo') > x in bar", Some(serde_json::json!(["always"]))),
        ("false <= ((x))in foo", "((x)) >= false in foo", Some(serde_json::json!(["never"]))),
        ("x >= (1)instanceof foo", "(1) <= x instanceof foo", Some(serde_json::json!(["always"]))),
        ("false <= ((x)) in foo", "((x)) >= false in foo", Some(serde_json::json!(["never"]))),
        ("x >= 1 instanceof foo", "1 <= x instanceof foo", Some(serde_json::json!(["always"]))),
        (
            "x >= 1/**/instanceof foo",
            "1 <= x/**/instanceof foo",
            Some(serde_json::json!(["always"])),
        ),
        ("(x >= 1)instanceof foo", "(1 <= x)instanceof foo", Some(serde_json::json!(["always"]))),
        (
            "(x) >= (1)instanceof foo",
            "(1) <= (x)instanceof foo",
            Some(serde_json::json!(["always"])),
        ),
        ("1 > x===foo", "x < 1===foo", Some(serde_json::json!(["never"]))),
        ("1 > x", "x < 1", Some(serde_json::json!(["never"]))),
        (
            "if (`green` < x.y && x.y < `blue`) {}",
            "if (`green` < x.y && `blue` > x.y) {}",
            Some(serde_json::json!(["always", { "exceptRange": true }])),
        ),
        (
            "if('a' <= x && x < 'b') {}",
            "if('a' <= x && 'b' > x) {}",
            Some(serde_json::json!(["always"])),
        ),
        (
            "if ('b' <= x && x < 'a') {}",
            "if (x >= 'b' && x < 'a') {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if('a' <= x && x < 1) {}",
            "if(x >= 'a' && x < 1) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        (
            "if (0 < a && b < max) {}",
            "if (a > 0 && b < max) {}",
            Some(serde_json::json!(["never", { "exceptRange": true }])),
        ),
        ("y>E>1", "1<y>E", Some(serde_json::json!(["always", { "exceptRange": false }]))),
        // Issue: <https://github.com/oxc-project/oxc/issues/7714>
        (
            "{( t=='' )}",
            "{(  ''==t  )}",
            Some(serde_json::json!(["always", { "onlyEquality": true }])),
        ),
        // Keyword expressions (typeof/void/delete/await/yield/new) need space after return
        (
            "function a(){return\"undefined\"===typeof x}",
            "function a(){return typeof x===\"undefined\"}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function a(){return 0===void x}",
            "function a(){return void x===0}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function a(){return true===delete x.y}",
            "function a(){return delete x.y===true}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "async function a(){return\"x\"===await foo}",
            "async function a(){return await foo===\"x\"}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function* a(){return\"x\"===(yield foo)}",
            "function* a(){return(yield foo)===\"x\"}",
            Some(serde_json::json!(["never"])),
        ),
        (
            "function a(){return\"x\"===new Foo()}",
            "function a(){return new Foo()===\"x\"}",
            Some(serde_json::json!(["never"])),
        ),
    ];

    Tester::new(Yoda::NAME, Yoda::PLUGIN, pass, fail).expect_fix(fix).test_and_snapshot();
}
