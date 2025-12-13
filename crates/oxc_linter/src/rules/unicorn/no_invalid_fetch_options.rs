use std::borrow::Cow;

use crate::{LintContext, ast_util::is_new_expression, rule::Rule};
use cow_utils::CowUtils;
use oxc_allocator::Box;
use oxc_ast::{
    AstKind,
    ast::{
        Argument, BindingPattern, Expression, FormalParameter, ObjectExpression,
        ObjectPropertyKind, PropertyKey, TSLiteral, TSLiteralType, TSType, TSTypeAnnotation,
        TemplateLiteral,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{CompactStr, Span};

fn no_invalid_fetch_options_diagnostic(span: Span, method: &str) -> OxcDiagnostic {
    let message = format!(r#""body" is not allowed when method is "{method}""#);

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
        match node.kind() {
            AstKind::CallExpression(call_expr) => {
                if !call_expr.callee.is_specific_id("fetch") || call_expr.arguments.len() < 2 {
                    return;
                }

                if let Argument::ObjectExpression(expr) = &call_expr.arguments[1]
                    && let Some((method_name, body_span)) = is_invalid_fetch_options(expr, ctx)
                {
                    ctx.diagnostic(no_invalid_fetch_options_diagnostic(body_span, &method_name));
                }
            }
            AstKind::NewExpression(new_expr) => {
                if !is_new_expression(new_expr, &["Request"], Some(2), None) {
                    return;
                }

                if let Argument::ObjectExpression(expr) = &new_expr.arguments[1]
                    && let Some((method_name, body_span)) = is_invalid_fetch_options(expr, ctx)
                {
                    ctx.diagnostic(no_invalid_fetch_options_diagnostic(body_span, &method_name));
                }
            }
            _ => {}
        }
    }
}

// set to method_name to "UNKNOWN" if we can't infer the method name
const UNKNOWN_METHOD_NAME: Cow<'static, str> = Cow::Borrowed("UNKNOWN");

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
            match &obj_prop.value {
                Expression::StaticMemberExpression(s) => {
                    let symbols = ctx.scoping();
                    let Expression::Identifier(ident_ref) = &s.object else {
                        method_name = UNKNOWN_METHOD_NAME;
                        continue;
                    };
                    let reference_id = ident_ref.reference_id();
                    // Check if reference is to an enum and if so then get the string literal initialiser
                    // for the enum value being referenced.
                    let reference = symbols.get_reference(reference_id);

                    if let Some(symbol_id) = reference.symbol_id()
                        && ctx.scoping().symbol_flags(symbol_id).is_enum()
                    {
                        let decl = ctx.semantic().symbol_declaration(symbol_id);
                        let enum_member_res: Option<CompactStr> = match decl.kind() {
                            AstKind::TSEnumDeclaration(enum_decl) => {
                                let member_string_lit: Option<CompactStr> =
                                    enum_decl.body.members.iter().find_map(|m| {
                                        if let Some(Expression::StringLiteral(str_lit)) =
                                            &m.initializer
                                        {
                                            Some(str_lit.value.to_compact_str())
                                        } else {
                                            None
                                        }
                                    });
                                member_string_lit
                            }
                            _ => None,
                        };

                        if let Some(value_ident) = enum_member_res {
                            method_name = value_ident.into();
                        }
                    } else {
                        method_name = UNKNOWN_METHOD_NAME;
                    }
                }
                Expression::StringLiteral(value_ident) => {
                    method_name = value_ident.value.cow_to_ascii_uppercase();
                }
                Expression::TemplateLiteral(template_lit) => {
                    method_name = extract_method_name_from_template_literal(template_lit);
                }
                Expression::Identifier(value_ident) => {
                    let symbols = ctx.scoping();
                    let reference_id = value_ident.reference_id();

                    let Some(symbol_id) = symbols.get_reference(reference_id).symbol_id() else {
                        method_name = UNKNOWN_METHOD_NAME;
                        continue;
                    };

                    let decl = ctx.nodes().get_node(symbols.symbol_declaration(symbol_id));

                    match decl.kind() {
                        AstKind::VariableDeclarator(declarator) => match &declarator.init {
                            Some(Expression::StringLiteral(str_lit)) => {
                                method_name = str_lit.value.cow_to_ascii_uppercase();
                            }
                            Some(Expression::TemplateLiteral(template_lit)) => {
                                method_name =
                                    extract_method_name_from_template_literal(template_lit);
                            }
                            _ => {
                                method_name = UNKNOWN_METHOD_NAME;
                            }
                        },
                        AstKind::FormalParameter(FormalParameter {
                            pattern: BindingPattern { type_annotation: Some(annotation), .. },
                            ..
                        }) => {
                            let TSTypeAnnotation { type_annotation, .. } = &**annotation;
                            match type_annotation {
                                TSType::TSUnionType(union_type) => {
                                    if !union_type.types.iter().any(|ty| {
                                        if let TSType::TSLiteralType(ty) = ty {
                                            let TSLiteralType { literal, .. } = &**ty;
                                            if let TSLiteral::StringLiteral(str_lit) = literal {
                                                return str_lit.value.cow_to_ascii_uppercase()
                                                    == "GET"
                                                    || str_lit.value.cow_to_ascii_uppercase()
                                                        == "HEAD";
                                            }
                                        }
                                        false
                                    }) {
                                        method_name = UNKNOWN_METHOD_NAME;
                                    }
                                }
                                TSType::TSLiteralType(literal_type) => {
                                    let TSLiteralType { literal, .. } = &**literal_type;
                                    if let TSLiteral::StringLiteral(str_lit) = literal {
                                        method_name = str_lit.value.cow_to_ascii_uppercase();
                                    }
                                }
                                _ => {
                                    method_name = UNKNOWN_METHOD_NAME;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {
                    method_name = UNKNOWN_METHOD_NAME;
                }
            }
        }
    }

    if (method_name == "GET" || method_name == "HEAD") && !body_span.is_empty() {
        Some((method_name, body_span))
    } else {
        None
    }
}

fn extract_method_name_from_template_literal<'a>(
    template_lit: &'a TemplateLiteral<'a>,
) -> Cow<'a, str> {
    if let Some(template_element_value) = template_lit.quasis.first() {
        // only one template element
        if template_element_value.tail {
            return template_element_value.value.raw.cow_to_ascii_uppercase();
        }
    }
    UNKNOWN_METHOD_NAME
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
        r#"const method = `post`; fetch(url, {method, body: "foo=bar"})"#,
        r#"const method = `po${"st"}`; fetch(url, {method, body: "foo=bar"})"#,
        r#"function foo(method: "POST" | "PUT", body: string) {
            return new Request(url, {method, body});
        }"#,
        "function foo(method: string, body: string) {
            return new Request(url, {method, body});
        }",
        r#"enum Method {
          Post = "POST",
        }
        const response = await fetch("/", {
         method: Method.Post,
         body: "",
        });"#,
        ("const response = await fetch('', { method, headers, body, });"),
        (r#"fetch("/url", { method: logic ? "PATCH" : "POST", body: "some body" });"#),
        (r#"new Request("/url", { method: logic ? "PATCH" : "POST", body: "some body" });"#),
        (r#"fetch("/url", { method: getMethod(), body: "some body" });"#),
        (r"const method = 'POST' as const; await fetch('some-url', { method, body: '' });"),
        (r"const options = { method: 'POST' } as const; await fetch('some-url', { method: options.method, body: '' });"),
        (r"const options = { method: 'POST' }; await fetch('some-url', { method: options.method, body: '' });"),
        (r"const options = { method: 'POST' } as const; new Request('some-url', { method: options.method, body: '' });"),
        (r#"fetch("/url", { method: getOptions().method, body: "some body" });"#),
        (r#"new Request("/url", { method: getOptions().method, body: "some body" });"#),
        (r#"fetch("/url", { method: (options).method, body: "some body" });"#),
        (r#"new Request("/url", { method: (options).method, body: "some body" });"#),
    ];

    let fail = vec![
        r"fetch(url, {body})",
        r"new Request(url, {body})",
        r#"fetch(url, {method: "GET", body})"#,
        r#"new Request(url, {method: "GET", body})"#,
        r#"fetch(url, {method: "HEAD", body})"#,
        r#"new Request(url, {method: "HEAD", body})"#,
        r#"fetch(url, {method: "head", body})"#,
        r#"fetch(url, {method: `head`, body: "foo=bar"})"#,
        r#"new Request(url, {method: "head", body})"#,
        r#"const method = "head"; new Request(url, {method, body: "foo=bar"})"#,
        r#"const method = "head"; fetch(url, {method, body: "foo=bar"})"#,
        r#"const method = `head`; fetch(url, {method, body: "foo=bar"})"#,
        r"fetch(url, {body}, extraArgument)",
        r"new Request(url, {body}, extraArgument)",
        r#"fetch(url, {body: undefined, body: "foo=bar"});"#,
        r#"new Request(url, {body: undefined, body: "foo=bar"});"#,
        r#"fetch(url, {method: "post", body: "foo=bar", method: "HEAD"});"#,
        r#"new Request(url, {method: "post", body: "foo=bar", method: "HEAD"});"#,
        r#"fetch('/', {body: new URLSearchParams({ data: "test" })})"#,
        r#"function foo(method: "HEAD" | "GET") {
            return new Request(url, {method, body: ""});
        }"#,
        r#"enum Method {
            Get = "GET",
          }
          const response = await fetch("/", {
           method: Method.Get,
           body: "",
          });"#,
        r#"enum Method {
            Foo = "GET",
          }
          const response = await fetch("/", {
           method: Method.Foo,
           body: "",
          });"#,
    ];

    Tester::new(NoInvalidFetchOptions::NAME, NoInvalidFetchOptions::PLUGIN, pass, fail)
        .test_and_snapshot();
}
