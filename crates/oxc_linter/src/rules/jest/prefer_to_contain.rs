use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_ecmascript::{ToBoolean, WithoutGlobalReferenceInformation};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        KnownMemberExpressionParentKind, PossibleJestNode, is_equality_matcher,
        parse_expect_jest_fn_call,
    },
};

fn use_to_contain(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Suggest using `toContain()`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferToContain;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In order to have a better failure message, `toContain()` should be used upon
    /// asserting expectations on an array containing an object.
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if `toBe()`, `toEqual()` or `toStrictEqual()` is
    /// used to assert object inclusion in an array
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// expect(a.includes(b)).toBe(true);
    /// expect(a.includes(b)).not.toBe(true);
    /// expect(a.includes(b)).toBe(false);
    /// expect(a.includes(b)).toEqual(true);
    /// expect(a.includes(b)).toStrictEqual(true);
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// expect(a).toContain(b);
    /// expect(a).not.toContain(b);
    /// ```
    ///
    /// This rule is compatible with [eslint-plugin-vitest](https://github.com/vitest-dev/eslint-plugin-vitest/blob/main/docs/rules/prefer-to-contain.md),
    /// to use it, add the following configuration to your `.oxlintrc.json`:
    ///
    /// ```json
    /// {
    ///   "rules": {
    ///      "vitest/prefer-to-contain": "error"
    ///   }
    /// }
    /// ```
    PreferToContain,
    jest,
    style,
    fix
);

impl Rule for PreferToContain {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl PreferToContain {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(jest_expect_fn_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Some(parent) = jest_expect_fn_call.head.parent else {
            return;
        };
        let Some(matcher) = jest_expect_fn_call.matcher() else {
            return;
        };

        if !matches!(
            jest_expect_fn_call.head.parent_kind.unwrap(),
            KnownMemberExpressionParentKind::Call
        ) || jest_expect_fn_call.args.is_empty()
        {
            return;
        }

        let Some(jest_expect_first_arg) =
            jest_expect_fn_call.args.first().and_then(Argument::as_expression)
        else {
            return;
        };
        let Expression::CallExpression(expect_call_expr) = parent else {
            return;
        };

        // handle "expect()"
        if expect_call_expr.arguments.is_empty()
            || !matches!(
                jest_expect_first_arg.get_inner_expression(),
                Expression::BooleanLiteral(_)
            )
        {
            return;
        }

        let Some(first_argument) = expect_call_expr.arguments.first() else {
            return;
        };
        let Argument::CallExpression(includes_call_expr) = first_argument else {
            return;
        };

        if !is_equality_matcher(matcher)
            || !Self::is_fixable_includes_call_expression(includes_call_expr)
        {
            return;
        }

        ctx.diagnostic_with_fix(use_to_contain(matcher.span), |fixer| {
            let Some(mem_expr) = includes_call_expr.callee.as_member_expression() else {
                return fixer.noop();
            };

            let Some(argument) = includes_call_expr.arguments.first() else {
                return fixer.noop();
            };

            let negation = {
                let boolean_value = jest_expect_first_arg
                    .get_inner_expression()
                    .to_boolean(&WithoutGlobalReferenceInformation {})
                    .unwrap_or(false);
                let has_not = jest_expect_fn_call
                    .modifiers()
                    .iter()
                    .any(|modifier| modifier.is_name_equal("not"));

                match (boolean_value, has_not) {
                    (false, true) | (true, false) => "",
                    (true, true) | (false, false) => ".not",
                }
            };

            let mut formatter = fixer.codegen();

            formatter.print_expression(&expect_call_expr.callee);
            formatter.print_str("(");
            formatter.print_expression(mem_expr.object());
            formatter.print_str(format!("){negation}.toContain(").as_str());
            formatter.print_expression(argument.to_expression());
            formatter.print_str(")");

            fixer.replace(call_expr.span, formatter.into_source_text())
        });
    }

