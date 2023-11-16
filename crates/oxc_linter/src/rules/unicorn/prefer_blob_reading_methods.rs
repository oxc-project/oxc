use oxc_ast::{ast::Expression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-blob-reading-methods): Prefer `Blob#{1}()` over `FileReader#{2}(blob)`.")]
#[diagnostic(severity(warning))]
struct PreferBlobReadingMethodsDiagnostic(#[label] pub Span, pub &'static str, pub &'static str);

#[derive(Debug, Default, Clone)]
pub struct PreferBlobReadingMethods;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Recommends using `Blob#text()` and `Blob#arrayBuffer()` over `FileReader#readAsText()` and `FileReader#readAsArrayBuffer()`.
    ///
    /// ### Why is this bad?
    ///
    /// `FileReader` predates promises, and the newer [`Blob#arrayBuffer()`](https://developer.mozilla.org/en-US/docs/Web/API/Blob/arrayBuffer) and [`Blob#text()`](https://developer.mozilla.org/en-US/docs/Web/API/Blob/text) methods are much cleaner and easier to use.
    ///
    /// ### Example
    /// ```javascript
    /// // bad
    /// const arrayBuffer = await new Promise((resolve, reject) => {
    /// 	const fileReader = new FileReader();
    /// 	fileReader.addEventListener('load', () => {
    /// 		resolve(fileReader.result);
    /// 	});
    /// 	fileReader.addEventListener('error', () => {
    /// 		reject(fileReader.error);
    /// 	});
    /// 	fileReader.readAsArrayBuffer(blob);
    /// });
    ///
    /// // good
    /// const arrayBuffer = await blob.arrayBuffer();
    /// ```
    PreferBlobReadingMethods,
    pedantic
);

impl Rule for PreferBlobReadingMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else { return };

        let Expression::MemberExpression(member_expr) = &call_expr.callee else { return };

        if member_expr.is_computed()
            || member_expr.optional()
            || call_expr.optional
            || call_expr.arguments.len() != 1
        {
            return;
        }

        let (current, replacement, span) = match member_expr.static_property_info() {
            Some((span, "readAsText")) => ("readAsText", "text", span),
            Some((span, "readAsArrayBuffer")) => ("readAsArrayBuffer", "arrayBuffer", span),
            _ => return,
        };

        ctx.diagnostic(PreferBlobReadingMethodsDiagnostic(span, replacement, current));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"blob.arrayBuffer()",
        r"blob.text()",
        r"new Response(blob).arrayBuffer()",
        r"new Response(blob).text()",
        r"fileReader.readAsDataURL(blob)",
        r"fileReader.readAsBinaryString(blob)",
        r#"fileReader.readAsText(blob, "ascii")"#,
    ];

    let fail = vec![r"fileReader.readAsArrayBuffer(blob)", r"fileReader.readAsText(blob)"];

    Tester::new_without_config(PreferBlobReadingMethods::NAME, pass, fail).test_and_snapshot();
}
