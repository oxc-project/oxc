use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::NodeId;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

fn no_multi_str_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected multi string.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoMultiStr;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow multiline strings.
    ///
    /// ### Why is this bad?
    ///
    /// Some consider this to be a bad practice as it was an undocumented feature of JavaScript
    /// that was only formalized later.
    ///
    /// ### Example
    /// ```javascript
    /// var x = "Line 1 \
    ///  Line 2";
    /// ```
    NoMultiStr,
    eslint,
    style,
);

impl Rule for NoMultiStr {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::StringLiteral(literal) = node.kind() {
            let source = literal.span.source_text(ctx.source_text());
            // https://github.com/eslint/eslint/blob/9e6d6405c3ee774c2e716a3453ede9696ced1be7/lib/shared/ast-utils.js#L12
            let position = source.find(['\r', '\n', '\u{2028}', '\u{2029}']).unwrap_or(0);
            if position != 0 && !is_within_jsx_attribute_item(node.id(), ctx) {
                // We found the "newline" character but want to highlight the '\', so go back one
                // character.
                let multi_span_start =
                    literal.span.start + u32::try_from(position).unwrap_or_default() - 1;
                ctx.diagnostic(no_multi_str_diagnostic(Span::new(
                    multi_span_start,
                    multi_span_start + 1,
                )));
            }
        }
    }
}

fn is_within_jsx_attribute_item(id: NodeId, ctx: &LintContext) -> bool {
    if matches!(ctx.nodes().parent_kind(id), Some(AstKind::JSXAttributeItem(_))) {
        return true;
    }
    false
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "var a = 'Line 1 Line 2';",
        "var a = <div>
			<h1>Wat</h1>
			</div>;", // { "ecmaVersion": 6, "parserOptions": { "ecmaFeatures": { "jsx": true } } }
        r#"<div class="line1
        line2"></div>"#, // jsx
    ];

    let fail = vec![
        "var x = 'Line 1 \\
			 Line 2'",
        "test('Line 1 \\
			 Line 2');",
        "'foo\\\rbar';",
        "'foo\\ bar';",
        "'foo\\ ar';",
        "'\\ still fails';",
    ];

    Tester::new(NoMultiStr::NAME, NoMultiStr::PLUGIN, pass, fail).test_and_snapshot();
}
