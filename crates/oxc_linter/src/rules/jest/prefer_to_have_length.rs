use oxc_ast::{
    ast::{Argument, CallExpression, Expression, MemberExpression},
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
        collect_possible_jest_call_node, parse_expect_jest_fn_call, ParsedExpectFnCall,
        PossibleJestNode,
    },
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jest(prefer-to-have-length): Suggest using `toHaveLength()`.")]
#[diagnostic(severity(warning))]
struct UseToHaveLength(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveLength;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// In order to have a better failure message, `toHaveLength()` should be used upon
    /// asserting expectations on objects length property.
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
    ///
    PreferToHaveLength,
    style,
);

impl Rule for PreferToHaveLength {
    fn run_once(&self, ctx: &LintContext) {
        for possible_jest_node in &collect_possible_jest_call_node(ctx) {
            Self::run(possible_jest_node, ctx);
        }
    }
}

impl PreferToHaveLength {
    fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(parsed_expect_call) =
            parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };
        let Expression::MemberExpression(static_expr) = &call_expr.callee else {
            return;
        };

        match static_expr.object() {
            Expression::MemberExpression(mem_expr) => {
                let Expression::CallExpression(expr_call_expr) = mem_expr.object() else {
                    return;
                };
                dbg!(mem_expr);
                match &mem_expr.0 {
                    MemberExpression::ComputedMemberExpression(_) => Self::check_and_fix(
                        expr_call_expr,
                        &parsed_expect_call,
                        call_expr,
                        Some("ComputedMember"),
                        ctx,
                    ),
                    MemberExpression::StaticMemberExpression(_) => Self::check_and_fix(
                        expr_call_expr,
                        &parsed_expect_call,
                        call_expr,
                        Some("StaticMember"),
                        ctx,
                    ),
                    _ => (),
                };
            }
            Expression::CallExpression(expr_call_expr) => {
                Self::check_and_fix(
                    expr_call_expr,
                    &parsed_expect_call,
                    call_expr,
                    None,
                    ctx,
                );
            }
            _ => (),
        }
    }

    fn check_and_fix<'a>(
        expr_call_expr: &CallExpression<'a>,
        parsed_expect_call: &ParsedExpectFnCall<'a>,
        call_expr: &CallExpression<'a>,
        kind: Option<&str>,
        ctx: &LintContext<'a>,
    ) {
        let Some(argument) = expr_call_expr.arguments.first() else {
            return;
        };
        let Argument::Expression(Expression::MemberExpression(static_mem_expr)) = argument else {
            return;
        };
        // Get property `name` field from expect(file.NAME) call
        let Some(expect_property_name) = static_mem_expr.static_property_name() else {
            return;
        };
        let Some(matcher) = parsed_expect_call.matcher() else {
            return;
        };
        let Some(matcher_name) = matcher.name() else {
            return;
        };

        if expect_property_name != "length" || !Self::is_equality_matcher(&matcher_name) {
            return;
        }

        let code = Self::build_code(expr_call_expr, static_mem_expr, call_expr, kind, ctx);

        ctx.diagnostic_with_fix(UseToHaveLength(matcher.span), || {
            Fix::new(code, Span { start: 0, end: call_expr.span.end })
        });
    }

    fn is_equality_matcher(matcher_name: &str) -> bool {
        matcher_name == "toBe" || matcher_name == "toEqual" || matcher_name == "toStrictEqual"
    }

    fn build_code<'a>(
        expr_call_expr: &CallExpression<'a>,
        mem_expr: &MemberExpression,
        call_expr: &CallExpression<'a>,
        kind: Option<&str>,
        ctx: &LintContext<'a>,
    ) -> String {
        let mut formatter = ctx.codegen();

        let Expression::Identifier(prop_ident) = mem_expr.object() else {
            return formatter.into_code();
        };

        if let Expression::Identifier(ident) = &expr_call_expr.callee {
            formatter.print_str(ident.name.as_bytes());
            formatter.print_str(b"(");
            formatter.print_str(prop_ident.name.as_bytes());
            formatter.print_str(b")");
        }

        if let Some(kind_val) = kind {
            if kind_val == "ComputedMember" {
                formatter.print_str(b"[\"not\"]");
            } else if kind_val == "StaticMember" {
                formatter.print_str(b".not");
            }
        }

        formatter.print_str(b".toHaveLength(");

        if call_expr.arguments.len() > 0 {
            formatter.print_str(call_expr.span.source_text(ctx.source_text()).as_bytes());
        }
        // if parsed_expect_call.args.len() > 0 {
        //     // formatter.print_str(parsed_expect_call.args.s);
        // }

        formatter.print_str(b")");
        formatter.into_code()
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    let pass = vec![
        ("expect.hasAssertions", None),
        // ("expect.hasAssertions()", None),
        // ("expect(files).toHaveLength(1);", None),
        // ("expect(files.name).toBe('file');", None),
        // ("expect(files[`name`]).toBe('file');", None),
        // ("expect(users[0]?.permissions?.length).toBe(1);", None),
        // ("expect(result).toBe(true);", None),
        ("expect(user.getUserName(5)).resolves.not.toEqual('Paul')", None),
        // ("expect(user.getUserName(5)).rejects.toEqual('Paul')", None),
        // ("expect(a);", None),
    ];

    let fail = vec![
        // ("expect(files[\"length\"]).toBe(1);", None),
        // ("expect(files[\"length\"]).toBe(1,);", None),
        // ("expect(files[\"length\"])[\"not\"].toBe(1);", None),
        // ("expect(files[\"length\"])[\"toBe\"](1);", None),
        // ("expect(files[\"length\"]).not[\"toBe\"](1);", None),
        // ("expect(files[\"length\"])[\"not\"][\"toBe\"](1);", None),
        // ("expect(files.length).toBe(1);", None),
        // ("expect(files.length).toEqual(1);", None),
        // ("expect(files.length).toStrictEqual(1);", None),
        // ("expect(files.length).not.toStrictEqual(1);", None),
    ];

    let fix = vec![
        // ("expect(files[\"length\"]).not.toBe(1);", "expect(files).not.toHaveLength(1);", None),
        (
            "expect(files[\"length\"]).resolves.not.toBe(1,);",
            "expect(files)['resolve'].toHaveLength(1,);",
            None
        ),
        // ("expect(files[\"length\"]).toBe(1,);", "expect(files).toHaveLength(1);", None),
        // (
        //     "expect(files[\"length\"])[\"not\"].toBe(1);",
        //     "expect(files)[\"not\"].toHaveLength(1);",
        //     None,
        // ),
        // ("expect(files[\"length\"])[\"toBe\"](1);", "expect(files).toHaveLength(1);", None),
        // ("expect(files[\"length\"]).not[\"toBe\"](1);", "expect(files).not.toHaveLength(1);", None),
        // (
        //     "expect(files[\"length\"])[\"not\"][\"toBe\"](1);",
        //     "expect(files)[\"not\"].toHaveLength(1);",
        //     None,
        // ),
        // ("expect(files.length).toBe(1);", "expect(files).toHaveLength(1);", None),
        // ("expect(files.length).toEqual(1);", "expect(files).toHaveLength(1);", None),
        // ("expect(files.length).toStrictEqual(1);", "expect(files).toHaveLength(1);", None),
        // ("expect(files.length).not.toStrictEqual(1);", "expect(files).not.toHaveLength(1);", None),
    ];

    Tester::new(PreferToHaveLength::NAME, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
