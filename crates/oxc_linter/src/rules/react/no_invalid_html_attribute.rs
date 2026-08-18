use cow_utils::CowUtils;
use phf::{Map, Set, phf_map, phf_set};
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_ast::{
    AstKind,
    ast::{
        Argument, ArrayExpressionElement, Expression, JSXAttributeName, JSXAttributeValue,
        JSXElementName, JSXOpeningElement, ObjectExpression, ObjectPropertyKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    globals::HTML_TAG,
    rule::{DefaultRuleConfig, Rule},
    utils::is_create_element_call,
};

fn invalid_value_diagnostic(
    value: &str,
    attribute: &str,
    element: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "`{value}` is not a valid value for the `{attribute}` attribute on `<{element}>`."
    ))
    .with_help(format!("Remove `{value}`, or replace it with a value allowed on `<{element}>`."))
    .with_label(span)
}

fn invalid_element_diagnostic(attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The `{attribute}` attribute has no meaning on this element."))
        .with_help(format!("Remove the `{attribute}` attribute."))
        .with_label(span)
}

fn never_valid_diagnostic(value: &str, attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{value}` is never a valid `{attribute}` attribute value."))
        .with_help(format!("Remove `{value}`."))
        .with_label(span)
}

fn empty_value_diagnostic(attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("An empty `{attribute}` attribute is meaningless."))
        .with_help(format!("Give the `{attribute}` attribute a value, or remove it."))
        .with_label(span)
}

fn non_string_value_diagnostic(attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The `{attribute}` attribute only supports strings."))
        .with_help(format!("Pass a space delimited string to `{attribute}`, or remove it."))
        .with_label(span)
}

fn method_value_diagnostic(attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("The `{attribute}` attribute cannot be a method."))
        .with_help(format!("Pass a space delimited string to `{attribute}`, or remove it."))
        .with_label(span)
}

fn space_delimited_diagnostic(attribute: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{attribute}` attribute values should be space delimited."))
        .with_help("Replace this whitespace with a single space.")
        .with_label(span)
}

fn unpaired_value_diagnostic(value: &str, expected: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{value}` must be directly followed by {expected}."))
        .with_help(format!("Add {expected} after `{value}`, or remove `{value}`."))
        .with_label(span)
}

/// Elements the `rel` attribute itself has any meaning on.
static REL_ELEMENTS: Set<&'static str> = phf_set! { "a", "area", "link", "form" };

/// Every valid `rel` value, mapped to the elements it is meaningful on.
static REL_VALUES: Map<&'static str, Set<&'static str>> = phf_map! {
    "alternate" => phf_set! { "link", "area", "a" },
    "apple-touch-icon" => phf_set! { "link" },
    "apple-touch-startup-image" => phf_set! { "link" },
    "author" => phf_set! { "link", "area", "a" },
    "bookmark" => phf_set! { "area", "a" },
    "canonical" => phf_set! { "link" },
    "dns-prefetch" => phf_set! { "link" },
    "expect" => phf_set! { "link" },
    "external" => phf_set! { "area", "a", "form" },
    "help" => phf_set! { "link", "area", "a", "form" },
    "icon" => phf_set! { "link" },
    "license" => phf_set! { "link", "area", "a", "form" },
    "manifest" => phf_set! { "link" },
    "mask-icon" => phf_set! { "link" },
    // Registered through the microformats registry the HTML spec defers to, and
    // the basis of IndieAuth and Mastodon profile verification.
    "me" => phf_set! { "link", "area", "a" },
    "modulepreload" => phf_set! { "link" },
    "next" => phf_set! { "link", "area", "a", "form" },
    "nofollow" => phf_set! { "area", "a", "form" },
    "noopener" => phf_set! { "area", "a", "form" },
    "noreferrer" => phf_set! { "area", "a", "form" },
    "opener" => phf_set! { "area", "a", "form" },
    "pingback" => phf_set! { "link" },
    "preconnect" => phf_set! { "link" },
    "prefetch" => phf_set! { "link" },
    "preload" => phf_set! { "link" },
    "prerender" => phf_set! { "link" },
    "prev" => phf_set! { "link", "area", "a", "form" },
    "privacy-policy" => phf_set! { "link", "area", "a", "form" },
    "search" => phf_set! { "link", "area", "a", "form" },
    "shortcut" => phf_set! { "link" },
    // Only reachable through the `React.createElement` form, which looks the
    // whole value up as a single key instead of splitting it.
    "shortcut icon" => phf_set! { "link" },
    "stylesheet" => phf_set! { "link" },
    "tag" => phf_set! { "area", "a" },
    "terms-of-service" => phf_set! { "link", "area", "a", "form" },
};

/// Values that are only meaningful when directly followed by one of a fixed set
/// of other values.
static REL_PAIRS: Map<&'static str, Set<&'static str>> = phf_map! {
    "shortcut" => phf_set! { "icon" },
};

/// Attributes this rule knows how to validate. Upstream restricts the option to
/// `rel`, which is also the default when no configuration is provided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum CheckedAttribute {
    Rel,
}

impl CheckedAttribute {
    fn name(self) -> &'static str {
        match self {
            Self::Rel => "rel",
        }
    }

    /// Elements on which the attribute has any meaning at all.
    fn elements(self) -> &'static Set<&'static str> {
        match self {
            Self::Rel => &REL_ELEMENTS,
        }
    }

    /// Valid values, mapped to the elements each one is meaningful on.
    fn values(self) -> &'static Map<&'static str, Set<&'static str>> {
        match self {
            Self::Rel => &REL_VALUES,
        }
    }

    /// Values that must be directly followed by one of a set of other values.
    fn pairs(self) -> &'static Map<&'static str, Set<&'static str>> {
        match self {
            Self::Rel => &REL_PAIRS,
        }
    }
}

// Configured as the set of attributes to check. With only one attribute known
// that set is just a flag, so it is stored as one; the serialized form stays a
// list so the option keeps accepting `["rel"]`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(from = "Vec<CheckedAttribute>", into = "Vec<CheckedAttribute>")]
// A set rather than a `Vec` in the schema, so the list keeps `uniqueItems`.
pub struct NoInvalidHtmlAttribute(#[schemars(with = "FxHashSet<CheckedAttribute>")] bool);

impl Default for NoInvalidHtmlAttribute {
    fn default() -> Self {
        // Upstream defaults to `["rel"]`, not an empty set.
        Self(true)
    }
}

impl From<Vec<CheckedAttribute>> for NoInvalidHtmlAttribute {
    fn from(attributes: Vec<CheckedAttribute>) -> Self {
        Self(attributes.contains(&CheckedAttribute::Rel))
    }
}

impl From<NoInvalidHtmlAttribute> for Vec<CheckedAttribute> {
    fn from(config: NoInvalidHtmlAttribute) -> Self {
        config.0.then_some(CheckedAttribute::Rel).into_iter().collect()
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow values that have no meaning for the `rel` attribute on the
    /// elements that accept it (`<a>`, `<area>`, `<link>` and `<form>`).
    ///
    /// ### Why is this bad?
    ///
    /// Each of these elements accepts only a fixed set of `rel` values, and the
    /// sets differ between them. A value that is misspelled, or valid on a
    /// different element, is silently ignored by the browser. The markup then
    /// reads as if it expresses a relationship that is never actually applied.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <a rel="alternate stylesheet" />
    /// <link rel="foobar" href="./s.css" />
    /// <a rel="" />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <a rel="alternate" href="/feed" />
    /// <link rel="stylesheet" href="./s.css" />
    /// <form rel="noreferrer" />
    /// ```
    NoInvalidHtmlAttribute,
    react,
    correctness,
    // Upstream offers suggestions; switch to `suggestion` once one is implemented.
    pending,
    config = NoInvalidHtmlAttribute,
    version = "next",
    short_description = "Disallow invalid values for the `rel` attribute on HTML elements.",
);

