use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_comparison_matcher::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferComparisonMatcher;

declare_oxc_lint!(
    PreferComparisonMatcher,
    jest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "0.2.15",
);

impl Rule for PreferComparisonMatcher {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
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
        ("expect(value).toBeGreaterThan(1);", None),
        ("expect(value).toBeLessThanOrEqual(1);", None),
        ("expect(value).not.toBeGreaterThan(1);", None),
        ("expect(value).not.toBeLessThanOrEqual(1)", None),
        ("expect(value).toBeLessThan(1);", None),
        ("expect(value).toBeGreaterThanOrEqual(1);", None),
        ("expect(value).not.toBeLessThan(1);", None),
        ("expect(value).not.toBeGreaterThanOrEqual(1)", None),
        ("expect(value).toBeGreaterThanOrEqual(1);", None),
        ("expect(value).toBeLessThan(1);", None),
        ("expect(value).not.toBeGreaterThanOrEqual(1);", None),
        ("expect(value).not.toBeLessThan(1)", None),
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

    Tester::new(PreferComparisonMatcher::NAME, PreferComparisonMatcher::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
