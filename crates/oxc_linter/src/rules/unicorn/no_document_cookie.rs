use oxc_ast::{
    AstKind,
    ast::{Expression, match_member_expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use rustc_hash::FxHashSet;

use crate::{
    AstNode, ast_util::get_symbol_id_of_variable, context::LintContext,
    globals::GLOBAL_OBJECT_NAMES, rule::Rule,
};

fn no_document_cookie_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Do not use `document.cookie` directly.")
        .with_help("Use the Cookie Store API or a cookie library instead.")
        .with_note("https://developer.mozilla.org/en-US/docs/Web/API/Cookie_Store_API")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoDocumentCookie;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows direct use of
    /// [`document.cookie`](https://developer.mozilla.org/en-US/docs/Web/API/Document/cookie).
    ///
    /// ### Why is this bad?
    ///
    /// It's not recommended to use
    /// [`document.cookie`](https://developer.mozilla.org/en-US/docs/Web/API/Document/cookie)
    /// directly as it's easy to get the string wrong. Instead, you should use
    /// the [Cookie Store
    /// API](https://developer.mozilla.org/en-US/docs/Web/API/Cookie_Store_API)
    /// or a [cookie library](https://npmx.dev/search?q=cookie).
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
    restriction,
    recommended,
    version = "0.0.18",
    short_description = "Disallows direct use of `document.cookie`.",
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
    mut expr: &'a Expression<'b>,
    ctx: &'a LintContext<'b>,
) -> bool {
    let mut visited = FxHashSet::default();
    while let Expression::Identifier(ident) = expr {
        if ident.name.as_str() == "document" {
            return true;
        }

        let Some(symbol_id) = get_symbol_id_of_variable(ident, ctx) else {
            return false;
        };
        if !visited.insert(symbol_id) {
            return false;
        }

        let declaration_id = ctx.scoping().symbol_declaration(symbol_id);
        let AstKind::VariableDeclarator(var_decl) = ctx.nodes().kind(declaration_id) else {
            return false;
        };

        let Some(init) = &var_decl.init else {
            return false;
        };
        expr = init;
    }

    match expr {
        match_member_expression!(Expression) => {
            let member_expr = expr.to_member_expression();
            let Some(static_prop_name) = member_expr.static_property_name() else {
                return false;
            };
            if static_prop_name != "document" {
                return false;
            }

            if let Expression::Identifier(ident) = member_expr.object().without_parentheses()
                && !GLOBAL_OBJECT_NAMES.contains(&ident.name.as_str())
            {
                return false;
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
        "document.cookie",
        "const foo = document.cookie",
        "foo = document.cookie",
        "foo = document?.cookie",
        r#"foo = document.cookie + ";foo=bar""#,
        "delete document.cookie",
        r#"if (document.cookie.includes("foo")){}"#,
        r#"Object.assign(document, {cookie: "foo=bar"})"#,
        r#"document[CONSTANTS_COOKIE] = "foo=bar""#,
        r#"document[cookie] = "foo=bar""#,
        r#"const CONSTANTS_COOKIE = "cookie";
            document[CONSTANTS_COOKIE] = "foo=bar";"#,
        "var a = a; a.cookie = 0;",
        "var a = b; var b = a; a.cookie = 0;",
        "var a = b; var b = c; var c = a; a.cookie = 0;",
        "var a = b; var b = b; a.cookie = 0;",
    ];

    let fail = vec![
        r#"document.cookie = "foo=bar""#,
        r#"document.cookie += ";foo=bar""#,
        r#"document.cookie = document.cookie + ";foo=bar""#,
        "document.cookie &&= true",
        // r#"document["coo" + "kie"] = "foo=bar""#,
        r#"foo = document.cookie = "foo=bar""#,
        r#"var doc = document; doc.cookie = "foo=bar""#,
        r#"let doc = document; doc.cookie = "foo=bar""#,
        r#"const doc = globalThis.document; doc.cookie = "foo=bar""#,
        r#"window.document.cookie = "foo=bar""#,
        "const doc = globalThis.document; { const alias = doc; { const doc = alias; doc.cookie = 0; } }",
        "var a = a; a.cookie = 0; document.cookie = 0;",
    ];

    Tester::new(NoDocumentCookie::NAME, NoDocumentCookie::PLUGIN, pass, fail).test_and_snapshot();
}