impl Rule for NoInvalidHtmlAttribute {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        match node.kind() {
            AstKind::JSXOpeningElement(jsx_el) => {
                let JSXElementName::Identifier(name) = &jsx_el.name else {
                    return;
                };
                // Only real HTML elements have a fixed set of meaningful values;
                // a custom component may do anything with the prop.
                if !HTML_TAG.contains(name.name.as_str()) {
                    return;
                }
                self.check_jsx_element(jsx_el, name.name.as_str(), ctx);
            }
            AstKind::CallExpression(call_expr) if is_create_element_call(call_expr) => {
                // Only a literal element name can be resolved statically.
                let Some(Argument::StringLiteral(element)) = call_expr.arguments.first() else {
                    return;
                };
                let element = element.value.as_str();
                if !HTML_TAG.contains(element) {
                    return;
                }
                // Props passed as a variable or a spread cannot be inspected.
                let Some(Argument::ObjectExpression(props)) = call_expr.arguments.get(1) else {
                    return;
                };
                self.check_create_element_props(props, element, ctx);
            }
            _ => {}
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        self.0 && ctx.source_type().is_jsx()
    }
}

impl NoInvalidHtmlAttribute {
    /// Resolves `name` to an attribute this rule is configured to check.
    fn checked_attribute(self, name: &str) -> Option<CheckedAttribute> {
        (self.0 && name == "rel").then_some(CheckedAttribute::Rel)
    }

