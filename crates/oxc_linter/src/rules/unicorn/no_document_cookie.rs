use oxc_ast::{
    ast::{match_member_expression, Expression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::get_declaration_of_variable, context::LintContext, globals::GLOBAL_OBJECT_NAMES,
    rule::Rule, AstNode,
};

fn no_document_cookie_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `document.cookie` directly")
        .with_help("Use the Cookie Store API or a cookie library instead")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDocumentCookie;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow direct use of
    /// [`document.cookie`](https://developer.mozilla.org/en-US/docs/Web/API/Document/cookie).
    ///
    /// ### Why is this bad?
    ///
    /// It's not recommended to use
    /// [`document.cookie`](https://developer.mozilla.org/en-US/docs/Web/API/Document/cookie)
    /// directly as it's easy to get the string wrong. Instead, you should use
    /// the [Cookie Store
    /// API](https://developer.mozilla.org/en-US/docs/Web/API/Cookie_Store_API)
    /// or a [cookie library](https://www.npmjs.com/search?q=cookie).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// document.cookie =
    ///     'foo=bar' +
    ///     '; Path=/' +
    ///     '; Domain=example.com' +
    ///     '; expires=Fri, 31 Dec 9999 23:59:59 GMT' +
    ///     '; Secure';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// async function storeCookies() {
    ///     await cookieStore.set({
    ///         name: 'foo',
    ///         value: 'bar',
    ///         expires: Date.now() + 24 * 60 * 60 * 1000,
    ///         domain: 'example.com'
    ///     });
    /// }
    /// ```
    NoDocumentCookie,
    unicorn,
    correctness
);

impl Rule for NoDocumentCookie {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::AssignmentExpression(assignment_expr) = node.kind() else {
            return;
        };

        let Some(ident) = assignment_expr.left.as_member_expression() else {
            return;
        };

        let Some(static_prop_name) = ident.static_property_name() else {
            return;
        };

        if static_prop_name != "cookie" {
            return;
        }

        if !is_document_cookie_reference(ident.object(), ctx) {
            return;
        }

        ctx.diagnostic(no_document_cookie_diagnostic(assignment_expr.left.span()));
    }
}

fn is_document_cookie_reference<'a, 'b>(
    expr: &'a Expression<'b>,
    ctx: &'a LintContext<'b>,
) -> bool {
    match expr {
        Expression::Identifier(ident) => {
            if ident.name.as_str() != "document" {
                let Some(var_decl) = get_declaration_of_variable(ident, ctx) else {
                    return false;
                };

                let AstKind::VariableDeclarator(var_decl) = var_decl.kind() else {
                    return false;
                };

                let Some(init) = &var_decl.init else {
                    return false;
                };

                return is_document_cookie_reference(init, ctx);
            }
            true
        }
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            let Some(static_prop_name) = member_expr.static_property_name() else {
                return false;
            };
            if static_prop_name != "document" {
                return false;
            }

            if let Expression::Identifier(ident) = member_expr.object().without_parentheses() {
                if !GLOBAL_OBJECT_NAMES.contains(ident.name.as_str()) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"document.cookie",
        r"const foo = document.cookie",
        r"foo = document.cookie",
        r"foo = document?.cookie",
        r#"foo = document.cookie + ";foo=bar""#,
        r"delete document.cookie",
        r#"if (document.cookie.includes("foo")){}"#,
        r#"Object.assign(document, {cookie: "foo=bar"})"#,
        r#"document[CONSTANTS_COOKIE] = "foo=bar""#,
        r#"document[cookie] = "foo=bar""#,
    ];

    let fail = vec![
        r#"document.cookie = "foo=bar""#,
        r#"document.cookie += ";foo=bar""#,
        r#"document.cookie = document.cookie + ";foo=bar""#,
        r"document.cookie &&= true",
        // r#"document["coo" + "kie"] = "foo=bar""#,
        r#"foo = document.cookie = "foo=bar""#,
        r#"var doc = document; doc.cookie = "foo=bar""#,
        r#"let doc = document; doc.cookie = "foo=bar""#,
        r#"const doc = globalThis.document; doc.cookie = "foo=bar""#,
        r#"window.document.cookie = "foo=bar""#,
    ];

    Tester::new(NoDocumentCookie::NAME, NoDocumentCookie::PLUGIN, pass, fail).test_and_snapshot();
}
