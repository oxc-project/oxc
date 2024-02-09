use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_type_of_jest_fn_call, JestFnKind, JestGeneralFnKind,
        PossibleJestNode,
    },
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-comparison-matcher): Prefer using `.each` rather than manual loops")]
#[diagnostic(
    severity(warning),
    help("prefer using `{0:?}.each` rather than a manual loop"),
)]
struct UseToBeComparison(Atom, #[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferComparisonMatcher;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ### Examples
    PreferComparisonMatcher,
    style,
);

impl Rule for PreferComparisonMatcher {}

impl PreferComparisonMatcher {}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
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
    ];

    let fail = vec![];

    Tester::new(PreferComparisonMatcher::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
