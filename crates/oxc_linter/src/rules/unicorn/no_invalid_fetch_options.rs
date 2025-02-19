use std::borrow::Cow;

use cow_utils::CowUtils;
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

fn no_invalid_fetch_options_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    let message = format!("The `body` is not allowed when method is `{method}`");

    OxcDiagnostic::warn(message).with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoInvalidFetchOptions;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow invalid options in `fetch()` and `new Request()`. Specifically, this rule ensures that
    /// a body is not provided when the method is `GET` or `HEAD`, as it will result in a `TypeError`.
    ///
    /// ### Why is this bad?
    ///
    /// The `fetch()` function throws a `TypeError` when the method is `GET` or `HEAD` and a body is provided.
    /// This can lead to unexpected behavior and errors in your code. By disallowing such invalid options,
    /// the rule ensures that requests are correctly configured and prevents unnecessary errors.
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
        let arg = match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if !call_expr.callee.is_specific_id("fetch") || call_expr.arguments.len() < 2 {
                    return;
                }

                &call_expr.arguments[1]
            }
            AstKind::NewExpression(new_expr) => {
                if !is_new_expression(new_expr, &["Request"], Some(2), None) {
                    return;
                }

                &new_expr.arguments[1]
            }
            _ => return,
        };

        let Argument::ObjectExpression(expr) = arg else { return };
        let result = is_invalid_fetch_options(expr, ctx);

        if let Some((method_name, body_span)) = result {
            ctx.diagnostic(no_invalid_fetch_options_diagnostic(body_span, &method_name));
        }
    }
}

fn is_invalid_fetch_options<'a>(
    obj_expr: &'a Box<'_, ObjectExpression<'_>>,
    ctx: &'a LintContext<'_>,
) -> Option<(Cow<'a, str>, Span)> {
    // fetch and Request method defaults to "GET"
    let mut body_span = Span::default();
    let mut method_name = Cow::Borrowed("GET");

    for property in &obj_expr.properties {
        if property.is_spread() {
            return None;
        }

        let ObjectPropertyKind::ObjectProperty(obj_prop) = property else {
            continue;
        };
        let PropertyKey::StaticIdentifier(key_ident) = &obj_prop.key else {
            continue;
        };

        let key_ident_name = key_ident.name.as_str();

        if key_ident_name == "body" {
            if obj_prop.value.is_null_or_undefined() {
                body_span.end = body_span.start;
            } else {
                body_span = key_ident.span;
            }
        } else if key_ident_name == "method" {
            let method = match &obj_prop.value {
                Expression::StringLiteral(value_ident) => &value_ident.value,
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

                    let Some(Expression::StringLiteral(str_lit)) = &declarator.init else {
                        continue;
                    };

                    &str_lit.value
                }
                _ => continue,
            };

            method_name = method.cow_to_ascii_uppercase();
        }
    }

    if (method_name == "GET" || method_name == "HEAD") && !body_span.is_empty() {
        Some((method_name, body_span))
    } else {
        None
    }
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
