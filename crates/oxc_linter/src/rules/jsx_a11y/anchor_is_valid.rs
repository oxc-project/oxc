use std::ops::Deref;

use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeValue, JSXExpression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_str::CompactStr;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::{get_element_type, has_jsx_prop_ignore_case},
};

fn missing_href_attribute<S: AsRef<str>>(span: Span, valid_attrs: &[S]) -> OxcDiagnostic {
    let help = if valid_attrs.len() == 1 {
        format!("Provide the `{}` attribute for the `a` element.", valid_attrs[0].as_ref())
    } else {
        let list =
            valid_attrs.iter().map(|a| format!("`{}`", a.as_ref())).collect::<Vec<_>>().join(", ");
        format!("Provide one of these attributes for the `a` element: {list}")
    };
    OxcDiagnostic::warn("Missing `href` attribute for the `a` element.")
        .with_help(help)
        .with_label(span)
}

fn incorrect_href(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Use of incorrect `href` for the 'a' element.")
        .with_help("Provide a correct `href` for the `a` element.")
        .with_label(span)
}

fn cant_be_anchor(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `a` element has `href` and `onClick`.")
        .with_help("Use a `button` element instead of an `a` element.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct AnchorIsValid(Box<AnchorIsValidConfig>);

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct AnchorIsValidConfig {
    /// Custom components to treat as anchor elements.
    components: Vec<CompactStr>,
    /// Custom prop names to treat as link destinations.
    special_link: Vec<CompactStr>,
    /// Sub-rule aspects to run.
    aspects: Option<Vec<AnchorIsValidAspect>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase")]
enum AnchorIsValidAspect {
    NoHref,
    InvalidHref,
    PreferButton,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HrefValueKind {
    Nullish,
    Invalid,
    Valid,
}

impl Deref for AnchorIsValid {
    type Target = AnchorIsValidConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// The HTML `<a>` element, with a valid href attribute, is formally defined as representing a **hyperlink**.
    /// That is, a link between one HTML document and another, or between one location inside an HTML document and another location inside the same document.
    ///
    /// While before it was possible to attach logic to an anchor element, with the advent of JSX libraries,
    /// it's now  easier to attach logic to any HTML element, anchors included.
    ///
    /// This rule is designed to prevent users to attach logic at the click of anchors, and also makes
    /// sure that the `href` provided to the anchor element is valid. If the anchor has logic attached to it,
    /// the rules suggests to turn it to a `button`, because that's likely what the user wants.
    ///
    /// Anchor `<a></a>` elements should be used for navigation, while `<button></button>` should be
    /// used for user interaction.
    ///
    /// Consider the following:
    ///
    /// ```jsx
    /// <>
    ///     <a href="javascript:void(0)" onClick={foo}>Perform action</a>
    ///     <a href="#" onClick={foo}>Perform action</a>
    ///     <a onClick={foo}>Perform action</a>
    /// </>
    /// ```
    ///
    /// All these anchor implementations indicate that the element is only used to execute JavaScript code. All the above should be replaced with:
    ///
    /// ```jsx
    /// <button onClick={foo}>Perform action</button>
    /// ```
    ///
    /// ### Why is this bad?
    ///
    /// There are **many reasons** why an anchor should not have logic and have a correct `href` attribute:
    /// - it can disrupt the correct flow of the user navigation e.g. a user that wants to open the link
    /// in another tab, but the default "click" behaviour is prevented
    /// - it can source of invalid links, and crawlers can't navigate the website, risking to penalise SEO ranking
    ///
    /// ### Examples
    ///
    /// Examples of **valid** code for this rule:
    ///
    /// ```jsx
    /// <>
    ///     <a href={`https://www.javascript.com`}>navigate here</a>
    ///     <a href={somewhere}>navigate here</a>
    ///     <a {...spread}>navigate here</a>
    /// </>
    /// ```
    ///
    /// Examples of **invalid** code for this rule:
    ///
    /// ```jsx
    /// <>
    ///     <a href={null}>navigate here</a>
    ///     <a href={undefined}>navigate here</a>
    ///     <a href>navigate here</a>
    ///     <a href="javascript:void(0)">navigate here</a>
    ///     <a href="https://example.com" onClick={something}>navigate here</a>
    /// </>
    /// ```
    ///
    /// ### Reference
    ///
    /// - [WCAG 2.1.1](https://www.w3.org/WAI/WCAG21/Understanding/keyboard)
    AnchorIsValid,
    jsx_a11y,
    correctness,
    config = AnchorIsValidConfig,
    version = "0.0.19",
    short_description = "Enforce that anchors have a valid `href` and are not used in place of buttons for attaching click logic.",
);

impl Rule for AnchorIsValid {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            let name = get_element_type(ctx, &jsx_el.opening_element);

            if name != "a" && !self.components.iter().any(|component| component == name.as_ref()) {
                return;
            }
            // Don't eagerly get `span` here, to avoid that work unless rule fails
            let get_span = || jsx_el.opening_element.name.span();

            let href_names = self.href_names(ctx);
            let mut has_href = false;
            let mut has_invalid_href = false;
            let mut has_spread_attr = false;

            for attr in &jsx_el.opening_element.attributes {
                match attr {
                    JSXAttributeItem::SpreadAttribute(_) => has_spread_attr = true,
                    JSXAttributeItem::Attribute(attr) => {
                        if href_names
                            .iter()
                            .any(|href_name| attr.is_identifier_ignore_case(href_name))
                        {
                            match attr
                                .value
                                .as_ref()
                                .map_or(HrefValueKind::Nullish, Self::check_value)
                            {
                                HrefValueKind::Nullish => {}
                                HrefValueKind::Invalid => {
                                    has_href = true;
                                    has_invalid_href = true;
                                }
                                HrefValueKind::Valid => has_href = true,
                            }
                        }
                    }
                }
            }

            let has_on_click =
                has_jsx_prop_ignore_case(&jsx_el.opening_element, "onclick").is_some();

            if !has_href {
                if !has_spread_attr
                    && self.has_aspect(AnchorIsValidAspect::NoHref)
                    && (!has_on_click || !self.has_aspect(AnchorIsValidAspect::PreferButton))
                {
                    ctx.diagnostic(missing_href_attribute(get_span(), &href_names));
                }
                if !has_spread_attr
                    && has_on_click
                    && self.has_aspect(AnchorIsValidAspect::PreferButton)
                {
                    ctx.diagnostic(cant_be_anchor(get_span()));
                }
                return;
            }

            if has_invalid_href {
                if has_on_click && self.has_aspect(AnchorIsValidAspect::PreferButton) {
                    ctx.diagnostic(cant_be_anchor(get_span()));
                } else if self.has_aspect(AnchorIsValidAspect::InvalidHref) {
                    ctx.diagnostic(incorrect_href(get_span()));
                }
            }
        }
    }
}

impl AnchorIsValid {
    fn href_names(&self, ctx: &LintContext<'_>) -> Vec<CompactStr> {
        let mut href_names: Vec<CompactStr> = ctx
            .settings()
            .jsx_a11y
            .attributes
            .get("href")
            .map_or_else(|| vec![CompactStr::new("href")], Clone::clone);
        href_names.extend(self.special_link.iter().cloned());
        href_names
    }

    fn has_aspect(&self, aspect: AnchorIsValidAspect) -> bool {
        self.aspects.as_ref().is_none_or(|aspects| aspects.contains(&aspect))
    }

    fn check_value(value: &JSXAttributeValue) -> HrefValueKind {
        match value {
            JSXAttributeValue::Element(_) => HrefValueKind::Valid,
            JSXAttributeValue::StringLiteral(str_lit) => {
                Self::href_value_kind_from_string(&str_lit.value)
            }
            JSXAttributeValue::ExpressionContainer(exp) => match &exp.expression {
                JSXExpression::Identifier(ident) if ident.name == "undefined" => {
                    HrefValueKind::Nullish
                }
                JSXExpression::NullLiteral(_) => HrefValueKind::Nullish,
                JSXExpression::StringLiteral(str_lit) => {
                    Self::href_value_kind_from_string(&str_lit.value)
                }
                JSXExpression::TemplateLiteral(temp_lit) => {
                    if !temp_lit.expressions.is_empty() {
                        return HrefValueKind::Valid;
                    }

                    let Some(quasi) = temp_lit.single_quasi() else {
                        return HrefValueKind::Valid;
                    };
                    Self::href_value_kind_from_string(&quasi)
                }
                _ => HrefValueKind::Valid,
            },
            JSXAttributeValue::Fragment(_) => HrefValueKind::Nullish,
        }
    }

    fn href_value_kind_from_string(href: &str) -> HrefValueKind {
        if Self::is_invalid_href(href) { HrefValueKind::Invalid } else { HrefValueKind::Valid }
    }

    fn is_invalid_href(href: &str) -> bool {
        let href_without_leading_non_word =
            href.trim_start_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_');
        href.is_empty() || href == "#" || href_without_leading_non_word.starts_with("javascript:")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components() -> serde_json::Value {
        serde_json::json!([{ "components": ["Anchor", "Link"] }])
    }

    fn special_link() -> serde_json::Value {
        serde_json::json!([{ "specialLink": ["hrefLeft", "hrefRight"] }])
    }

    fn no_href_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["noHref"] }])
    }

    fn invalid_href_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["invalidHref"] }])
    }

    fn prefer_button_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["preferButton"] }])
    }

    fn no_href_invalid_href_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["noHref", "invalidHref"] }])
    }

    fn no_href_prefer_button_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["noHref", "preferButton"] }])
    }

    fn prefer_button_invalid_href_aspect() -> serde_json::Value {
        serde_json::json!([{ "aspects": ["preferButton", "invalidHref"] }])
    }

    fn components_and_special_link() -> serde_json::Value {
        serde_json::json!([{ "components": ["Anchor"], "specialLink": ["hrefLeft"] }])
    }

    fn components_and_special_link_and_invalid_href_aspect() -> serde_json::Value {
        serde_json::json!([{
            "components": ["Anchor"],
            "specialLink": ["hrefLeft"],
            "aspects": ["invalidHref"]
        }])
    }

    fn components_and_special_link_and_no_href_aspect() -> serde_json::Value {
        serde_json::json!([{
            "components": ["Anchor"],
            "specialLink": ["hrefLeft"],
            "aspects": ["noHref"]
        }])
    }

    fn components_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": {
                "jsx-a11y": {
                    "components": {
                        "Anchor": "a",
                        "Link": "a"
                    }
                }
            }
        })
    }

    fn attributes_settings() -> serde_json::Value {
        serde_json::json!({
            "settings": {
                "jsx-a11y": {
                    "components": { "Link": "a" },
                    "attributes": { "href": ["href", "to"] }
                }
            }
        })
    }

    // https://raw.githubusercontent.com/jsx-eslint/eslint-plugin-jsx-a11y/main/__tests__/src/rules/anchor-is-valid-test.js
    let pass = vec![
        (r"<Anchor />", None, None),
        (r"<a {...props} />", None, None),
        (r"<a href='foo' />", None, None),
        (r"<a href={foo} />", None, None),
        (r"<a href='/foo' />", None, None),
        (r"<a href='https://foo.bar.com' />", None, None),
        (r"<div href='foo' />", None, None),
        (r"<a href='javascript' />", None, None),
        (r"<a href='javascriptFoo' />", None, None),
        (r"<a href={`#foo`}/>", None, None),
        (r"<a href={'foo'}/>", None, None),
        (r"<a href={'javascript'}/>", None, None),
        (r"<a href={`#javascript`}/>", None, None),
        (r"<a href='#foo' />", None, None),
        (r"<a href='#javascript' />", None, None),
        (r"<a href='#javascriptFoo' />", None, None),
        (r"<UX.Layout>test</UX.Layout>", None, None),
        (r"<a href={this} />", None, None),
        (r"<Anchor {...props} />", Some(components()), None),
        (r"<Anchor href='foo' />", Some(components()), None),
        (r"<Anchor href={foo} />", Some(components()), None),
        (r"<Anchor href='/foo' />", Some(components()), None),
        (r"<Anchor href='https://foo.bar.com' />", Some(components()), None),
        (r"<div href='foo' />", Some(components()), None),
        (r"<Anchor href={`#foo`}/>", Some(components()), None),
        (r"<Anchor href={'foo'}/>", Some(components()), None),
        (r"<Anchor href='#foo' />", Some(components()), None),
        (r"<Link {...props} />", Some(components()), None),
        (r"<Link href='foo' />", Some(components()), None),
        (r"<Link href={foo} />", Some(components()), None),
        (r"<Link href='/foo' />", Some(components()), None),
        (r"<Link href='https://foo.bar.com' />", Some(components()), None),
        (r"<div href='foo' />", Some(components()), None),
        (r"<Link href={`#foo`}/>", Some(components()), None),
        (r"<Link href={'foo'}/>", Some(components()), None),
        (r"<Link href='#foo' />", Some(components()), None),
        (r"<Link href='#foo' />", None, Some(components_settings())),
        (r"<Link to='https://example.com' />", None, Some(attributes_settings())),
        (r"<Link to={dest} />", None, Some(attributes_settings())),
        (r"<a {...props} />", Some(special_link()), None),
        (r"<a hrefLeft='foo' />", Some(special_link()), None),
        (r"<a hrefLeft={foo} />", Some(special_link()), None),
        (r"<a hrefLeft='/foo' />", Some(special_link()), None),
        (r"<a hrefLeft='https://foo.bar.com' />", Some(special_link()), None),
        (r"<div hrefLeft='foo' />", Some(special_link()), None),
        (r"<a hrefLeft={`#foo`}/>", Some(special_link()), None),
        (r"<a hrefLeft={'foo'}/>", Some(special_link()), None),
        (r"<a hrefLeft='#foo' />", Some(special_link()), None),
        (r"<UX.Layout>test</UX.Layout>", Some(special_link()), None),
        (r"<a hrefRight={this} />", Some(special_link()), None),
        (r"<a {...props} />", Some(special_link()), None),
        (r"<a hrefRight='foo' />", Some(special_link()), None),
        (r"<a hrefRight={foo} />", Some(special_link()), None),
        (r"<a hrefRight='/foo' />", Some(special_link()), None),
        (r"<a hrefRight='https://foo.bar.com' />", Some(special_link()), None),
        (r"<div hrefRight='foo' />", Some(special_link()), None),
        (r"<a hrefRight={`#foo`}/>", Some(special_link()), None),
        (r"<a hrefRight={'foo'}/>", Some(special_link()), None),
        (r"<a hrefRight='#foo' />", Some(special_link()), None),
        (r"<UX.Layout>test</UX.Layout>", Some(special_link()), None),
        (r"<a hrefRight={this} />", Some(special_link()), None),
        (r"<Anchor {...props} />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='foo' />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={foo} />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='/foo' />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='https://foo.bar.com' />", Some(components_and_special_link()), None),
        (r"<div hrefLeft='foo' />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={`#foo`}/>", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={'foo'}/>", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='#foo' />", Some(components_and_special_link()), None),
        (r"<UX.Layout>test</UX.Layout>", Some(components_and_special_link()), None),
        (r"<a {...props} onClick={() => void 0} />", None, None),
        (r"<a href='foo' onClick={() => void 0} />", None, None),
        (r"<a href={foo} onClick={() => void 0} />", None, None),
        (r"<a href='/foo' onClick={() => void 0} />", None, None),
        (r"<a href='https://foo.bar.com' onClick={() => void 0} />", None, None),
        (r"<div href='foo' onClick={() => void 0} />", None, None),
        (r"<a href={`#foo`} onClick={() => void 0} />", None, None),
        (r"<a href={'foo'} onClick={() => void 0} />", None, None),
        (r"<a href='#foo' onClick={() => void 0} />", None, None),
        (r"<a href={this} onClick={() => void 0} />", None, None),
        (r"<Anchor {...props} onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href={foo} onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='/foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='https://foo.bar.com' onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href={`#foo`} onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href={'foo'} onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='#foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Link {...props} onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Link href={foo} onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='/foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='https://foo.bar.com' onClick={() => void 0} />", Some(components()), None),
        (r"<div href='foo' onClick={() => void 0} />", Some(components()), None),
        (r"<Link href={`#foo`} onClick={() => void 0} />", Some(components()), None),
        (r"<Link href={'foo'} onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='#foo' onClick={() => void 0} />", Some(components()), None),
        (r"<a {...props} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft='foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft={foo} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft='/foo' onClick={() => void 0} />", Some(special_link()), None),
        (
            r"<a hrefLeft href='https://foo.bar.com' onClick={() => void 0} />",
            Some(special_link()),
            None,
        ),
        (r"<div hrefLeft='foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft={`#foo`} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft={'foo'} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft='#foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight={this} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a {...props} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight='foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight={foo} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight='/foo' onClick={() => void 0} />", Some(special_link()), None),
        (
            r"<a hrefRight href='https://foo.bar.com' onClick={() => void 0} />",
            Some(special_link()),
            None,
        ),
        (r"<div hrefRight='foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight={`#foo`} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight={'foo'} onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight='#foo' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefRight={this} onClick={() => void 0} />", Some(special_link()), None),
        (
            r"<Anchor {...props} onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft='foo' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft={foo} onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft='/foo' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft href='https://foo.bar.com' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft={`#foo`} onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft={'foo'} onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft='#foo' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (r"<a />", Some(invalid_href_aspect()), None),
        (r"<a href={undefined} />", Some(invalid_href_aspect()), None),
        (r"<a href={null} />", Some(invalid_href_aspect()), None),
        (r"<a />", Some(prefer_button_aspect()), None),
        (r"<a href={undefined} />", Some(prefer_button_aspect()), None),
        (r"<a href={null} />", Some(prefer_button_aspect()), None),
        (r"<a />", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href={undefined} />", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href={null} />", Some(prefer_button_invalid_href_aspect()), None),
        (r#"<a href="" />;"#, Some(prefer_button_aspect()), None),
        (r"<a href='#' />", Some(prefer_button_aspect()), None),
        (r"<a href={'#'} />", Some(prefer_button_aspect()), None),
        (r"<a href='javascript:void(0)' />", Some(prefer_button_aspect()), None),
        (r"<a href={'javascript:void(0)'} />", Some(prefer_button_aspect()), None),
        (r#"<a href="" />;"#, Some(no_href_aspect()), None),
        (r"<a href='#' />", Some(no_href_aspect()), None),
        (r"<a href={'#'} />", Some(no_href_aspect()), None),
        (r"<a href='javascript:void(0)' />", Some(no_href_aspect()), None),
        (r"<a href={'javascript:void(0)'} />", Some(no_href_aspect()), None),
        (r#"<a href="" />;"#, Some(no_href_prefer_button_aspect()), None),
        (r"<a href='#' />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href={'#'} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href='javascript:void(0)' />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href={'javascript:void(0)'} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(invalid_href_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(no_href_aspect()), None),
        (r"<a href='javascript:void(0)' onClick={() => void 0} />", Some(no_href_aspect()), None),
        (r"<a href={'javascript:void(0)'} onClick={() => void 0} />", Some(no_href_aspect()), None),
        (
            r"<Anchor hrefLeft={undefined} />",
            Some(components_and_special_link_and_invalid_href_aspect()),
            None,
        ),
        (
            r"<Anchor hrefLeft={null} />",
            Some(components_and_special_link_and_invalid_href_aspect()),
            None,
        ),
    ];

    let fail = vec![
        (r"<a />", None, None),
        (r"<a href />", None, None),
        (r"<a href={undefined} />", None, None),
        (r"<a href={null} />", None, None),
        (r"<a href='' />;", None, None),
        (r"<a href='#' />", None, None),
        (r"<a href={'#'} />", None, None),
        (r"<a href={`#`} />", None, None),
        (r"<a href='javascript:void(0)' />", None, None),
        (r"<a href={'javascript:void(0)'} />", None, None),
        (r"<a onClick={() => void 0} />", None, None),
        (r"<a href='#' onClick={() => void 0} />", None, None),
        (r"<a href='javascript:void(0)' onClick={() => void 0} />", None, None),
        (r"<a href={'javascript:void(0)'} onClick={() => void 0} />", None, None),
        (r"<Link />", Some(components()), None),
        (r"<Link href={undefined} />", Some(components()), None),
        (r"<Link href={null} />", Some(components()), None),
        (r#"<Link href="" />"#, Some(components()), None),
        (r"<Link href='#' />", Some(components()), None),
        (r"<Link href={'#'} />", Some(components()), None),
        (r"<Link href='javascript:void(0)' />", Some(components()), None),
        (r"<Link href={'javascript:void(0)'} />", Some(components()), None),
        (r#"<Anchor href="" />"#, Some(components()), None),
        (r"<Anchor href='#' />", Some(components()), None),
        (r"<Anchor href={'#'} />", Some(components()), None),
        (r"<Anchor href='javascript:void(0)' />", Some(components()), None),
        (r"<Anchor href={'javascript:void(0)'} />", Some(components()), None),
        (r"<Link onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='#' onClick={() => void 0} />", Some(components()), None),
        (r"<Link href='javascript:void(0)' onClick={() => void 0} />", Some(components()), None),
        (r"<Link href={'javascript:void(0)'} onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='#' onClick={() => void 0} />", Some(components()), None),
        (r"<Anchor href='javascript:void(0)' onClick={() => void 0} />", Some(components()), None),
        (
            r"<Anchor href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(components()),
            None,
        ),
        (r"<Link href='#' onClick={() => void 0} />", None, Some(components_settings())),
        (r"<Link />", None, Some(attributes_settings())),
        (r"<Link to='#' />", None, Some(attributes_settings())),
        (r"<a hrefLeft={undefined} />", Some(special_link()), None),
        (r"<a hrefLeft />", Some(special_link()), None),
        (r"<a hrefLeft={null} />", Some(special_link()), None),
        (r#"<a hrefLeft="" />;"#, Some(special_link()), None),
        (r"<a hrefLeft='#' />", Some(special_link()), None),
        (r"<a hrefLeft={'#'} />", Some(special_link()), None),
        (r"<a hrefLeft='javascript:void(0)' />", Some(special_link()), None),
        (r"<a hrefLeft={'javascript:void(0)'} />", Some(special_link()), None),
        (r"<a hrefLeft='#' onClick={() => void 0} />", Some(special_link()), None),
        (r"<a hrefLeft='javascript:void(0)' onClick={() => void 0} />", Some(special_link()), None),
        (
            r"<a hrefLeft={'javascript:void(0)'} onClick={() => void 0} />",
            Some(special_link()),
            None,
        ),
        (r"<Anchor Anchor={undefined} />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={null} />", Some(components_and_special_link()), None),
        (r#"<Anchor hrefLeft="" />;"#, Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='#' />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={'#'} />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft='javascript:void(0)' />", Some(components_and_special_link()), None),
        (r"<Anchor hrefLeft={'javascript:void(0)'} />", Some(components_and_special_link()), None),
        (
            r"<Anchor hrefLeft='#' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft='javascript:void(0)' onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (
            r"<Anchor hrefLeft={'javascript:void(0)'} onClick={() => void 0} />",
            Some(components_and_special_link()),
            None,
        ),
        (r"<a />", Some(no_href_aspect()), None),
        (r"<a />", Some(no_href_prefer_button_aspect()), None),
        (r"<a />", Some(no_href_invalid_href_aspect()), None),
        (r"<a href={undefined} />", Some(no_href_aspect()), None),
        (r"<a href={undefined} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href={undefined} />", Some(no_href_invalid_href_aspect()), None),
        (r"<a href={null} />", Some(no_href_aspect()), None),
        (r"<a href={null} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href={null} />", Some(no_href_invalid_href_aspect()), None),
        (r#"<a href="" />;"#, Some(invalid_href_aspect()), None),
        (r#"<a href="" />;"#, Some(no_href_invalid_href_aspect()), None),
        (r#"<a href="" />;"#, Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href='#' />;", Some(invalid_href_aspect()), None),
        (r"<a href='#' />;", Some(no_href_invalid_href_aspect()), None),
        (r"<a href='#' />;", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href={'#'} />;", Some(invalid_href_aspect()), None),
        (r"<a href={'#'} />;", Some(no_href_invalid_href_aspect()), None),
        (r"<a href={'#'} />;", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href='javascript:void(0)' />;", Some(invalid_href_aspect()), None),
        (r"<a href='javascript:void(0)' />;", Some(no_href_invalid_href_aspect()), None),
        (r"<a href='javascript:void(0)' />;", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href={'javascript:void(0)'} />;", Some(invalid_href_aspect()), None),
        (r"<a href={'javascript:void(0)'} />;", Some(no_href_invalid_href_aspect()), None),
        (r"<a href={'javascript:void(0)'} />;", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(prefer_button_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(no_href_aspect()), None),
        (r"<a onClick={() => void 0} />", Some(no_href_invalid_href_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(prefer_button_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(no_href_prefer_button_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(prefer_button_invalid_href_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(invalid_href_aspect()), None),
        (r"<a href='#' onClick={() => void 0} />", Some(no_href_invalid_href_aspect()), None),
        (
            r"<a href='javascript:void(0)' onClick={() => void 0} />",
            Some(prefer_button_aspect()),
            None,
        ),
        (
            r"<a href='javascript:void(0)' onClick={() => void 0} />",
            Some(no_href_prefer_button_aspect()),
            None,
        ),
        (
            r"<a href='javascript:void(0)' onClick={() => void 0} />",
            Some(prefer_button_invalid_href_aspect()),
            None,
        ),
        (
            r"<a href='javascript:void(0)' onClick={() => void 0} />",
            Some(invalid_href_aspect()),
            None,
        ),
        (
            r"<a href='javascript:void(0)' onClick={() => void 0} />",
            Some(no_href_invalid_href_aspect()),
            None,
        ),
        (
            r"<a href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(prefer_button_aspect()),
            None,
        ),
        (
            r"<a href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(no_href_prefer_button_aspect()),
            None,
        ),
        (
            r"<a href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(prefer_button_invalid_href_aspect()),
            None,
        ),
        (
            r"<a href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(invalid_href_aspect()),
            None,
        ),
        (
            r"<a href={'javascript:void(0)'} onClick={() => void 0} />",
            Some(no_href_invalid_href_aspect()),
            None,
        ),
        (
            r"<Anchor hrefLeft={undefined} />",
            Some(components_and_special_link_and_no_href_aspect()),
            None,
        ),
        (
            r"<Anchor hrefLeft={null} />",
            Some(components_and_special_link_and_no_href_aspect()),
            None,
        ),
    ];

    Tester::new(AnchorIsValid::NAME, AnchorIsValid::PLUGIN, pass, fail).test_and_snapshot();
}
