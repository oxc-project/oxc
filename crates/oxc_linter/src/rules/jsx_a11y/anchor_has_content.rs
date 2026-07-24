use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXChild, JSXElement},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{Fix, RuleFix},
    rule::{DefaultRuleConfig, Rule},
    utils::{
        get_element_type, has_jsx_prop_ignore_case, is_hidden_from_screen_reader,
        object_has_accessible_child,
    },
};

fn missing_content(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing accessible content when using `a` elements.")
        .with_help("Provide screen reader accessible content when using `a` elements.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AnchorHasContent(Box<AnchorHasContentConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct AnchorHasContentConfig {
    /// Additional custom component names to treat as anchor elements.
    components: Vec<CompactStr>,
}

impl std::ops::Deref for AnchorHasContent {
    type Target = AnchorHasContentConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that anchors have content and that the content is accessible to screen readers.
    /// Accessible means that it is not hidden using the `aria-hidden` prop.
    ///
    /// Alternatively, you may use the `title` prop or the `aria-label` prop.
    ///
    /// ### Options
    ///
    /// This rule accepts a `components` option containing custom component names that should be
    /// checked in addition to the native `a` element.
    ///
    /// ```json
    /// {
    ///   "components": ["Anchor"]
    /// }
    /// ```
    ///
    /// ### Why is this bad?
    ///
    /// Anchor elements without content can be confusing for users relying
    /// on screen readers to understand.
    ///
    /// ### Examples
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <a>Anchor Content!</a>
    /// <a><TextWrapper /></a>
    /// <a dangerouslySetInnerHTML={{ __html: 'foo' }} />
    /// <a title='foo' />
    /// <a aria-label='foo' />
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <a />
    /// <a><TextWrapper aria-hidden /></a>
    /// ```
    AnchorHasContent,
    jsx_a11y,
    correctness,
    config = AnchorHasContentConfig,
    conditional_suggestion,
    version = "0.0.18",
    short_description = "Enforce that anchors have content and that the content is accessible to screen readers.",
);

impl Rule for AnchorHasContent {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            let name = get_element_type(ctx, &jsx_el.opening_element);

            if name == "a" || self.components.iter().any(|component| component == name.as_ref()) {
                if is_hidden_from_screen_reader(ctx, &jsx_el.opening_element) {
                    return;
                }

                if object_has_accessible_child(ctx, jsx_el) {
                    return;
                }

                for attr in ["title", "aria-label"] {
                    if has_jsx_prop_ignore_case(&jsx_el.opening_element, attr).is_some() {
                        return;
                    }
                }

                let diagnostic = missing_content(jsx_el.span);
                if jsx_el.children.len() == 1 {
                    let child = &jsx_el.children[0];
                    if let JSXChild::Element(child) = child {
                        ctx.diagnostic_with_suggestion(diagnostic, |_fixer| {
                            remove_hidden_attributes(child).with_message("Remove hidden attribute")
                        });
                        return;
                    }
                }

                ctx.diagnostic(diagnostic);
            }
        }
    }
}

fn remove_hidden_attributes(element: &JSXElement<'_>) -> RuleFix {
    element
        .opening_element
        .attributes
        .iter()
        .filter_map(JSXAttributeItem::as_attribute)
        .filter_map(|attr| {
            attr.name.as_identifier().and_then(|name| {
                if name.name.eq_ignore_ascii_case("aria-hidden")
                    || name.name.eq_ignore_ascii_case("hidden")
                {
                    Some(Fix::delete(attr.span))
                } else {
                    None
                }
            })
        })
        .collect()
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components() -> serde_json::Value {
        serde_json::json!([{
            "components": ["Anchor", "Link"],
        }])
    }

    // https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules/anchor-has-content-test.js
    let pass = vec![
        (r"<div />;", None, None),
        (r"<a>Foo</a>", None, None),
        (r"<a><Bar /></a>", None, None),
        (r"<a>{foo}</a>", None, None),
        (r"<a>{foo.bar}</a>", None, None),
        (r#"<a dangerouslySetInnerHTML={{ __html: "foo" }} />"#, None, None),
        (r"<a children={children} />", None, None),
        (r"<Link />", None, None),
        (r"<Anchor>Anchor Content!</Anchor>", Some(components()), None),
        (r"<Anchor><TextWrapper /></Anchor>", Some(components()), None),
        (r#"<Anchor dangerouslySetInnerHTML={{ __html: "foo" }} />"#, Some(components()), None),
        (r"<Anchor title='foo' />", Some(components()), None),
        (r"<Anchor aria-label='foo' />", Some(components()), None),
        (
            r"<Link>foo</Link>",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        (r"<a title={title} />", None, None),
        (r"<a aria-label={ariaLabel} />", None, None),
        (r"<a title={title} aria-label={ariaLabel} />", None, None),
        (r#"<a><Bar aria-hidden="false" /></a>"#, None, None),
        // anchors can be hidden
        (r"<a aria-hidden>Foo</a>", None, None),
        (r#"<a aria-hidden="true">Foo</a>"#, None, None),
        (r"<a hidden>Foo</a>", None, None),
        (r"<a aria-hidden><span aria-hidden>Foo</span></a>", None, None),
        (r#"<a hidden="true">Foo</a>"#, None, None),
        (r#"<a hidden="">Foo</a>"#, None, None),
        // TODO: should these be failing?
        (r"<a><div hidden /></a>", None, None),
        (r"<a><Bar hidden /></a>", None, None),
        (r#"<a><Bar hidden="" /></a>"#, None, None),
        (r#"<a><Bar hidden="until-hidden" /></a>"#, None, None),
    ];

    let fail = vec![
        (r"<a />", None, None),
        (r"<a><Bar aria-hidden /></a>", None, None),
        (r#"<a><Bar aria-hidden="true" /></a>"#, None, None),
        (r#"<a><input type="hidden" /></a>"#, None, None),
        (r"<a>{undefined}</a>", None, None),
        (r"<a>{null}</a>", None, None),
        (r"<Anchor />", Some(components()), None),
        (r"<Anchor><TextWrapper aria-hidden /></Anchor>", Some(components()), None),
        (
            r"<Link />",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
    ];

    let fix = vec![
        (r"<a><Bar aria-hidden /></a>", "<a><Bar  /></a>"),
        (r"<a><Bar aria-hidden>Can't see me</Bar></a>", r"<a><Bar >Can't see me</Bar></a>"),
        (r"<a><Bar aria-hidden={true}>Can't see me</Bar></a>", r"<a><Bar >Can't see me</Bar></a>"),
        (
            r#"<a><Bar aria-hidden="true">Can't see me</Bar></a>"#,
            r"<a><Bar >Can't see me</Bar></a>",
        ),
    ];

    Tester::new(AnchorHasContent::NAME, AnchorHasContent::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
