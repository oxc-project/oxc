use oxc_ast::{
    AstKind,
    ast::{Argument, BinaryExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    context::LintContext,
    fixer::RuleFixer,
    rule::Rule,
    utils::{
        KnownMemberExpressionProperty, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_to_be_comparison(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using the built-in comparison matchers")
        .with_help(format!("Prefer using `{x0:?}` instead"))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct PreferComparisonMatcher;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule checks for comparisons in tests that could be replaced with one of the
    /// following built-in comparison matchers:
    /// - `toBeGreaterThan`
    /// - `toBeGreaterThanOrEqual`
    /// - `toBeLessThan`
    /// - `toBeLessThanOrEqual`
    ///
    /// ### Why is this bad?
    ///
    /// Using generic matchers like `toBe(true)` with comparison expressions
    /// makes tests less readable and provides less helpful error messages when
    /// they fail. Jest's specific comparison matchers offer clearer intent and
    /// better error output that shows the actual values being compared.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// expect(x > 5).toBe(true);
    /// expect(x < 7).not.toEqual(true);
    /// expect(x <= y).toStrictEqual(true);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// expect(x).toBeGreaterThan(5);
    /// expect(x).not.toBeLessThanOrEqual(7);
    /// expect(x).toBeLessThanOrEqual(y);
    /// // special case - see below
    /// expect(x < 'Carl').toBe(true);
    /// ```
    PreferComparisonMatcher,
    jest,
    style,
    fix
);

impl Rule for PreferComparisonMatcher {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferComparisonMatcher {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(parse_expect_jest_fn) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(matcher) = parse_expect_jest_fn.matcher() else {
            return;
        };
        let Some(parent_node) = parse_expect_jest_fn.head.parent else {
            return;
        };
        let Expression::CallExpression(parent_call_expr) = parent_node else {
            return;
        };
        let Some(Argument::BinaryExpression(binary_expr)) = parent_call_expr.arguments.first()
        else {
            return;
        };
        let Some(first_matcher_arg) =
            parse_expect_jest_fn.args.first().and_then(Argument::as_expression)
        else {
            return;
        };

        if Self::is_comparing_to_string(binary_expr) || !is_equality_matcher(matcher) {
            return;
        }

        let has_not_modifier =
            parse_expect_jest_fn.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));
        let Expression::BooleanLiteral(matcher_arg_value) =
            first_matcher_arg.get_inner_expression()
        else {
            return;
        };
        let negated = matcher_arg_value.value == has_not_modifier;
        let preferred_matcher = Self::determine_matcher(binary_expr.operator, negated);
        let Some(prefer_matcher_name) = preferred_matcher else {
            return;
        };

        ctx.diagnostic_with_fix(use_to_be_comparison(prefer_matcher_name, matcher.span), |fixer| {
            // This is to handle the case can be transform into the following case:
            // expect(value > 1,).toEqual(true,) => expect(value,).toBeGreaterThan(1,)
            //                 ^              ^
            // Therefore the range starting after ',' and before '.' is called as call_span_end,
            // and the same as `arg_span_end`.
            let call_span_end =
                fixer.source_range(Span::new(binary_expr.span.end, parent_call_expr.span.end));
            let arg_span_end =
                fixer.source_range(Span::new(matcher_arg_value.span.end, call_expr.span.end));
            let content = Self::building_code(
                binary_expr,
                call_span_end,
                arg_span_end,
                &parse_expect_jest_fn.local,
                &parse_expect_jest_fn.modifiers(),
                prefer_matcher_name,
                fixer,
            );
            fixer.replace(call_expr.span, content)
        });
    }

    fn is_comparing_to_string(expr: &BinaryExpression) -> bool {
        matches!(expr.left, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
            || matches!(expr.right, Expression::StringLiteral(_) | Expression::TemplateLiteral(_))
    }

    fn determine_matcher(operator: BinaryOperator, negated: bool) -> Option<&'static str> {
        let op = if negated { Self::invert_operator(operator) } else { Some(operator) };

        match op {
            // >
            Some(BinaryOperator::GreaterThan) => Some("toBeGreaterThan"),
            // >=
            Some(BinaryOperator::GreaterEqualThan) => Some("toBeGreaterThanOrEqual"),
            // <
            Some(BinaryOperator::LessThan) => Some("toBeLessThan"),
            // <=
            Some(BinaryOperator::LessEqualThan) => Some("toBeLessThanOrEqual"),
            _ => None,
        }
    }

    fn invert_operator(operator: BinaryOperator) -> Option<BinaryOperator> {
        match operator {
            // ">" => "<="
            BinaryOperator::GreaterThan => Some(BinaryOperator::LessEqualThan),
            // "<" => ">="
            BinaryOperator::LessThan => Some(BinaryOperator::GreaterEqualThan),
            // ">=" => "<"
            BinaryOperator::GreaterEqualThan => Some(BinaryOperator::LessThan),
            // "<=" => ">"
            BinaryOperator::LessEqualThan => Some(BinaryOperator::GreaterThan),
            _ => None,
        }
    }

    fn building_code<'a>(
        binary_expr: &BinaryExpression<'a>,
        call_span_end: &str,
        arg_span_end: &str,
        local_name: &str,
        modifiers: &[&KnownMemberExpressionProperty<'a>],
        prefer_matcher_name: &str,
        fixer: RuleFixer<'_, 'a>,
    ) -> String {
        let mut content = fixer.codegen();
        content.print_str(local_name);
        content.print_ascii_byte(b'(');
        content.print_expression(&binary_expr.left);
        content.print_str(call_span_end);
        content.print_ascii_byte(b'.');
        for modifier in modifiers {
            let Some(modifier_name) = modifier.name() else {
                continue;
            };

            if !modifier_name.eq("not") {
                content.print_str(&modifier_name);
                content.print_ascii_byte(b'.');
            }
        }
        content.print_str(prefer_matcher_name);
        content.print_ascii_byte(b'(');
        content.print_expression(&binary_expr.right);
        content.print_str(arg_span_end);
        content.into_source_text()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn generate_test_cases(
        operator: &str,
        generate_fn: fn(operator: &str, matcher: &str) -> Vec<String>,
    ) -> Vec<String> {
        let equality_matchers = vec!["toBe", "toEqual", "toStrictEqual"];
        let mut cases: Vec<String> = Vec::new();

        for equality_matcher in &equality_matchers {
            let case = generate_fn(operator, equality_matcher);
            cases.extend(case);
        }

        cases
    }

    fn generate_valid_string_literal_cases(operator: &str, matcher: &str) -> Vec<String> {
        [("x", "'y'"), ("x", "`y`"), ("x", "`y${z}`")]
            .iter()
            .flat_map(|(a, b)| {
                vec![
                    format!("expect({} {} {}).{}(true)", a, operator, b, matcher),
                    format!("expect({} {} {}).{}(false)", a, operator, b, matcher),
                    format!("expect({} {} {}).not.{}(true)", a, operator, b, matcher),
                    format!("expect({} {} {}).not.{}(false)", a, operator, b, matcher),
                    format!("expect({} {} {}).{}(true)", b, operator, a, matcher),
                    format!("expect({} {} {}).{}(false)", b, operator, a, matcher),
                    format!("expect({} {} {}).not.{}(true)", b, operator, a, matcher),
                    format!("expect({} {} {}).not.{}(false)", b, operator, a, matcher),
                    format!("expect({} {} {}).{}(true)", a, operator, b, matcher),
                    format!("expect({} {} {}).{}(false)", a, operator, b, matcher),
                    format!("expect({} {} {}).not.{}(true)", a, operator, b, matcher),
                    format!("expect({} {} {}).not.{}(false)", a, operator, b, matcher),
                    format!("expect({} {} {}).{}(true)", b, operator, a, matcher),
                    format!("expect({} {} {}).{}(false)", b, operator, a, matcher),
                    format!("expect({} {} {}).not.{}(true)", b, operator, a, matcher),
                    format!("expect({} {} {}).not.{}(false)", b, operator, a, matcher),
                    format!("expect({} {} {}).not.{}(false)", b, operator, b, matcher),
                    format!("expect({} {} {}).resolves.not.{}(false)", b, operator, b, matcher),
                    format!("expect({} {} {}).resolves.{}(false)", b, operator, b, matcher),
                ]
            })
            .collect()
    }

    fn generate_fail_cases(operator: &str, matcher: &str) -> Vec<String> {
        vec![
            format!("expect(value {} 1).{}(true);", operator, matcher),
            format!("expect(value {} 1,).{}(true,);", operator, matcher),
            format!("expect(value {} 1)['{}'](true);", operator, matcher),
            format!("expect(value {} 1).resolves.{}(true);", operator, matcher),
            format!("expect(value {} 1).{}(false);", operator, matcher),
            format!("expect(value {} 1)['{}'](false);", operator, matcher),
            format!("expect(value {} 1).resolves.{}(false);", operator, matcher),
            format!("expect(value {} 1).not.{}(true);", operator, matcher),
            format!("expect(value {} 1)['not'].{}(true);", operator, matcher),
            format!("expect(value {} 1).resolves.not.{}(true);", operator, matcher),
            format!("expect(value {} 1).not.{}(false);", operator, matcher),
            format!("expect(value {} 1).resolves.not.{}(false);", operator, matcher),
            format!("expect(value {} 1)[\"resolves\"].not.{}(false);", operator, matcher),
            format!("expect(value {} 1)[\"resolves\"][\"not\"].{}(false);", operator, matcher),
            format!("expect(value {} 1)[\"resolves\"][\"not\"]['{}'](false);", operator, matcher),
        ]
    }

    fn generate_fix_cases(
        operator: &str,
        matcher: &str,
        preferred_matcher: &str,
        preferred_matcher_when_negated: &str,
    ) -> Vec<(String, String)> {
        vec![
            (
                format!("expect(value {operator} 1).{matcher}(true);"),
                format!("expect(value).{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1,).{matcher}(true,);"),
                format!("expect(value,).{preferred_matcher}(1,);"),
            ),
            (
                format!("expect(value {operator} 1)['{matcher}'](true);"),
                format!("expect(value).{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1).resolves.{matcher}(true);"),
                format!("expect(value).resolves.{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1).{matcher}(false);"),
                format!("expect(value).{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1)['{matcher}'](false);"),
                format!("expect(value).{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1).resolves.{matcher}(false);"),
                format!("expect(value).resolves.{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1).not.{matcher}(true);"),
                format!("expect(value).{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1)['not'].{matcher}(true);"),
                format!("expect(value).{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1).resolves.not.{matcher}(true);"),
                format!("expect(value).resolves.{preferred_matcher_when_negated}(1);"),
            ),
            (
                format!("expect(value {operator} 1).not.{matcher}(false);"),
                format!("expect(value).{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1).resolves.not.{matcher}(false);"),
                format!("expect(value).resolves.{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1)[\"resolves\"].not.{matcher}(false);"),
                format!("expect(value).resolves.{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1)[\"resolves\"][\"not\"].{matcher}(false);"),
                format!("expect(value).resolves.{preferred_matcher}(1);"),
            ),
            (
                format!("expect(value {operator} 1)[\"resolves\"][\"not\"]['{matcher}'](false);"),
                format!("expect(value).resolves.{preferred_matcher}(1);"),
            ),
        ]
    }

    fn building_fix_cases(
        operator: &str,
        preferred_matcher: &str,
        preferred_matcher_when_negated: &str,
    ) -> Vec<(String, String)> {
        let equality_matchers = vec!["toBe", "toEqual", "toStrictEqual"];
        let mut cases: Vec<(String, String)> = Vec::new();

        for equality_matcher in &equality_matchers {
            let case = generate_fix_cases(
                operator,
                equality_matcher,
                preferred_matcher,
                preferred_matcher_when_negated,
            );
            cases.extend(case);
        }

        cases
    }

    let valid_greater_cases = generate_test_cases(">", generate_valid_string_literal_cases);
    let valid_less_cases = generate_test_cases("<", generate_valid_string_literal_cases);
    let valid_greater_equal_cases = generate_test_cases(">=", generate_valid_string_literal_cases);
    let valid_less_equal_cases = generate_test_cases("<=", generate_valid_string_literal_cases);

    let mut pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect()", None),
        ("expect({}).toStrictEqual({})", None),
        ("expect(a === b).toBe(true)", None),
        ("expect(a !== 2).toStrictEqual(true)", None),
        ("expect(a === b).not.toEqual(true)", None),
        ("expect(a !== \"string\").toStrictEqual(true)", None),
        ("expect(5 != a).toBe(true)", None),
        ("expect(a == \"string\").toBe(true)", None),
        ("expect(a == \"string\").not.toBe(true)", None),
        // >
        ("expect(value).toBeGreaterThan(1);", None),
        ("expect(value).toBeLessThanOrEqual(1);", None),
        ("expect(value).not.toBeGreaterThan(1);", None),
        ("expect(value).not.toBeLessThanOrEqual(1)", None),
        // <
        ("expect(value).toBeLessThan(1);", None),
        ("expect(value).toBeGreaterThanOrEqual(1);", None),
        ("expect(value).not.toBeLessThan(1);", None),
        ("expect(value).not.toBeGreaterThanOrEqual(1)", None),
        // >=
        ("expect(value).toBeGreaterThanOrEqual(1);", None),
        ("expect(value).toBeLessThan(1);", None),
        ("expect(value).not.toBeGreaterThanOrEqual(1);", None),
        ("expect(value).not.toBeLessThan(1)", None),
        // <=
        ("expect(value).toBeLessThanOrEqual(1);", None),
        ("expect(value).toBeGreaterThan(1);", None),
        ("expect(value).not.toBeLessThanOrEqual(1);", None),
        ("expect(value).not.toBeGreaterThan(1)", None),
    ];

    for case in &valid_greater_cases {
        pass.push((case.as_str(), None));
    }

    for case in &valid_less_cases {
        pass.push((case.as_str(), None));
    }

    for case in &valid_greater_equal_cases {
        pass.push((case.as_str(), None));
    }

    for case in &valid_less_equal_cases {
        pass.push((case.as_str(), None));
    }

    let invalid_greater_cases = generate_test_cases(">", generate_fail_cases);
    let invalid_less_cases = generate_test_cases("<", generate_fail_cases);
    let invalid_greater_equal_cases = generate_test_cases(">=", generate_fail_cases);
    let invalid_less_equal_cases = generate_test_cases("<=", generate_fail_cases);
    let mut fail = vec![];

    for case in &invalid_greater_cases {
        fail.push((case.as_str(), None));
    }

    for case in &invalid_less_cases {
        fail.push((case.as_str(), None));
    }

    for case in &invalid_greater_equal_cases {
        fail.push((case.as_str(), None));
    }

    for case in &invalid_less_equal_cases {
        fail.push((case.as_str(), None));
    }

    let fix_greater_cases = building_fix_cases(">", "toBeGreaterThan", "toBeLessThanOrEqual");
    let fix_less_cases = building_fix_cases("<", "toBeLessThan", "toBeGreaterThanOrEqual");
    let fix_greater_equal_cases =
        building_fix_cases(">=", "toBeGreaterThanOrEqual", "toBeLessThan");
    let fix_less_equal_cases = building_fix_cases("<=", "toBeLessThanOrEqual", "toBeGreaterThan");
    let mut fix = vec![];

    for (case, fixer) in &fix_greater_cases {
        fix.push((case.as_str(), fixer.as_str(), None));
    }

    for (case, fixer) in &fix_less_cases {
        fix.push((case.as_str(), fixer.as_str(), None));
    }

    for (case, fixer) in &fix_greater_equal_cases {
        fix.push((case.as_str(), fixer.as_str(), None));
    }

    for (case, fixer) in &fix_less_equal_cases {
        fix.push((case.as_str(), fixer.as_str(), None));
    }

    let pass_vitest = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect()", None),
        ("expect({}).toStrictEqual({})", None),
        ("expect(a === b).toBe(true)", None),
        ("expect(a !== 2).toStrictEqual(true)", None),
        ("expect(a === b).not.toEqual(true)", None),
        (r#"expect(a !== "string").toStrictEqual(true)"#, None),
        ("expect(5 != a).toBe(true)", None),
        (r#"expect(a == "string").toBe(true)"#, None),
        (r#"expect(a == "string").not.toBe(true)"#, None),
        ("expect().fail('Should not succeed a HTTPS proxy request.');", None),
    ];

    let fail_vitest = vec![
        ("expect(a > b).toBe(true)", None),
        ("expect(a < b).toBe(true)", None),
        ("expect(a >= b).toBe(true)", None),
        ("expect(a <= b).toBe(true)", None),
        ("expect(a > b).not.toBe(true)", None),
        ("expect(a < b).not.toBe(true)", None),
        ("expect(a >= b).not.toBe(true)", None),
    ];

    let fix_vitest = vec![
        ("expect(a > b).toBe(true)", "expect(a).toBeGreaterThan(b)", None),
        ("expect(a < b).toBe(true)", "expect(a).toBeLessThan(b)", None),
        ("expect(a >= b).toBe(true)", "expect(a).toBeGreaterThanOrEqual(b)", None),
        ("expect(a <= b).toBe(true)", "expect(a).toBeLessThanOrEqual(b)", None),
        ("expect(a > b).not.toBe(true)", "expect(a).toBeLessThanOrEqual(b)", None),
        ("expect(a < b).not.toBe(true)", "expect(a).toBeGreaterThanOrEqual(b)", None),
        ("expect(a >= b).not.toBe(true)", "expect(a).toBeLessThan(b)", None),
    ];

    pass.extend(pass_vitest);
    fail.extend(fail_vitest);
    fix.extend(fix_vitest);

    Tester::new(PreferComparisonMatcher::NAME, PreferComparisonMatcher::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
