use oxc_ast::{
    ast::{Argument, BooleanLiteral, CallExpression, Expression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::Fix,
    rule::Rule,
    utils::{
        collect_possible_jest_call_node, is_equality_matcher, parse_expect_jest_fn_call,
        KnownMemberExpressionParentKind, ParsedExpectFnCall, PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-to-contain): Suggest using `toContain()`.")]
#[diagnostic(severity(warning))]
struct UseToContain(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferToContain;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// When test cases are empty then it is better to mark them as `test.todo` as it
    /// will be highlighted in the summary output.
    ///
    /// ### Why is this bad?
    ///
    /// This rule triggers a warning if `toBe()`, `toEqual()` or `toStrictEqual()` is
    /// used to assert objects length property.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // valid
    /// expect.hasAssertions;
    /// expect.hasAssertions();
    /// expect(files).toHaveLength(1);
    /// expect(files.name).toBe('file');
    ///
    /// // invalid
    /// expect(files["length"]).toBe(1);
    /// expect(files["length"]).toBe(1,);
    /// expect(files["length"])["not"].toBe(1)
    /// ```
    PreferToContain,
    style,
);

impl Rule for PreferToContain {
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            Self::run(possible_jest_node, ctx);
        }
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

        let Some(Argument::Expression(jest_expect_first_arg)) = jest_expect_fn_call.args.first()
        else {
            return;
        };
        let Expression::BooleanLiteral(bool_literal) = jest_expect_first_arg.get_inner_expression()
        else {
            return;
        };
        let Expression::CallExpression(expect_call_expr) = parent else {
            return;
        };

        // handle "expect()"
        if expect_call_expr.arguments.is_empty() {
            return;
        }

        let first_argument = expect_call_expr.arguments.first().unwrap();
        let Argument::Expression(Expression::CallExpression(includes_call_expr)) = first_argument
        else {
            return;
        };

        if !is_equality_matcher(matcher)
            || !Self::is_fixable_includes_call_expression(includes_call_expr)
        {
            return;
        }

        Self::report_and_fix(
            bool_literal.0,
            includes_call_expr,
            &jest_expect_fn_call,
            matcher.span,
            call_expr.span,
            ctx,
        );
    }

    fn is_fixable_includes_call_expression(call_expr: &CallExpression) -> bool {
        let Expression::MemberExpression(mem_expr) = &call_expr.callee else {
            return false;
        };

        mem_expr.static_property_name() == Some("includes")
            // handle "expect(a.includes())"
            && !call_expr.arguments.is_empty()
            // handle "expect(a.includes(b,c))"
            && call_expr.arguments.len() == 1
            // handle "expect(a.includes(...[]))"
            && matches!(call_expr.arguments.first(), Some(Argument::Expression(_)))
    }

    fn report_and_fix(
        bool_arg: &BooleanLiteral,
        call_expr: &CallExpression,
        jest_expect_fn_call: &ParsedExpectFnCall,
        matcher_span: Span,
        call_span: Span,
        ctx: &LintContext,
    ) {
        let (includes_call_span, _) =
            call_expr.callee.get_member_expr().unwrap().static_property_info().unwrap();
        let arg = call_expr.arguments.first().unwrap();
        let Argument::Expression(argument_expr) = arg else {
            return;
        };
        let arg_span = match argument_expr {
            Expression::ArrayExpression(arr_expr) => arr_expr.span,
            Expression::BooleanLiteral(literal) => literal.span,
            Expression::Identifier(ident) => ident.span,
            Expression::StringLiteral(literal) => literal.span,
            Expression::NumericLiteral(literal) => literal.span,
            Expression::ObjectExpression(obj_expr) => obj_expr.span,
            Expression::CallExpression(call_expr) => call_expr.span,
            _ => unreachable!("Should upgrade to support this unknown type"),
        };
        let caller_source = Span::new(call_expr.span.start, includes_call_span.start - 1)
            .source_text(ctx.source_text());
        let arg_source =
            Span::new(arg_span.start, call_expr.span.end).source_text(ctx.source_text());
        let other_arg_source =
            Span::new(arg_span.end, call_expr.span.end).source_text(ctx.source_text());
        let has_not_modifier = jest_expect_fn_call
            .modifiers()
            .iter()
            .filter(|modifier| modifier.is_name_equal("not"))
            .count()
            > 0;
        let expect_name = &jest_expect_fn_call.local;
        let mut content = ctx.codegen();

        content.print_str(expect_name.as_bytes());
        content.print(b'(');
        content.print_str(caller_source.as_bytes());
        content.print_str(other_arg_source.as_bytes());

        if has_not_modifier {
            if bool_arg.value {
                // expect().not.to[MatcherName](true)
                content.print_str(b".not.toContain(");
            } else {
                // expect().not.to[MatcherName](false)
                content.print_str(b".toContain(");
            }
        } else if bool_arg.value {
            // expect().to[MatcherName](true)
            content.print_str(b".toContain(");
        } else {
            // expect().to[MatcherName](false)
            content.print_str(b".not.toContain(");
        }

        content.print_str(arg_source.as_bytes());
        ctx.diagnostic_with_fix(UseToContain(matcher_span), || {
            Fix::new(content.into_source_text(), call_span)
        });
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
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

    let fail = vec![
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

    let fix = vec![
        ("expect(a.includes(b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b,),).toEqual(true,);", "expect(a,).toContain(b,);", None),
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
        ("expect([{a:1}].includes({a:1})).toBe(true);", "expect([{a:1}]).toContain({a:1});", None),
        (
            "expect([{a:1}].includes({a:1})).toBe(false);",
            "expect([{a:1}]).not.toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(true);",
            "expect([{a:1}]).not.toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(false);",
            "expect([{a:1}]).toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(true);",
            "expect([{a:1}]).toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(false);",
            "expect([{a:1}]).not.toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(true);",
            "expect([{a:1}]).not.toContain({a:1});",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "expect([{a:1}]).toContain({a:1});",
            None,
        ),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);
            ",
            "
                import { expect as pleaseExpect } from '@jest/globals';
                pleaseExpect([{a:1}]).toContain({a:1});
            ",
            None,
        ),
        // typescript
        ("expect(a.includes(b)).toEqual(false as boolean);", "expect(a).not.toContain(b);", None),
    ];

    Tester::new(PreferToContain::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
