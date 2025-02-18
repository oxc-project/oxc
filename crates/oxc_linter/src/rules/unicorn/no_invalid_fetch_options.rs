use oxc_allocator::Box;
use oxc_ast::{
    ast::{Argument, Expression, ObjectExpression, ObjectPropertyKind, PropertyKey},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::Span;

use crate::{ast_util::is_new_expression, rule::Rule, LintContext};

fn no_invalid_fetch_options_diagnostic(span: Span, method: Option<String>) -> OxcDiagnostic {
    let method = method.unwrap_or("GET/HEAD".to_string()).to_uppercase();
    let message = format!(r#""body" is not allowed when method is "{method}""#);

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInvalidFetchOptions;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow invalid options in fetch() and new Request()
    ///
    /// ### Why is this bad?
    /// fetch() throws a TypeError when the method is GET or HEAD and a body is provided.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// const response = await fetch('/', {method: 'GET', body: 'foo=bar'});
    ///
    /// const request = new Request('/', {method: 'GET', body: 'foo=bar'});
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// const response = await fetch('/', {method: 'POST', body: 'foo=bar'});
    ///
    /// const request = new Request('/', {method: 'POST', body: 'foo=bar'});
    /// ```
    NoInvalidFetchOptions,
    unicorn,
    correctness,
);

impl Rule for NoInvalidFetchOptions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };

                if ident.name != "fetch" {
                    return;
                }

                if call_expr.arguments.len() < 2 {
                    return;
                }

                let Some(Argument::ObjectExpression(obj_expr)) = call_expr.arguments.get(1) else {
                    return;
                };

                let (is_invalid_options, method_name, body_span) =
                    is_invalid_fetch_options(obj_expr, ctx);

                if is_invalid_options {
                    ctx.diagnostic(no_invalid_fetch_options_diagnostic(
                        body_span.unwrap_or(call_expr.span),
                        method_name,
                    ));
                }
            }
            AstKind::NewExpression(new_expr) => {
                if !is_new_expression(new_expr, &["Request"], Some(2), None) {
                    return;
                }

                let Some(Argument::ObjectExpression(obj_expr)) = new_expr.arguments.get(1) else {
                    return;
                };

                let (is_invalid_options, method_name, body_span) =
                    is_invalid_fetch_options(obj_expr, ctx);

                if is_invalid_options {
                    ctx.diagnostic(no_invalid_fetch_options_diagnostic(
                        body_span.unwrap_or(new_expr.span),
                        method_name,
                    ));
                }
            }
            _ => {}
        }
    }
}

fn is_invalid_fetch_options(
    obj_expr: &Box<'_, ObjectExpression<'_>>,
    ctx: &LintContext<'_>,
) -> (bool, Option<String>, Option<Span>) {
    // fetch and Request method defaults to "GET"
    let mut is_get_or_head = true;
    let mut method_name = "GET";

    let mut has_body = false;
    let mut body_span = None;

    for property in &obj_expr.properties {
        if ObjectPropertyKind::is_spread(property) {
            return (false, None, None);
        }

        let ObjectPropertyKind::ObjectProperty(obj_prop) = property else {
            continue;
        };

        let PropertyKey::StaticIdentifier(key_ident) = &obj_prop.key else {
            continue;
        };

        let key_ident_name = key_ident.name.as_str();

        if key_ident_name == "body" {
            has_body = !obj_prop.value.is_null_or_undefined();

            if has_body {
                body_span = Some(key_ident.span);
            } else {
                body_span = None;
            }
        }

        if key_ident_name == "method" {
            match &obj_prop.value {
                Expression::StringLiteral(value_ident) => {
                    let method = value_ident.value.as_str();

                    is_get_or_head = !is_not_get_and_head(method);
                    method_name = method;
                }
                Expression::Identifier(value_ident) => {
                    let symbols = ctx.semantic().symbols();
                    let reference_id = value_ident.reference_id();

                    let Some(symbol_id) = symbols.get_reference(reference_id).symbol_id() else {
                        continue;
                    };

                    let decl = ctx.semantic().nodes().get_node(symbols.get_declaration(symbol_id));

                    let AstKind::VariableDeclarator(declarator) = decl.kind() else {
                        continue;
                    };

                    let Some(init) = &declarator.init else {
                        continue;
                    };

                    let Expression::StringLiteral(str_lit) = init else {
                        continue;
                    };

                    is_get_or_head = !is_not_get_and_head(&str_lit.value);
                    method_name = &str_lit.value;
                }
                _ => continue,
            };
        }
    }

    let is_invalid_options = is_get_or_head && has_body;

    (is_invalid_options, Some(method_name.to_string()), body_span)
}

fn is_not_get_and_head(method: &str) -> bool {
    let method = method.to_uppercase();
    method != "GET" && method != "HEAD"
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"fetch(url, {method: "POST", body})"#,
        r#"new Request(url, {method: "POST", body})"#,
        r"fetch(url, {})",
        r"new Request(url, {})",
        r"fetch(url)",
        r"new Request(url)",
        r#"fetch(url, {method: "UNKNOWN", body})"#,
        r#"new Request(url, {method: "UNKNOWN", body})"#,
        r"fetch(url, {body: undefined})",
        r"new Request(url, {body: undefined})",
        r"fetch(url, {body: null})",
        r"new Request(url, {body: null})",
        r"fetch(url, {...options, body})",
        r"new Request(url, {...options, body})",
        r"new fetch(url, {body})",
        r"Request(url, {body})",
        r"not_fetch(url, {body})",
        r"new not_Request(url, {body})",
        r"fetch({body}, url)",
        r"new Request({body}, url)",
        r#"fetch(url, {[body]: "foo=bar"})"#,
        r#"new Request(url, {[body]: "foo=bar"})"#,
        r#"fetch(url, {body: "foo=bar", body: undefined});"#,
        r#"new Request(url, {body: "foo=bar", body: undefined});"#,
        r#"fetch(url, {method: "HEAD", body: "foo=bar", method: "post"});"#,
        r#"new Request(url, {method: "HEAD",body: "foo=bar", method: "POST"});"#,
        r#"fetch('/', {body: new URLSearchParams({ data: "test" }), method: "POST"})"#,
        r#"const method = "post"; new Request(url, {method, body: "foo=bar"})"#,
        r#"const method = "post"; fetch(url, {method, body: "foo=bar"})"#,
    ];

    let fail = vec![
        r"fetch(url, {body})",
        r"new Request(url, {body})",
        r#"fetch(url, {method: "GET", body})"#,
        r#"new Request(url, {method: "GET", body})"#,
        r#"fetch(url, {method: "HEAD", body})"#,
        r#"new Request(url, {method: "HEAD", body})"#,
        r#"fetch(url, {method: "head", body})"#,
        r#"new Request(url, {method: "head", body})"#,
        r#"const method = "head"; new Request(url, {method, body: "foo=bar"})"#,
        r#"const method = "head"; fetch(url, {method, body: "foo=bar"})"#,
        r"fetch(url, {body}, extraArgument)",
        r"new Request(url, {body}, extraArgument)",
        r#"fetch(url, {body: undefined, body: "foo=bar"});"#,
        r#"new Request(url, {body: undefined, body: "foo=bar"});"#,
        r#"fetch(url, {method: "post", body: "foo=bar", method: "HEAD"});"#,
        r#"new Request(url, {method: "post", body: "foo=bar", method: "HEAD"});"#,
        r#"fetch('/', {body: new URLSearchParams({ data: "test" })})"#,
    ];

    Tester::new(NoInvalidFetchOptions::NAME, NoInvalidFetchOptions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