    fn check_jsx_element(
        self,
        jsx_el: &JSXOpeningElement<'_>,
        element: &str,
        ctx: &LintContext<'_>,
    ) {
        for attr in &jsx_el.attributes {
            // Spread attributes carry no name to match against.
            let Some(attr) = attr.as_attribute() else {
                continue;
            };
            let JSXAttributeName::Identifier(attr_name) = &attr.name else {
                continue;
            };
            let Some(attribute) = self.checked_attribute(attr_name.name.as_str()) else {
                continue;
            };
            let name = attribute.name();

            if !attribute.elements().contains(element) {
                ctx.diagnostic(invalid_element_diagnostic(name, attr.name.span()));
                continue;
            }

            // `<a rel />`
            let Some(value) = &attr.value else {
                ctx.diagnostic(empty_value_diagnostic(name, attr.name.span()));
                continue;
            };

            match value {
                // React decodes character references in this position, so the
                // source text is not necessarily the value the browser sees.
                JSXAttributeValue::StringLiteral(literal) => {
                    check_literal_value(
                        attribute,
                        literal.value.as_str(),
                        literal.span,
                        element,
                        true,
                        ctx,
                    );
                }
                JSXAttributeValue::ExpressionContainer(container) => {
                    match container.expression.as_expression().map(Expression::without_parentheses)
                    {
                        // A JS string, so what the lexer produced is the value.
                        Some(Expression::StringLiteral(literal)) => {
                            check_literal_value(
                                attribute,
                                literal.value.as_str(),
                                literal.span,
                                element,
                                false,
                                ctx,
                            );
                        }
                        // `<a rel={5} />`, `<a rel={{}} />`, `<a rel={undefined} />`
                        Some(expr)
                            if is_non_string_literal(expr)
                                || matches!(expr, Expression::ObjectExpression(_))
                                || expr.is_undefined() =>
                        {
                            ctx.diagnostic(non_string_value_diagnostic(name, container.span));
                        }
                        // Anything else only has a value at runtime.
                        _ => {}
                    }
                }
                // React rejects element and fragment attribute values outright.
                JSXAttributeValue::Element(_) | JSXAttributeValue::Fragment(_) => {}
            }
        }
    }

    fn check_create_element_props(
        self,
        props: &ObjectExpression<'_>,
        element: &str,
        ctx: &LintContext<'_>,
    ) {
        for prop in &props.properties {
            // A spread property carries no key to match against.
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                continue;
            };
            // Resolves `rel`, `"rel"` and `["rel"]` alike; a key that is only
            // known at runtime yields `None`.
            let Some(key) = prop.key.static_name() else {
                continue;
            };
            let Some(attribute) = self.checked_attribute(&key) else {
                continue;
            };
            let name = attribute.name();

            if !attribute.elements().contains(element) {
                ctx.diagnostic(invalid_element_diagnostic(name, prop.key.span()));
                continue;
            }

            // `React.createElement("a", { rel() { return 1; } })`
            if prop.method {
                ctx.diagnostic(method_value_diagnostic(name, prop.span));
                continue;
            }

            // `React.createElement("a", { rel: ["noopener", "noreferrer"] })`
            if let Expression::ArrayExpression(array) = &prop.value {
                for value in array.elements.iter().filter_map(ArrayExpressionElement::as_expression)
                {
                    check_create_element_value(attribute, value, element, ctx);
                }
                continue;
            }

            check_create_element_value(attribute, &prop.value, element, ctx);
        }
    }
}

/// Matches the ESTree `Literal` node types that can never hold a string.
fn is_non_string_literal(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::NullLiteral(_)
            | Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::BigIntLiteral(_)
            | Expression::RegExpLiteral(_)
    )
}

/// Calls `f` with the byte offset, text, and whether it is whitespace, for every
/// maximal run of ASCII whitespace or non-whitespace in `value`, in source order.
///
/// The attribute is a space separated token list, which HTML splits on ASCII
/// whitespace only. Other Unicode spaces are ordinary characters that join the
/// text around them into a single token.
fn for_each_run<'s>(value: &'s str, mut f: impl FnMut(usize, &'s str, bool)) {
    let mut start = 0;
    while start < value.len() {
        let rest = &value[start..];
        let is_whitespace = rest.starts_with(|c: char| c.is_ascii_whitespace());
        let end = rest
            .char_indices()
            .find(|(_, c)| c.is_ascii_whitespace() != is_whitespace)
            .map_or(value.len(), |(idx, _)| start + idx);
        f(start, &value[start..end], is_whitespace);
        start = end;
    }
}

/// Looks `key` up in a table whose keys are all lowercase, ignoring ASCII case.
///
/// HTML link types are ASCII case-insensitive, so `NOFOLLOW` names the same link
/// type as `nofollow`. Only a key that actually contains uppercase allocates.
fn get_ignore_ascii_case<'m, V>(map: &'m Map<&'static str, V>, key: &str) -> Option<&'m V> {
    map.get(key.cow_to_ascii_lowercase().as_ref())
}

