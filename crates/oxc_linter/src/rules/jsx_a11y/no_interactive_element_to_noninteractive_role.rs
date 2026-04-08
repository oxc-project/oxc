use oxc_ast::AstKind;
use oxc_ast::ast::{JSXAttributeItem, JSXAttributeValue, JSXElementName};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule, utils::has_jsx_prop_ignore_case};

fn diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Interactive element should not be assigned a non-interactive role.")
        .with_help("Interactive HTML elements like `<button>`, `<a>`, and `<input>` should not be given non-interactive roles like `article` or `presentation`.")
        .with_label(span)
}

const INTERACTIVE_ELEMENTS: &[&str] =
    &["a", "button", "input", "select", "textarea", "details", "summary"];

const NON_INTERACTIVE_ROLES: &[&str] = &[
    "article",
    "banner",
    "complementary",
    "contentinfo",
    "definition",
    "directory",
    "document",
    "feed",
    "figure",
    "group",
    "heading",
    "img",
    "list",
    "listitem",
    "main",
    "math",
    "navigation",
    "none",
    "note",
    "presentation",
    "region",
    "separator",
    "status",
    "term",
    "toolbar",
];

#[derive(Debug, Default, Clone)]
pub struct NoInteractiveElementToNoninteractiveRole;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows assigning non-interactive ARIA roles to interactive HTML elements.
    ///
    /// ### Why is this bad?
    ///
    /// Interactive elements like `<button>` have built-in interaction behaviors.
    /// Assigning a non-interactive role overrides the native semantics, which
    /// confuses assistive technologies.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <button role="article" />
    /// <a href="#" role="presentation" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <button />
    /// <div role="article" />
    /// ```
    NoInteractiveElementToNoninteractiveRole,
    jsx_a11y,
    correctness,
    pending
);

impl Rule for NoInteractiveElementToNoninteractiveRole {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_name = match &jsx_el.name {
            JSXElementName::Identifier(id) => id.name.as_str(),
            JSXElementName::IdentifierReference(id) => id.name.as_str(),
            _ => return,
        };

        if !INTERACTIVE_ELEMENTS.contains(&element_name) {
            return;
        }

        // Check for role attribute
        let Some(JSXAttributeItem::Attribute(attr)) = has_jsx_prop_ignore_case(jsx_el, "role")
        else {
            return;
        };

        let Some(JSXAttributeValue::StringLiteral(role_value)) = &attr.value else {
            return;
        };

        let role = role_value.value.as_str().to_lowercase();
        if NON_INTERACTIVE_ROLES.contains(&role.as_str()) {
            ctx.diagnostic(diagnostic(attr.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "<button />",
        "<div role=\"article\" />",
        "<button role=\"button\" />",
        "<a href=\"#\" />",
    ];

    let fail = vec![
        "<button role=\"article\" />",
        "<button role=\"presentation\" />",
        "<a href=\"#\" role=\"none\" />",
    ];

    Tester::new(
        NoInteractiveElementToNoninteractiveRole::NAME,
        NoInteractiveElementToNoninteractiveRole::PLUGIN,
        pass,
        fail,
    )
    .test_and_snapshot();
}
