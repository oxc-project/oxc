use oxc_ast::{
    ast::{Argument, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::operator::BinaryOperator;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_expect_jest_fn_call, PossibleJestNode},
};

fn use_equality_matcher_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using the built-in equality matchers.")
        .with_help("Prefer using one of the equality matchers instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferEqualityMatcher;

declare_oxc_lint!(
    /// ### What it does
    /// Jest has built-in matchers for expecting equality, which allow for more readable
    /// tests and error messages if an expectation fails.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // invalid
    /// expect(x === 5).toBe(true);
    /// expect(name === 'Carl').not.toEqual(true);
    /// expect(myObj !== thatObj).toStrictEqual(true);
    ///
    /// // valid
    /// expect(x).toBe(5);
    /// expect(name).not.toEqual('Carl');
    /// expect(myObj).toStrictEqual(thatObj);
    /// ```
    PreferEqualityMatcher,
    style,
);

impl Rule for PreferEqualityMatcher {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferEqualityMatcher {
    pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let Some(expect_parent) = jest_fn_call.head.parent else {
            return;
        };
        let expr = expect_parent.get_inner_expression();
        let Expression::CallExpression(call_expr) = expr else {
            return;
        };
        let Some(argument) = call_expr.arguments.first() else {
            return;
        };

        let Argument::BinaryExpression(binary_expr) = argument else {
            return;
        };

        if binary_expr.operator != BinaryOperator::StrictEquality
            && binary_expr.operator != BinaryOperator::StrictInequality
        {
            return;
        }

        let Some(matcher) = jest_fn_call.matcher() else {
            return;
        };

        ctx.diagnostic(use_equality_matcher_diagnostic(matcher.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect(true).toBe(...true)", None),
        ("expect(a == 1).toBe(true)", None),
        ("expect(1 == a).toBe(true)", None),
        ("expect(a == b).toBe(true)", None),
    ];

    let fail = vec![
        ("expect(a !== b).toBe(true)", None),
        ("expect(a !== b).toBe(false)", None),
        ("expect(a !== b).resolves.toBe(true)", None),
        ("expect(a !== b).resolves.toBe(false)", None),
        ("expect(a !== b).not.toBe(true)", None),
        ("expect(a !== b).not.toBe(false)", None),
        ("expect(a !== b).resolves.not.toBe(true)", None),
        ("expect(a !== b).resolves.not.toBe(false)", None),
    ];

    Tester::new(PreferEqualityMatcher::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