/// Validates a string value written as a single space delimited list, which is
/// how the attribute is spelled in JSX.
///
/// `span` covers the whole string literal, quotes included. `entities_decoded`
/// says whether the runtime decodes character references in `value`, which is
/// true of a JSX attribute string but not of a JS string in `{...}`.
fn check_literal_value(
    attribute: CheckedAttribute,
    value: &str,
    span: Span,
    element: &str,
    entities_decoded: bool,
    ctx: &LintContext<'_>,
) {
    let name = attribute.name();

    if value.trim_ascii().is_empty() {
        ctx.diagnostic(empty_value_diagnostic(name, span));
        return;
    }

    // Byte offsets into `value` only line up with the source when the literal
    // holds no escapes. When they don't, fall back to labelling the literal as a
    // whole rather than pointing at the wrong text.
    let content_start = (span.size() as usize == value.len() + 2).then_some(span.start + 1);
    #[expect(clippy::cast_possible_truncation)] // offsets within a span already fit in a `u32`
    let part_span = |offset: usize, len: usize| {
        content_start.map_or(span, |start| Span::sized(start + offset as u32, len as u32))
    };

    // A character reference stands for text this rule cannot see, so a value
    // containing one is left alone rather than matched against the source.
    let resolvable = |part: &str| !entities_decoded || !part.contains('&');

    let values = attribute.values();
    let pairs = attribute.pairs();

    for_each_run(value, |offset, run, is_whitespace| {
        let run_span = part_span(offset, run.len());

        if is_whitespace {
            // Leading and trailing whitespace is always redundant; in between,
            // only a single space actually delimits two values.
            if offset == 0 || offset + run.len() == value.len() || run != " " {
                ctx.diagnostic(space_delimited_diagnostic(name, run_span));
            }
            return;
        }

        if !resolvable(run) {
            return;
        }

        let valid = match get_ignore_ascii_case(values, run) {
            None => {
                ctx.diagnostic(never_valid_diagnostic(run, name, run_span));
                false
            }
            Some(elements) if !elements.contains(element) => {
                ctx.diagnostic(invalid_value_diagnostic(run, name, element, run_span));
                false
            }
            Some(_) => true,
        };

        // A value already reported as invalid here has been advised once; adding
        // its partner would not make it valid, so the pairing advice is skipped.
        if valid && let Some(siblings) = get_ignore_ascii_case(pairs, run) {
            check_pair(siblings, run, &value[offset + run.len()..], run_span, &resolvable, ctx);
        }
    });
}

/// Reports `value` unless the value directly following it in `rest` is one it may
/// pair with.
///
/// Upstream matches pairs with an overlapping lookahead regex and compares against
/// the *last* value in the match. That regex never spans more than two values, so
/// its last value is always the immediately following one and the two approaches
/// agree — except when the separator is ASCII whitespace other than a space.
/// Upstream splits the match on `" "` alone, so a tab separated `shortcut\tfoobar`
/// stays a single value, matches no pairing key, and goes unreported there.
fn check_pair(
    siblings: &Set<&'static str>,
    value: &str,
    rest: &str,
    span: Span,
    resolvable: &impl Fn(&str) -> bool,
    ctx: &LintContext<'_>,
) {
    if let Some(next) = rest.split_ascii_whitespace().next() {
        // The partner may be spelled as a character reference.
        if !resolvable(next) || siblings.iter().any(|sibling| sibling.eq_ignore_ascii_case(next)) {
            return;
        }
    }

    let expected =
        siblings.iter().map(|sibling| format!("`{sibling}`")).collect::<Vec<_>>().join(" or ");
    ctx.diagnostic(unpaired_value_diagnostic(value, &expected, span));
}

