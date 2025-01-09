use std::borrow::Cow;

use oxc_ast::{
    ast::{JSXChild, JSXElement},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{
        get_element_type, get_string_literal_prop_value, has_jsx_prop_ignore_case,
        is_hidden_from_screen_reader,
    },
    AstNode,
};

fn anchor_has_ambiguous_text(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected ambagious anchor link text.").with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct AnchorAmbiguousText(Box<AnchorAmbiguousTextConfig>);

#[derive(Debug, Clone)]
pub struct AnchorAmbiguousTextConfig {
    words: Vec<CompactStr>,
}

impl Default for AnchorAmbiguousTextConfig {
    fn default() -> Self {
        Self {
            words: vec![
                CompactStr::new("click here"),
                CompactStr::new("here"),
                CompactStr::new("link"),
                CompactStr::new("a link"),
                CompactStr::new("learn more"),
            ],
        }
    }
}

impl std::ops::Deref for AnchorAmbiguousText {
    type Target = AnchorAmbiguousTextConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Inspects anchor link text for the use of ambiguous words.
    ///
    /// This rule checks the text from the anchor element `aria-label` if available.
    /// In absence of an anchor `aria-label` it combines the following text of it's children:
    /// * `aria-label` if available
    /// * if the child is an image, the `alt` text
    /// * the text content of the HTML element
    ///
    /// ### Why is this bad?
    ///
    /// Screen readers users rely on link text for context, ambiguous words such as "click here" do
    /// not provide enough context.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <a>link</a>
    /// <a>click here</a>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <a>read this tutorial</a>
    /// <a aria-label="oxc linter documentation">click here</a>
    /// ```
    AnchorAmbiguousText,
    jsx_a11y,
    restriction,
);

impl Rule for AnchorAmbiguousText {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut config = AnchorAmbiguousTextConfig::default();

        if let Some(words_array) =
            value.get(0).and_then(|v| v.get("words")).and_then(serde_json::Value::as_array)
        {
            config.words = words_array
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(CompactStr::from)
                .collect();
        }

        Self(Box::new(config))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(jsx_el) = node.kind() else {
            return;
        };

        let name = get_element_type(ctx, &jsx_el.opening_element);

        if name != "a" {
            return;
        }

        let Some(text) = get_accessible_text(jsx_el, ctx) else {
            return;
        };

        if text.trim() == "" {
            return;
        }

        if self.words.contains(&normalize_str(&text)) {
            ctx.diagnostic(anchor_has_ambiguous_text(jsx_el.span));
        }
    }
}

// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/65c9338c62c558d3c1c2dbf5ecc55cf04dbfe80c/src/util/getAccessibleChildText.js#L15
fn normalize_str(text: &str) -> CompactStr {
    // `to_lowercase` is disallowed. however we need to remove certain chars later which requires converting to a String
    // the overhead of going &str -> cow string -> string is greater than just using to_lowercase
    #[allow(clippy::disallowed_methods)]
    let mut normalized_str = text.to_lowercase();
    normalized_str.retain(|c| {
        c != ','
            && c != '.'
            && c != '?'
            && c != '¿'
            && c != '!'
            && c != '‽'
            && c != '¡'
            && c != ';'
            && c != ':'
    });

    if normalized_str.contains(char::is_whitespace) {
        let parts: Vec<String> =
            normalized_str.split_whitespace().map(std::string::ToString::to_string).collect();
        return CompactStr::from(parts.join(" "));
    }

    CompactStr::from(normalized_str)
}

