use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn prefer_blob_reading_methods_diagnostic(
    span: Span,
    good_method: &str,
    bad_method: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Prefer `Blob#{good_method}()` over `FileReader#{bad_method}(blob)`."
    ))
    .with_label(span)
}

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
    /// async function bad() {
    ///     const arrayBuffer = await new Promise((resolve, reject) => {
    ///         const fileReader = new FileReader();
    ///         fileReader.addEventListener('load', () => {
    ///             resolve(fileReader.result);
    ///         });
    ///         fileReader.addEventListener('error', () => {
    ///             reject(fileReader.error);
    ///         });
    ///         fileReader.readAsArrayBuffer(blob);
    ///     });
    /// }
    ///
    /// async function good() {
    ///     const arrayBuffer = await blob.arrayBuffer();
    /// }
    /// ```
    PreferBlobReadingMethods,
    unicorn,
    pedantic,
    pending
);

impl Rule for PreferBlobReadingMethods {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.as_member_expression() else {
            return;
        };

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

        ctx.diagnostic(prefer_blob_reading_methods_diagnostic(span, replacement, current));
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

    Tester::new(PreferBlobReadingMethods::NAME, PreferBlobReadingMethods::PLUGIN, pass, fail)
        .test_and_snapshot();
}
