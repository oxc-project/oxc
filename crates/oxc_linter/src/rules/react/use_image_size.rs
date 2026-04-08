use oxc_ast::AstKind;
use oxc_ast::ast::JSXElementName;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn use_image_size_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Image element is missing `width` and/or `height` attributes.")
        .with_help("Add explicit `width` and `height` attributes to `<img>` elements to prevent layout shifts.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct UseImageSize;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that `<img>` elements have explicit `width` and `height` attributes.
    ///
    /// ### Why is this bad?
    ///
    /// Images without explicit dimensions cause layout shifts as the browser
    /// doesn't know their size until loaded. This hurts Core Web Vitals (CLS).
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <img src="photo.jpg" />
    /// <img src="photo.jpg" width="100" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <img src="photo.jpg" width="100" height="100" />
    /// ```
    UseImageSize,
    react,
    correctness,
    pending
);

impl Rule for UseImageSize {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_name = match &jsx_el.name {
            JSXElementName::Identifier(id) => id.name.as_str(),
            JSXElementName::IdentifierReference(id) => id.name.as_str(),
            _ => return,
        };

        if element_name != "img" {
            return;
        }

        let has_width = has_jsx_prop_ignore_case(jsx_el, "width").is_some();
        let has_height = has_jsx_prop_ignore_case(jsx_el, "height").is_some();

        if !has_width || !has_height {
            ctx.diagnostic(use_image_size_diagnostic(jsx_el.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<img src="photo.jpg" width="100" height="100" />"#,
        r#"<img src="photo.jpg" width={100} height={100} />"#,
        "<div />",
    ];

    let fail = vec![
        r#"<img src="photo.jpg" />"#,
        r#"<img src="photo.jpg" width="100" />"#,
        r#"<img src="photo.jpg" height="100" />"#,
    ];

    Tester::new(UseImageSize::NAME, UseImageSize::PLUGIN, pass, fail).test_and_snapshot();
}