    fn is_fixable_includes_call_expression(call_expr: &CallExpression) -> bool {
        let Some(mem_expr) = call_expr.callee.as_member_expression() else {
            return false;
        };

        mem_expr.static_property_name() == Some("includes")
            // handle "expect(a.includes())"
            && !call_expr.arguments.is_empty()
            // handle "expect(a.includes(b,c))"
            && call_expr.arguments.len() == 1
            // handle "expect(a.includes(...[]))"
            && call_expr.arguments[0].is_expression()
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    #[expect(clippy::literal_string_with_formatting_args)]
    let mut pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect().toBe(false);", None),
        ("expect(a).toContain(b);", None),
        ("expect(a.name).toBe('b');", None),
        ("expect(a).toBe(true);", None),
        ("expect(a).toEqual(b)", None),
        ("expect(a.test(c)).toEqual(b)", None),
        ("expect(a.includes(b)).toEqual()", None),
        ("expect(a.includes(b)).toEqual(\"test\")", None),
        ("expect(a.includes(b)).toBe(\"test\")", None),
        ("expect(a.includes()).toEqual()", None),
        ("expect(a.includes()).toEqual(true)", None),
        ("expect(a.includes(b,c)).toBe(true)", None),
        ("expect([{a:1}]).toContain({a:1})", None),
        ("expect([1].includes(1)).toEqual", None),
        ("expect([1].includes).toEqual", None),
        ("expect([1].includes).not", None),
        ("expect(a.test(b)).resolves.toEqual(true)", None),
        ("expect(a.test(b)).resolves.not.toEqual(true)", None),
        ("expect(a).not.toContain(b)", None),
        ("expect(a.includes(...[])).toBe(true)", None),
        ("expect(a.includes(b)).toBe(...true)", None),
        ("expect(a);", None),
        // typescript
        (
            "(expect('Model must be bound to an array if the multiple property is true') as any).toHaveBeenTipped()",
            None,
        ),
        ("expect(a.includes(b)).toEqual(0 as boolean);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let mut fail = vec![
        ("expect(a.includes(b)).toEqual(true);", None),
        ("expect(a.includes(b,),).toEqual(true,)", None),
        ("expect(a['includes'](b)).toEqual(true);", None),
        ("expect(a['includes'](b))['toEqual'](true);", None),
        ("expect(a['includes'](b)).toEqual(false);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", None),
        ("expect(a.includes(b)).toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(true);", None),
        ("expect(a.includes(b)).toBe(true);", None),
        ("expect(a.includes(b)).toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(true);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(false);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(false);", None),
        ("expect([{a:1}].includes({b:1})).toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(false);", None),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);
            ",
            None,
        ),
        // typescript
        ("expect(a.includes(b)).toEqual(false as boolean);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let mut fix = vec![
        ("expect(a.includes(b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b,),).toEqual(true,)", "expect(a).toContain(b)", None),
        ("expect(a['includes'](b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['toEqual'](true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toBe(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toBe(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", "expect(a).not.toContain(b);", None),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(true);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(false);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(true);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(false);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        // Diff with eslint: The default print_expression add a space between key and value, and before and after curly braces, values
        (
            "expect([{a:1}].includes({a:1})).toBe(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toBe(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "import { expect as pleaseExpect } from '@jest/globals';
			pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "import { expect as pleaseExpect } from '@jest/globals';
			pleaseExpect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        ("expect(a.includes(b)).toEqual(false as boolean);", "expect(a).not.toContain(b);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let vitest_pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect().toBe(false);", None),
        ("expect(a).toContain(b);", None),
        ("expect(a.name).toBe('b');", None),
        ("expect(a).toBe(true);", None),
        ("expect(a).toEqual(b)", None),
        ("expect(a.test(c)).toEqual(b)", None),
        ("expect(a.includes(b)).toEqual()", None),
        ("expect(a.includes(b)).toEqual('test')", None),
        ("expect(a.includes(b)).toBe('test')", None),
        ("expect(a.includes()).toEqual()", None),
        ("expect(a.includes()).toEqual(true)", None),
        ("expect(a.includes(b,c)).toBe(true)", None),
        ("expect([{a:1}]).toContain({a:1})", None),
        ("expect([1].includes(1)).toEqual", None),
        ("expect([1].includes).toEqual", None),
        ("expect([1].includes).not", None),
        ("expect(a.test(b)).resolves.toEqual(true)", None),
        ("expect(a.test(b)).resolves.not.toEqual(true)", None),
        ("expect(a).not.toContain(b)", None),
        ("expect(a.includes(...[])).toBe(true)", None),
        ("expect(a.includes(b)).toBe(...true)", None),
        ("expect(a);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let vitest_fail = vec![
        ("expect(a.includes(b)).toEqual(true);", None),
        ("expect(a.includes(b,),).toEqual(true,);", None),
        ("expect(a['includes'](b)).toEqual(true);", None),
        ("expect(a['includes'](b))['toEqual'](true);", None),
        ("expect(a['includes'](b)).toEqual(false);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", None),
        ("expect(a.includes(b)).toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(true);", None),
        ("expect(a.includes(b)).toBe(true);", None),
        ("expect(a.includes(b)).toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(true);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(false);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(false);", None),
        ("expect([{a:1}].includes({a:1})).toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(false);", None),
        (
            "
                    import { expect as pleaseExpect } from 'vitest';
                    pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);
                ",
            None,
        ),
        // typescript
        ("expect(a.includes(b)).toEqual(false as boolean);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let vitest_fix = vec![
        ("expect(a.includes(b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b,),).toEqual(true,);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['toEqual'](true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toBe(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toBe(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", "expect(a).not.toContain(b);", None),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(true);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(false);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(true);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(false);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toBe(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toBe(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "import { expect as pleaseExpect } from 'vitest';
                pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "import { expect as pleaseExpect } from 'vitest';
                pleaseExpect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        ("expect(a.includes(b)).toEqual(false as boolean);", "expect(a).not.toContain(b);", None),
    ];

    pass.extend(vitest_pass);
    fail.extend(vitest_fail);
    fix.extend(vitest_fix);

    Tester::new(PreferToContain::NAME, PreferToContain::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
