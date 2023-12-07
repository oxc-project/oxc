use oxc_ast::{ast::JSXElementName, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, utils::has_jsx_prop, AstNode};

use phf::phf_set;

const HTML_TAG: phf::Set<&'static str> = phf_set! {
    "a",
    "abbr",
    "acronym",
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "audio",
    "b",
    "base",
    "basefont",
    "bdi",
    "bdo",
    "bgsound",
    "big",
    "blink",
    "blockquote",
    "body",
    "br",
    "button",
    "canvas",
    "caption",
    "center",
    "cite",
    "code",
    "col",
    "colgroup",
    "command",
    "content",
    "data",
    "datalist",
    "dd",
    "del",
    "details",
    "dfn",
    "dialog",
    "dir",
    "div",
    "dl",
    "dt",
    "element",
    "em",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "font",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "i",
    "iframe",
    "image",
    "img",
    "input",
    "ins",
    "isindex",
    "kbd",
    "keygen",
    "label",
    "legend",
    "li",
    "link",
    "listing",
    "main",
    "map",
    "mark",
    "marquee",
    "math",
    "menu",
    "menuitem",
    "meta",
    "meter",
    "multicol",
    "nav",
    "nextid",
    "nobr",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "optgroup",
    "option",
    "output",
    "p",
    "param",
    "picture",
    "plaintext",
    "pre",
    "progress",
    "q",
    "rb",
    "rbc",
    "rp",
    "rt",
    "rtc",
    "ruby",
    "s",
    "samp",
    "script",
    "search",
    "section",
    "select",
    "shadow",
    "slot",
    "small",
    "source",
    "spacer",
    "span",
    "strike",
    "strong",
    "style",
    "sub",
    "summary",
    "sup",
    "svg",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "time",
    "title",
    "tr",
    "track",
    "tt",
    "u",
    "ul",
    "var",
    "video",
    "wbr",
    "xmp",
};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-jsx-a11y(no-autofocus): The `autofocus` attribute is found here")]
#[diagnostic(severity(warning), help("Remove `autofocus` attribute"))]
struct NoAutofocusDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoAutofocus {
    ignore_non_dom: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforce that autoFocus prop is not used on elements. Autofocusing elements can cause usability issues for sighted and non-sighted users, alike.
    ///
    /// ### Rule Option
    /// This rule takes one optional object argument of type object:
    ///
    /// ```
    /// {
    ///     "rules": {
    ///         "jsx-a11y/no-autofocus": [ 2, {
    ///             "ignoreNonDOM": true
    ///         }],
    ///     }
    /// }
    /// ```
    ///
    /// For the `ignoreNonDOM` option, this determines if developer created components are checked.
    ///
    /// ### Example
    /// // good
    ///
    /// ```javascript
    /// <div />
    /// ```
    ///
    /// // bad
    ///
    /// ```
    /// <div autoFocus />
    /// <div autoFocus="true" />
    /// <div autoFocus="false" />
    /// <div autoFocus={undefined} />
    /// ```
    ///
    NoAutofocus,
    correctness
);

impl NoAutofocus {
    pub fn set_option(&mut self, value: bool) {
        self.ignore_non_dom = value;
    }
}

impl Rule for NoAutofocus {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut no_focus = Self::default();

        let _ = value.as_array().unwrap().iter().find(|v| {
            if let serde_json::Value::Object(obj) = v {
                let config = obj.get("ignoreNonDOM").unwrap();
                if let serde_json::Value::Bool(val) = config {
                    no_focus.set_option(*val);
                }
                return true;
            }
            false
        });

        no_focus
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        if let AstKind::JSXElement(jsx_el) = node.kind() {
            if let Option::Some(autofocus) = has_jsx_prop(&jsx_el.opening_element, "autoFocus") {
                if self.ignore_non_dom {
                    let JSXElementName::Identifier(ident) = &jsx_el.opening_element.name else {
                        return;
                    };
                    let name = ident.name.as_str();
                    if HTML_TAG.contains(name) {
                        if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = autofocus {
                            ctx.diagnostic(NoAutofocusDiagnostic(attr.span));
                        }
                    }
                    return;
                }

                if let oxc_ast::ast::JSXAttributeItem::Attribute(attr) = autofocus {
                    ctx.diagnostic(NoAutofocusDiagnostic(attr.span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    fn array() -> serde_json::Value {
        serde_json::json!([2,{
            "ignoreNonDOM": true
        }])
    }

    let pass = vec![
        ("<div />;", None),
        ("<div autofocus />;", None),
        ("<input autofocus='true' />;", None),
        ("<Foo bar />", None),
        ("<Button />", None),
        ("<Foo autoFocus />", Some(array())),
        ("<div><div autofocus /></div>", Some(array())),
        // TODO we need components_settings to test this
        // ("<Button />", Some(serde_json::json!(ignoreNonDOMSchema))),
        // ("<Button />", Some(serde_json::json!(ignoreNonDOMSchema)), setting),
    ];

    let fail = vec![
        ("<div autoFocus />", None),
        ("<div autoFocus={true} />", None),
        ("<div autoFocus={false} />", None),
        ("<div autoFocus={undefined} />", None),
        ("<div autoFocus='true' />", None),
        ("<div autoFocus='false' />", None),
        ("<input autoFocus />", None),
        ("<Foo autoFocus />", None),
        ("<Button autoFocus />", None),
        // TODO we need components_settings to test this
        // ("<Button autoFocus />", Some(array())),
    ];

    Tester::new(NoAutofocus::NAME, pass, fail).test_and_snapshot();
}
