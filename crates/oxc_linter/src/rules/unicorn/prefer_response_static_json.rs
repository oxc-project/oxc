use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression, NewExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{could_be_asi_hazard, is_method_call, is_new_expression},
    context::LintContext,
    rule::Rule,
};

fn prefer_response_static_json_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Prefer using `Response.json(…)` over `JSON.stringify()`.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferResponseStaticJson;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces the use of `Response.json()` over `new Response(JSON.stringify())`.
    ///
    /// ### Why is this bad?
    ///
    /// `Response.json()` is a more concise and semantically clear way to create JSON responses.
    /// It automatically sets the correct `Content-Type` header (`application/json`) and handles
    /// serialization, making the code more maintainable and less error-prone.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const response = new Response(JSON.stringify(data));
    /// const response = new Response(JSON.stringify(data), { status: 200 });
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const response = Response.json(data);
    /// const response = Response.json(data, { status: 200 });
    /// ```
    PreferResponseStaticJson,
    unicorn,
    style,
    suggestion
);

impl Rule for PreferResponseStaticJson {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::NewExpression(new_expr) = node.kind() else {
            return;
        };

        if !is_new_expression(new_expr, &["Response"], Some(1), None) {
            return;
        }

        let Some(argument) = new_expr.arguments.first() else {
            return;
        };

        let Some(argument_expr) = argument.as_expression() else {
            return;
        };

        let Expression::CallExpression(call_expr) = argument_expr.get_inner_expression() else {
            return;
        };

        if !is_method_call(call_expr, Some(&["JSON"]), Some(&["stringify"]), Some(1), Some(1)) {
            return;
        }

        if stringify_has_spread_arguments(call_expr) {
            return;
        }

        ctx.diagnostic_with_suggestion(
            prefer_response_static_json_diagnostic(call_expr.callee.span()),
            |fixer| {
                let fixer = fixer.for_multifix();
                let mut fix = fixer.new_fix_with_capacity(7);

                let inner_callee = new_expr.callee.get_inner_expression();

                fix.push(fixer.insert_text_after(inner_callee, ".json"));

                let new_keyword_end = new_expr.span.start + 3; // "new" is 3 chars
                let callee_start = new_expr.callee.span().start;

                if ctx.has_comments_between(Span::new(new_keyword_end, callee_start)) {
                    fix.push(fixer.insert_text_before_range(new_expr.span, "( "));
                    fix.push(
                        fixer.delete_range(Span::new(new_expr.span.start, new_keyword_end + 1)),
                    );
                    fix.push(fixer.insert_text_after_range(new_expr.span, ")"));
                } else {
                    let new_keyword_span =
                        Span::new(new_expr.span.start, new_expr.callee.span().start);
                    fix.push(fixer.delete_range(new_keyword_span));
                }

                let Some(data_arg) = call_expr.arguments.first() else {
                    return fixer.noop();
                };
                let Some(data_expr) = data_arg.as_expression() else {
                    return fixer.noop();
                };

                let data_span = data_expr.span();
                let stringify_call_span = argument_expr.span();

                let before_data_span = Span::new(stringify_call_span.start, data_span.start);
                fix.push(fixer.delete_range(before_data_span));

                let after_data_span = Span::new(data_span.end, stringify_call_span.end);
                fix.push(fixer.delete_range(after_data_span));

                if should_add_semicolon(node, new_expr, ctx) {
                    fix.push(fixer.insert_text_before_range(new_expr.span, ";"));
                }

                fix.with_message(
                    "Replace `new Response(JSON.stringify(...))` with `Response.json(...)`",
                )
            },
        );
    }
}

fn should_add_semicolon(node: &AstNode, new_expr: &NewExpression, ctx: &LintContext) -> bool {
    let parent = ctx.nodes().parent_node(node.id());
    let new_expr_is_parenthesized = matches!(parent.kind(), AstKind::ParenthesizedExpression(_));

    let callee_is_parenthesized = !matches!(new_expr.callee, Expression::Identifier(_));

    !new_expr_is_parenthesized && callee_is_parenthesized && could_be_asi_hazard(node, ctx)
}

fn stringify_has_spread_arguments(call_expr: &CallExpression) -> bool {
    call_expr.arguments.iter().any(oxc_ast::ast::Argument::is_spread)
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "Response.json(data)",
        "Response(JSON.stringify(data))",
        "new Response()",
        "new NotResponse(JSON.stringify(data))",
        "new Response(JSON.stringify(...data))",
        "new Response(JSON.stringify())",
        "new Response(JSON.stringify(data, extraArgument))",
        "new Response(JSON.stringify?.(data))",
        "new Response(JSON?.stringify(data))",
        "new Response(new JSON.stringify(data))",
        "new Response(JSON.not_stringify(data))",
        "new Response(NOT_JSON.stringify(data))",
        "new Response(data(JSON.stringify))",
        r#"new Response("" + JSON.stringify(data))"#,
    ];

    let fail = vec![
        "new Response(JSON.stringify(data))",
        "new Response(JSON.stringify(data), extraArgument)",
        "new Response( (( JSON.stringify( (( 0, data )), ) )), )",
        "function foo() {
				return new // comment
					Response(JSON.stringify(data))
			}",
        "new Response(JSON.stringify(data), {status: 200})",
        "foo
			new (( Response ))(JSON.stringify(data))",
        "foo;
			new (( Response ))(JSON.stringify(data))",
        "foo;
			(( new (( Response ))(JSON.stringify(data)) ))",
        "foo
			(( new (( Response ))(JSON.stringify(data)) ))",
    ];

    let fix = vec![
        ("new Response(JSON.stringify(data))", "Response.json(data)"),
        ("new Response(JSON.stringify(data), extraArgument)", "Response.json(data, extraArgument)"),
        (
            "new Response( (( JSON.stringify( (( 0, data )), ) )), )",
            "Response.json( (( 0, data )), )",
        ),
        (
            "function foo() {
				return new // comment
					Response(JSON.stringify(data))
			}",
            "function foo() {
				return ( // comment
					Response.json(data))
			}",
        ),
        ("new Response(JSON.stringify(data), {status: 200})", "Response.json(data, {status: 200})"),
        (
            "foo
			new (( Response ))(JSON.stringify(data))",
            "foo
			;(( Response.json ))(data)",
        ),
        (
            "foo;
			new (( Response ))(JSON.stringify(data))",
            "foo;
			(( Response.json ))(data)",
        ),
        (
            "foo;
			(( new (( Response ))(JSON.stringify(data)) ))",
            "foo;
			(( (( Response.json ))(data) ))",
        ),
        (
            "foo
			(( new (( Response ))(JSON.stringify(data)) ))",
            "foo
			(( (( Response.json ))(data) ))",
        ),
    ];

    Tester::new(PreferResponseStaticJson::NAME, PreferResponseStaticJson::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
