use oxc_ast::{
    AstKind,
    ast::{CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    ast_util::{is_method_call, is_new_expression},
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
    pending,
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

        ctx.diagnostic(prefer_response_static_json_diagnostic(call_expr.callee.span()));
    }
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

    Tester::new(PreferResponseStaticJson::NAME, PreferResponseStaticJson::PLUGIN, pass, fail)
        .test_and_snapshot();
}
