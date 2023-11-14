use std::{collections::HashMap, hash::BuildHasherDefault};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{collect_possible_jest_call_node, PossibleJestNode},
};
use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::{FxHashMap, FxHasher};

#[derive(Debug, Error, Diagnostic)]
#[error(
    "eslint-plugin-jest(max-expects): Enforces a maximum number assertion calls in a test body."
)]
#[diagnostic(
    severity(warning),
    help("Too many assertion calls ({0:?}) - maximum allowed is {1:?}")
)]
pub struct ExceededMaxAssertion(pub usize, pub usize, #[label] pub Span);

#[derive(Debug, Clone)]
pub struct MaxExpects {
    pub max: usize,
}

impl Default for MaxExpects {
    fn default() -> Self {
        Self { max: 5 }
    }
}

declare_oxc_lint!(
    /// ### What it does
    /// As more assertions are made, there is a possible tendency for the test to be
    /// more likely to mix multiple objectives. To avoid this, this rule reports when
    /// the maximum number of assertions is exceeded.
    ///
    /// ### Why is this bad?
    ///
    /// This rule enforces a maximum number of `expect()` calls.
    /// The following patterns are considered warnings (with the default option of `{ "max": 5 } `):
    ///
    /// ### Example
    ///
    /// ```javascript
    /// test('should not pass', () => {
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    /// });
    ///
    /// it('should not pass', () => {
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    ///     expect(true).toBeDefined();
    /// });
    /// ```
    MaxExpects,
    style,
);

impl Rule for MaxExpects {
    fn from_configuration(value: serde_json::Value) -> Self {
        let max = value
            .get(0)
            .and_then(|config| config.get("max"))
            .and_then(serde_json::Value::as_number)
            .and_then(serde_json::Number::as_u64)
            .map_or(5, |v| usize::try_from(v).unwrap_or(5));

        Self { max }
    }

    fn run_once(&self, ctx: &LintContext) {
        let mut count_map: HashMap<usize, usize, BuildHasherDefault<FxHasher>> =
            FxHashMap::default();

        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            self.run(possible_jest_node, &mut count_map, ctx);
        }
    }
}

impl MaxExpects {
    fn run<'a>(
        &self,
        jest_node: &PossibleJestNode<'a, '_>,
        count_map: &mut HashMap<usize, usize, BuildHasherDefault<FxHasher>>,
        ctx: &LintContext<'a>,
    ) {
        let node = jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Expression::Identifier(ident) = &call_expr.callee else {
            return;
        };

        if ident.name == "expect" {
            let position = node.scope_id().index();

            if let Some(count) = count_map.get(&position) {
                if count > &self.max {
                    ctx.diagnostic(ExceededMaxAssertion(*count, self.max, ident.span));
                } else {
                    count_map.insert(position, count + 1);
                }
            } else {
                count_map.insert(position, 2);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("test('should pass')", None),
        ("test('should pass', () => {})", None),
        ("test.skip('should pass', () => {})", None),
        (
            "
                test('should pass', function () {
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    // expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect.hasAssertions();

                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toEqual(expect.any(Boolean));
                });
            ",
            None,
        ),
        (
            "
                test('should pass', async () => {
                    expect.hasAssertions();

                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toEqual(expect.any(Boolean));
                });
            ",
            None,
        ),
        (
            "
                describe('test', () => {
                    test('should pass', () => {
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                    });
                });
            ",
            None,
        ),
        (
            "
                test.each(['should', 'pass'], () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                function myHelper() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                function myHelper1() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                function myHelper2() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                function myHelper() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                const myHelper1 = () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };

                test('should pass', function() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });

                const myHelper2 = function() {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                };
            ",
            None,
        ),
        (
            "
                test('should pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "max": 10 }])),
        ),
    ];

    let fail = vec![
        (
            "
                test('should not pass', function () {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                it('should not pass', async () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                describe('test', () => {
                    test('should not pass', () => {
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                        expect(true).toBeDefined();
                    });
                });
            ",
            None,
        ),
        (
            "
                test.each(['should', 'not', 'pass'], () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            None,
        ),
        (
            "
                test('should not pass', () => {
                    expect(true).toBeDefined();
                    expect(true).toBeDefined();
                });
            ",
            Some(serde_json::json!([{ "max": 1 }])),
        ),
    ];

    Tester::new(MaxExpects::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