// https://github.com/jsx-eslint/eslint-plugin-jsx-a11y/blob/65c9338c62c558d3c1c2dbf5ecc55cf04dbfe80c/src/util/getAccessibleChildText.js#L31
fn get_accessible_text<'a, 'b>(
    jsx_el: &'b JSXElement<'a>,
    ctx: &LintContext<'a>,
) -> Option<Cow<'b, str>> {
    if let Some(aria_label) = has_jsx_prop_ignore_case(&jsx_el.opening_element, "aria-label") {
        if let Some(label_text) = get_string_literal_prop_value(aria_label) {
            return Some(Cow::Borrowed(label_text));
        };
    }

    let name = get_element_type(ctx, &jsx_el.opening_element);
    if name == "img" {
        if let Some(alt_text) = has_jsx_prop_ignore_case(&jsx_el.opening_element, "alt") {
            if let Some(text) = get_string_literal_prop_value(alt_text) {
                return Some(Cow::Borrowed(text));
            };
        };
    }

    if is_hidden_from_screen_reader(ctx, &jsx_el.opening_element) {
        return None;
    }

    let text: Vec<Cow<'b, str>> = jsx_el
        .children
        .iter()
        .filter_map(|child| match child {
            JSXChild::Element(child_el) => get_accessible_text(child_el, ctx),
            JSXChild::Text(text_el) => Some(Cow::Borrowed(text_el.value.as_str())),
            _ => None,
        })
        .collect();

    Some(Cow::Owned(text.join(" ")))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (
            r#"<a><Image alt="documentation" /></a>;"#,
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Image": "img" } } } }),
            ),
        ),
        ("<a>documentation</a>;", None, None),
        ("<a>${here}</a>;", None, None),
        (r#"<a aria-label="tutorial on using eslint-plugin-jsx-a11y">click here</a>;"#, None, None),
        (
            r#"<a><span aria-label="tutorial on using eslint-plugin-jsx-a11y">click here</span></a>;"#,
            None,
            None,
        ),
        (r#"<a><img alt="documentation" /></a>;"#, None, None),
        (
            "<a>click here</a>",
            Some(serde_json::json!([{        "words": ["disabling the defaults"],      }])),
            None,
        ),
        (
            "<Link>documentation</Link>;",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        (
            r#"<a><Image alt="documentation" /></a>;"#,
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Image": "img" } } } }),
            ),
        ),
        (
            "<Link>${here}</Link>;",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        (
            r#"<Link aria-label="tutorial on using eslint-plugin-jsx-a11y">click here</Link>;"#,
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        (
            "<Link>click here</Link>",
            Some(
                serde_json::json!([{        "words": ["disabling the defaults with components"],      }]),
            ),
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
    ];

    let fail = vec![
        ("<a>here</a>;", None, None),
        ("<a>HERE</a>;", None, None),
        ("<a>click here</a>;", None, None),
        ("<a>learn more</a>;", None, None),
        ("<a>learn      more</a>;", None, None),
        ("<a>learn more.</a>;", None, None),
        ("<a>learn more?</a>;", None, None),
        ("<a>learn more,</a>;", None, None),
        ("<a>learn more!</a>;", None, None),
        ("<a>learn more;</a>;", None, None),
        ("<a>learn more:</a>;", None, None),
        ("<a>link</a>;", None, None),
        ("<a>a link</a>;", None, None),
        (r#"<a aria-label="click here">something</a>;"#, None, None),
        ("<a> a link </a>;", None, None),
        ("<a>a<i></i> link</a>;", None, None),
        ("<a><i></i>a link</a>;", None, None),
        ("<a><span>click</span> here</a>;", None, None),
        ("<a><span> click </span> here</a>;", None, None),
        ("<a><span aria-hidden>more text</span>learn more</a>;", None, None),
        (r#"<a><span aria-hidden="true">more text</span>learn more</a>;"#, None, None),
        (r#"<a><img alt="click here"/></a>;"#, None, None),
        (r#"<a alt="tutorial on using eslint-plugin-jsx-a11y">click here</a>;"#, None, None),
        (
            r#"<a><span alt="tutorial on using eslint-plugin-jsx-a11y">click here</span></a>;"#,
            None,
            None,
        ),
        ("<a><CustomElement>click</CustomElement> here</a>;", None, None),
        (
            "<Link>here</Link>",
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Link": "a" } } } }),
            ),
        ),
        (
            r#"<a><Image alt="click here" /></a>"#,
            None,
            Some(
                serde_json::json!({ "settings": { "jsx-a11y": { "components": { "Image": "img" } } } }),
            ),
        ),
        (
            "<a>a disallowed word</a>",
            Some(serde_json::json!([{        "words": ["a disallowed word"],      }])),
            None,
        ),
    ];

    Tester::new(AnchorAmbiguousText::NAME, AnchorAmbiguousText::PLUGIN, pass, fail)
        .test_and_snapshot();
}