/// Validates a single `React.createElement` prop value.
///
/// Unlike the JSX form, upstream looks the whole value up as one key here rather
/// than splitting it, so `rel: "shortcut icon"` is accepted but `rel: "a b"` is
/// not.
fn check_create_element_value(
    attribute: CheckedAttribute,
    expr: &Expression<'_>,
    element: &str,
    ctx: &LintContext<'_>,
) {
    let name = attribute.name();
    let expr = expr.without_parentheses();

    let (value, span) = match expr {
        Expression::StringLiteral(literal) => {
            let value = literal.value.as_str();
            // Label the text between the quotes, as the JSX path does. Byte
            // offsets only line up with the source when there are no escapes.
            #[expect(clippy::cast_possible_truncation)] // a literal already fits in a `u32`
            let span = if literal.span.size() as usize == value.len() + 2 {
                Span::sized(literal.span.start + 1, value.len() as u32)
            } else {
                literal.span
            };
            (value, span)
        }
        // No other literal can match a key in the table.
        _ if is_non_string_literal(expr) => {
            let span = expr.span();
            ctx.diagnostic(never_valid_diagnostic(ctx.source_range(span), name, span));
            return;
        }
        // Anything else only has a value at runtime.
        _ => return,
    };

    if value.trim_ascii().is_empty() {
        ctx.diagnostic(empty_value_diagnostic(name, span));
        return;
    }

    match get_ignore_ascii_case(attribute.values(), value) {
        None => ctx.diagnostic(never_valid_diagnostic(value, name, span)),
        Some(elements) if !elements.contains(element) => {
            ctx.diagnostic(invalid_value_diagnostic(value, name, element, span));
        }
        // The whole value is one key here rather than a list, so a value that
        // needs a partner can never be followed by one.
        Some(_) => {
            if let Some(siblings) = get_ignore_ascii_case(attribute.pairs(), value) {
                check_pair(siblings, value, "", span, &|_| true, ctx);
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"<a rel="alternate"></a>"#,
        r#"React.createElement("a", { rel: "alternate" })"#,
        r#"React.createElement("a", { rel: ["alternate"] })"#,
        r#"<a rel="author"></a>"#,
        r#"React.createElement("a", { rel: "author" })"#,
        r#"React.createElement("a", { rel: ["author"] })"#,
        r#"<a rel="bookmark"></a>"#,
        r#"React.createElement("a", { rel: "bookmark" })"#,
        r#"React.createElement("a", { rel: ["bookmark"] })"#,
        r#"<a rel="external"></a>"#,
        r#"React.createElement("a", { rel: "external" })"#,
        r#"React.createElement("a", { rel: ["external"] })"#,
        r#"<a rel="help"></a>"#,
        r#"React.createElement("a", { rel: "help" })"#,
        r#"React.createElement("a", { rel: ["help"] })"#,
        r#"<a rel="license"></a>"#,
        r#"React.createElement("a", { rel: "license" })"#,
        r#"React.createElement("a", { rel: ["license"] })"#,
        r#"<a rel="next"></a>"#,
        r#"React.createElement("a", { rel: "next" })"#,
        r#"React.createElement("a", { rel: ["next"] })"#,
        r#"<a rel="nofollow"></a>"#,
        r#"React.createElement("a", { rel: "nofollow" })"#,
        r#"React.createElement("a", { rel: ["nofollow"] })"#,
        r#"<a rel="noopener"></a>"#,
        r#"React.createElement("a", { rel: "noopener" })"#,
        r#"React.createElement("a", { rel: ["noopener"] })"#,
        r#"<a rel="noreferrer"></a>"#,
        r#"React.createElement("a", { rel: "noreferrer" })"#,
        r#"React.createElement("a", { rel: ["noreferrer"] })"#,
        r#"<a rel="opener"></a>"#,
        r#"React.createElement("a", { rel: "opener" })"#,
        r#"React.createElement("a", { rel: ["opener"] })"#,
        r#"<a rel="prev"></a>"#,
        r#"React.createElement("a", { rel: "prev" })"#,
        r#"React.createElement("a", { rel: ["prev"] })"#,
        r#"<a rel="search"></a>"#,
        r#"React.createElement("a", { rel: "search" })"#,
        r#"React.createElement("a", { rel: ["search"] })"#,
        r#"<a rel="tag"></a>"#,
        r#"React.createElement("a", { rel: "tag" })"#,
        r#"React.createElement("a", { rel: ["tag"] })"#,
        r#"<area rel="alternate"></area>"#,
        r#"React.createElement("area", { rel: "alternate" })"#,
        r#"React.createElement("area", { rel: ["alternate"] })"#,
        r#"<area rel="author"></area>"#,
        r#"React.createElement("area", { rel: "author" })"#,
        r#"React.createElement("area", { rel: ["author"] })"#,
        r#"<area rel="bookmark"></area>"#,
        r#"React.createElement("area", { rel: "bookmark" })"#,
        r#"React.createElement("area", { rel: ["bookmark"] })"#,
        r#"<area rel="external"></area>"#,
        r#"React.createElement("area", { rel: "external" })"#,
        r#"React.createElement("area", { rel: ["external"] })"#,
        r#"<area rel="help"></area>"#,
        r#"React.createElement("area", { rel: "help" })"#,
        r#"React.createElement("area", { rel: ["help"] })"#,
        r#"<area rel="license"></area>"#,
        r#"React.createElement("area", { rel: "license" })"#,
        r#"React.createElement("area", { rel: ["license"] })"#,
        r#"<area rel="next"></area>"#,
        r#"React.createElement("area", { rel: "next" })"#,
        r#"React.createElement("area", { rel: ["next"] })"#,
        r#"<area rel="nofollow"></area>"#,
        r#"React.createElement("area", { rel: "nofollow" })"#,
        r#"React.createElement("area", { rel: ["nofollow"] })"#,
        r#"<area rel="noopener"></area>"#,
        r#"React.createElement("area", { rel: "noopener" })"#,
        r#"React.createElement("area", { rel: ["noopener"] })"#,
        r#"<area rel="noreferrer"></area>"#,
        r#"React.createElement("area", { rel: "noreferrer" })"#,
        r#"React.createElement("area", { rel: ["noreferrer"] })"#,
        r#"<area rel="opener"></area>"#,
        r#"React.createElement("area", { rel: "opener" })"#,
        r#"React.createElement("area", { rel: ["opener"] })"#,
        r#"<area rel="prev"></area>"#,
        r#"React.createElement("area", { rel: "prev" })"#,
        r#"React.createElement("area", { rel: ["prev"] })"#,
        r#"<area rel="search"></area>"#,
        r#"React.createElement("area", { rel: "search" })"#,
        r#"React.createElement("area", { rel: ["search"] })"#,
        r#"<area rel="tag"></area>"#,
        r#"React.createElement("area", { rel: "tag" })"#,
        r#"React.createElement("area", { rel: ["tag"] })"#,
        r#"<link rel="alternate"></link>"#,
        r#"React.createElement("link", { rel: "alternate" })"#,
        r#"React.createElement("link", { rel: ["alternate"] })"#,
        r#"<link rel="author"></link>"#,
        r#"React.createElement("link", { rel: "author" })"#,
        r#"React.createElement("link", { rel: ["author"] })"#,
        r#"<link rel="canonical"></link>"#,
        r#"React.createElement("link", { rel: "canonical" })"#,
        r#"React.createElement("link", { rel: ["canonical"] })"#,
        r#"<link rel="dns-prefetch"></link>"#,
        r#"React.createElement("link", { rel: "dns-prefetch" })"#,
        r#"React.createElement("link", { rel: ["dns-prefetch"] })"#,
        r#"<link rel="help"></link>"#,
        r#"React.createElement("link", { rel: "help" })"#,
        r#"React.createElement("link", { rel: ["help"] })"#,
        r#"<link rel="icon"></link>"#,
        r#"React.createElement("link", { rel: "icon" })"#,
        r#"React.createElement("link", { rel: ["icon"] })"#,
        r#"<link rel="shortcut icon"></link>"#,
        r#"React.createElement("link", { rel: "shortcut icon" })"#,
        r#"React.createElement("link", { rel: ["shortcut icon"] })"#,
        r#"<link rel="license"></link>"#,
        r#"React.createElement("link", { rel: "license" })"#,
        r#"React.createElement("link", { rel: ["license"] })"#,
        r#"<link rel="manifest"></link>"#,
        r#"React.createElement("link", { rel: "manifest" })"#,
        r#"React.createElement("link", { rel: ["manifest"] })"#,
        r#"<link rel="modulepreload"></link>"#,
        r#"React.createElement("link", { rel: "modulepreload" })"#,
        r#"React.createElement("link", { rel: ["modulepreload"] })"#,
        r#"<link rel="next"></link>"#,
        r#"React.createElement("link", { rel: "next" })"#,
        r#"React.createElement("link", { rel: ["next"] })"#,
        r#"<link rel="pingback"></link>"#,
        r#"React.createElement("link", { rel: "pingback" })"#,
        r#"React.createElement("link", { rel: ["pingback"] })"#,
        r#"<link rel="preconnect"></link>"#,
        r#"React.createElement("link", { rel: "preconnect" })"#,
        r#"React.createElement("link", { rel: ["preconnect"] })"#,
        r#"<link rel="prefetch"></link>"#,
        r#"React.createElement("link", { rel: "prefetch" })"#,
        r#"React.createElement("link", { rel: ["prefetch"] })"#,
        r#"<link rel="preload"></link>"#,
        r#"React.createElement("link", { rel: "preload" })"#,
        r#"React.createElement("link", { rel: ["preload"] })"#,
        r#"<link rel="prerender"></link>"#,
        r#"React.createElement("link", { rel: "prerender" })"#,
        r#"React.createElement("link", { rel: ["prerender"] })"#,
        r#"<link rel="prev"></link>"#,
        r#"React.createElement("link", { rel: "prev" })"#,
        r#"React.createElement("link", { rel: ["prev"] })"#,
        r#"<link rel="search"></link>"#,
        r#"React.createElement("link", { rel: "search" })"#,
        r#"React.createElement("link", { rel: ["search"] })"#,
        r#"<link rel="stylesheet"></link>"#,
        r#"React.createElement("link", { rel: "stylesheet" })"#,
        r#"React.createElement("link", { rel: ["stylesheet"] })"#,
        r#"<form rel="external"></form>"#,
        r#"React.createElement("form", { rel: "external" })"#,
        r#"React.createElement("form", { rel: ["external"] })"#,
        r#"<form rel="help"></form>"#,
        r#"React.createElement("form", { rel: "help" })"#,
        r#"React.createElement("form", { rel: ["help"] })"#,
        r#"<form rel="license"></form>"#,
        r#"React.createElement("form", { rel: "license" })"#,
        r#"React.createElement("form", { rel: ["license"] })"#,
        r#"<form rel="next"></form>"#,
        r#"React.createElement("form", { rel: "next" })"#,
        r#"React.createElement("form", { rel: ["next"] })"#,
        r#"<form rel="nofollow"></form>"#,
        r#"React.createElement("form", { rel: "nofollow" })"#,
        r#"React.createElement("form", { rel: ["nofollow"] })"#,
        r#"<form rel="noopener"></form>"#,
        r#"React.createElement("form", { rel: "noopener" })"#,
        r#"React.createElement("form", { rel: ["noopener"] })"#,
        r#"<form rel="noreferrer"></form>"#,
        r#"React.createElement("form", { rel: "noreferrer" })"#,
        r#"React.createElement("form", { rel: ["noreferrer"] })"#,
        r#"<form rel="opener"></form>"#,
        r#"React.createElement("form", { rel: "opener" })"#,
        r#"React.createElement("form", { rel: ["opener"] })"#,
        r#"<form rel="prev"></form>"#,
        r#"React.createElement("form", { rel: "prev" })"#,
        r#"React.createElement("form", { rel: ["prev"] })"#,
        r#"<form rel="search"></form>"#,
        r#"React.createElement("form", { rel: "search" })"#,
        r#"React.createElement("form", { rel: ["search"] })"#,
        "<form rel={callFoo()}></form>",
        r#"React.createElement("form", { rel: callFoo() })"#,
        r#"React.createElement("form", { rel: [callFoo()] })"#,
        r#"<a rel={{a: "noreferrer"}["a"]}></a>"#,
        r#"<a rel={{a: "noreferrer"}["b"]}></a>"#,
        "<Foo rel></Foo>",
        r#"React.createElement("Foo", { rel: true })"#,
        "
                    React.createElement('a', {
                      ...rest,
                      href: to,
                    })
                  ",
        r#"<link rel="apple-touch-icon" sizes="60x60" href="apple-touch-icon-60x60.png" />"#,
        r#"<link rel="apple-touch-icon" sizes="76x76" href="apple-touch-icon-76x76.png" />"#,
        r#"<link rel="apple-touch-icon" sizes="120x120" href="apple-touch-icon-120x120.png" />"#,
        r#"<link rel="apple-touch-icon" sizes="152x152" href="apple-touch-icon-152x152.png" />"#,
        r#"<link rel="apple-touch-icon" sizes="180x180" href="apple-touch-icon-180x180.png" />"#,
        r#"<link rel="apple-touch-startup-image" href="launch.png" />"#,
        r#"<link rel="apple-touch-startup-image" href="iphone5.png" media="(device-width: 320px) and (device-height: 568px) and (-webkit-device-pixel-ratio: 2)" />"#,
        r##"<link rel="mask-icon" href="/safari-pinned-tab.svg" color="#fff" />"##,
        // Template literals are not ESTree `Literal`s, so upstream skips them.
        "<a rel={`foobar`}></a>",
        // The `createElement` form only inspects literals, so an object is
        // skipped here even though the JSX form reports it.
        r#"React.createElement("a", { rel: {} })"#,
        // Shorthand aliases a binding whose value is unknown here.
        r#"React.createElement("a", { rel })"#,
        // A key that is only known at runtime cannot be resolved.
        r#"React.createElement("a", { [rel]: "foobar" })"#,
        // `document.createElement` takes no props argument at all, so the shared
        // `is_create_element_call` helper excludes it.
        r#"document.createElement("a", { rel: "foobar" })"#,
        // A namespaced name is a different attribute.
        r#"<a xlink:rel="foobar"></a>"#,
        // Link types are ASCII case-insensitive.
        r#"<link rel="Stylesheet" href="s.css" />"#,
        r#"<a rel="NOFOLLOW"></a>"#,
        r#"React.createElement("a", { rel: "NoFollow" })"#,
        r#"<link rel="SHORTCUT ICON"></link>"#,
        // Registered link types that are not in the WHATWG table.
        r#"<a rel="me" href="https://mastodon.social/@user"></a>"#,
        r#"<a rel="privacy-policy"></a>"#,
        r#"<a rel="terms-of-service"></a>"#,
        r#"<link rel="expect"></link>"#,
        // React decodes character references, so the source text here is not the
        // value the browser sees and cannot be matched against the table.
        r#"<link rel="&#x73;tylesheet" href="s.css" />"#,
        r#"<link rel="shortcut &#105;con"></link>"#,
        // A pairing is satisfied by the value directly after it; whatever follows
        // that is unrelated. Upstream accepts this too.
        r#"<link rel="shortcut icon stylesheet"></link>"#,
    ];

    let fail = vec![
        r#"<a rel="alternatex"></a>"#,
        r#"React.createElement("a", { rel: "alternatex" })"#,
        r#"React.createElement("a", { rel: ["alternatex"] })"#,
        r#"<a rel="alternatex alternate"></a>"#,
        r#"React.createElement("a", { rel: "alternatex alternate" })"#,
        r#"React.createElement("a", { rel: ["alternatex alternate"] })"#,
        r#"<a rel="alternate alternatex"></a>"#,
        r#"React.createElement("a", { rel: "alternate alternatex" })"#,
        r#"React.createElement("a", { rel: ["alternate alternatex"] })"#,
        "<html rel></html>",
        r#"React.createElement("html", { rel: 1 })"#,
        "<a rel></a>",
        r#"React.createElement("a", { rel: 1 })"#,
        r#"React.createElement("a", { rel() { return 1; } })"#,
        "<span rel></span>",
        "<a rel={null}></a>",
        "<a rel={5}></a>",
        "<a rel={true}></a>",
        "<a rel={{}}></a>",
        "<a rel={undefined}></a>",
        r#"<a rel="noreferrer noopener foobar"></a>"#,
        r#"<a rel="noreferrer noopener   "></a>"#,
        r#"<a rel="noreferrer        noopener"></a>"#,
        r#"<a rel="noreferrer  noopener"></a>"#,
        r#"<a rel={"noreferrer noopener foobar"}></a>"#,
        r#"React.createElement("a", { rel: ["noreferrer", "noopener", "foobar" ] })"#,
        r#"<a rel={"foobar noreferrer noopener"}></a>"#,
        r#"<a rel={"        noopener"}></a>"#,
        r#"<a rel={"noopener        "}></a>"#,
        r#"<a rel={"batgo noopener"}></a>"#,
        r#"<a rel={" noopener"}></a>"#,
        r#"<a rel="canonical"></a>"#,
        r#"<a rel="dns-prefetch"></a>"#,
        r#"<a rel="icon"></a>"#,
        r#"<link rel="shortcut"></link>"#,
        r#"<link rel="shortcut  icon"></link>"#,
        r#"<a rel="manifest"></a>"#,
        r#"<a rel="modulepreload"></a>"#,
        r#"<a rel="pingback"></a>"#,
        r#"<a rel="preconnect"></a>"#,
        r#"<a rel="prefetch"></a>"#,
        r#"<a rel="preload"></a>"#,
        r#"<a rel="prerender"></a>"#,
        r#"<a rel="stylesheet"></a>"#,
        r#"<area rel="canonical"></area>"#,
        r#"<area rel="dns-prefetch"></area>"#,
        r#"<area rel="icon"></area>"#,
        r#"<area rel="manifest"></area>"#,
        r#"<area rel="modulepreload"></area>"#,
        r#"<area rel="pingback"></area>"#,
        r#"<area rel="preconnect"></area>"#,
        r#"<area rel="prefetch"></area>"#,
        r#"<area rel="preload"></area>"#,
        r#"<area rel="prerender"></area>"#,
        r#"<area rel="stylesheet"></area>"#,
        r#"<link rel="bookmark"></link>"#,
        r#"<link rel="external"></link>"#,
        r#"<link rel="nofollow"></link>"#,
        r#"<link rel="noopener"></link>"#,
        r#"<link rel="noreferrer"></link>"#,
        r#"<link rel="opener"></link>"#,
        r#"<link rel="tag"></link>"#,
        r#"<form rel="alternate"></form>"#,
        r#"<form rel="author"></form>"#,
        r#"<form rel="bookmark"></form>"#,
        r#"<form rel="canonical"></form>"#,
        r#"<form rel="dns-prefetch"></form>"#,
        r#"<form rel="icon"></form>"#,
        r#"<form rel="manifest"></form>"#,
        r#"<form rel="modulepreload"></form>"#,
        r#"<form rel="pingback"></form>"#,
        r#"<form rel="preconnect"></form>"#,
        r#"<form rel="prefetch"></form>"#,
        r#"<form rel="preload"></form>"#,
        r#"<form rel="prerender"></form>"#,
        r#"<form rel="stylesheet"></form>"#,
        r#"<form rel="tag"></form>"#,
        r#"<form rel=""></form>"#,
        // Parentheses must be seen through.
        r#"<a rel={("foobar")}></a>"#,
        // An escape makes byte offsets into the value diverge from the source,
        // so the whole literal is labelled instead of a wrong sub-range.
        r#"<a rel={"\x66oobar"}></a>"#,
        // Multi-byte values must be sliced on char boundaries.
        r#"<a rel="日本語 noopener"></a>"#,
        r#"<a rel="noopener 日本語"></a>"#,
        // Non-ASCII whitespace does not delimit values, so this is one token and
        // neither link type is applied. Deliberate deviation from upstream: its
        // JS-regex `\s` treats U+00A0 as whitespace and splits it into two valid
        // tokens (plus a "should be space delimited" warning), whereas HTML only
        // ever splits attribute token lists on ASCII whitespace.
        "<a rel=\"noopener\u{00a0}noreferrer\"></a>",
        // A string literal or computed key resolves the same as an identifier.
        r#"React.createElement("a", { "rel": "foobar" })"#,
        r#"React.createElement("a", { ["rel"]: "foobar" })"#,
        // Every callee form the shared `is_create_element_call` helper accepts,
        // which is wider than upstream's literal `React.createElement` check: a
        // bare call, any object, and a computed property. Matching upstream here
        // would mean diverging from the other react rules built on the helper.
        r#"createElement("a", { rel: "foobar" })"#,
        r#"Preact.createElement("a", { rel: "foobar" })"#,
        r#"Foo["createElement"]("a", { rel: "foobar" })"#,
        // The `createElement` form must reject an empty value too.
        r#"React.createElement("a", { rel: "" })"#,
        r#"React.createElement("a", { rel: "   " })"#,
        // ... and enforce pairing, which only the JSX form used to do.
        r#"React.createElement("link", { rel: "shortcut" })"#,
        r#"React.createElement("link", { rel: ["shortcut"] })"#,
        // A value invalid on this element is reported once, without pairing
        // advice that would not make it valid either.
        r#"<a rel="shortcut"></a>"#,
        // Any ASCII whitespace delimits, so the pairing is checked against the
        // value after the tab. Deliberate deviation from upstream, which splits
        // its pair match on `" "` alone and so never sees `shortcut` here.
        "<link rel=\"shortcut\tfoobar\"></link>",
        // Diagnostics for one attribute are emitted in source order.
        r#"<a rel="  foobar"></a>"#,
        r#"<link rel="stylesheet shortcut foobar"></link>"#,
    ];

    Tester::new(NoInvalidHtmlAttribute::NAME, NoInvalidHtmlAttribute::PLUGIN, pass, fail)
        .test_and_snapshot();

    // The option is the list of attributes to check, defaulting to `["rel"]`.
    let pass_with_config = vec![
        (r#"<a rel="alternate"></a>"#, Some(serde_json::json!([["rel"]]))),
        // An empty list opts out of every check.
        (r#"<a rel="alternatex"></a>"#, Some(serde_json::json!([[]]))),
        ("<a rel></a>", Some(serde_json::json!([[]]))),
    ];

    let fail_with_config = vec![
        (r#"<a rel="alternatex"></a>"#, Some(serde_json::json!([["rel"]]))),
        ("<a rel></a>", Some(serde_json::json!([["rel"]]))),
    ];

    Tester::new(
        NoInvalidHtmlAttribute::NAME,
        NoInvalidHtmlAttribute::PLUGIN,
        pass_with_config,
        fail_with_config,
    )
    .test();
}
