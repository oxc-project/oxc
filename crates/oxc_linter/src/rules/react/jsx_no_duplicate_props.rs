use oxc_ast::{
    AstKind,
    ast::{JSXAttributeItem, JSXAttributeName},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::Deserialize;
use std::borrow::Cow;

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

fn jsx_no_duplicate_props_diagnostic(prop_name: &str, span1: Span, span2: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "No duplicate props allowed. The prop \"{prop_name}\" is duplicated."
    ))
    .with_help("Remove one of the props, or rename them so each prop is distinct.")
    .with_labels([span1, span2])
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct JsxNoDuplicateProps {
    // If set to `true`, the rule will consider props duplicates even if
    // they use different casing.
    //
    // For example, with `ignoreCase` set to `true`, the following code would be considered
    // to have duplicate props:
    //
    // ```jsx
    // <InputField inputProps="foo" InputProps="bar" />;
    // ```
    ignore_case: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule prevents duplicate props in JSX elements.
    ///
    /// ### Why is this bad?
    ///
    /// Having duplicate props in a JSX element is most likely a mistake.
    /// Creating JSX elements with duplicate props can cause unexpected behavior in your application.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <App a a />;
    /// <App foo={2} bar baz foo={3} />;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <App a />;
    /// <App bar baz foo={3} />;
    /// ```
    JsxNoDuplicateProps,
    react,
    correctness,
    config = JsxNoDuplicateProps,
);

impl Rule for JsxNoDuplicateProps {
    fn from_configuration(value: serde_json::Value) -> Self {
        serde_json::from_value::<DefaultRuleConfig<JsxNoDuplicateProps>>(value)
            .unwrap_or_default()
            .into_inner()
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_opening_elem) = node.kind() else {
            return;
        };

        let mut props: FxHashMap<Cow<'a, str>, Span> = FxHashMap::default();

        for attr in &jsx_opening_elem.attributes {
            let JSXAttributeItem::Attribute(jsx_attr) = attr else {
                continue;
            };

            let JSXAttributeName::Identifier(ident) = &jsx_attr.name else {
                continue;
            };

            let ident_name: Cow<'a, str> = if self.ignore_case {
                // Use ASCII lowercase to avoid allocating unless needed
                Cow::Owned(ident.name.as_str().to_ascii_lowercase())
            } else {
                Cow::Borrowed(ident.name.as_str())
            };

            if let Some(old_span) = props.insert(ident_name, ident.span) {
                ctx.diagnostic(jsx_no_duplicate_props_diagnostic(
                    ident.name.as_str(),
                    old_span,
                    ident.span,
                ));
            }
        }
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("<App />;", None),
        ("<App {...this.props} />;", None),
        ("<App a b c />;", None),
        ("<App a b c A />;", None),
        ("<App {...this.props} a b c />;", None),
        ("<App c {...this.props} a b />;", None),
        (r#"<App a="c" b="b" c="a" />;"#, None),
        (r#"<App {...this.props} a="c" b="b" c="a" />;"#, None),
        (r#"<App c="a" {...this.props} a="c" b="b" />;"#, None),
        ("<App A a />;", None),
        ("<App A b a />;", None),
        (r#"<App A="a" b="b" B="B" />;"#, None),
    ];

    let fail = vec![
        ("<App a a />;", None),
        ("<App a a />;", Some(serde_json::json!([{ "ignoreCase": false }]))), // default config
        ("<App A b c A />;", None),
        (r#"<App a="a" b="b" a="a" />;"#, None),
        (r#"<App a="a" {...this.props} b="b" a="a" />;"#, None),
        (r#"<App a b="b" {...this.props} a="a" />;"#, None),
        (r#"<App a={[]} b="b" {...this.props} a="a" />;"#, None),
        (r#"<App a="a" b="b" a="a" {...this.props} />;"#, None),
        (r#"<App {...this.props} a="a" b="b" a="a" />;"#, None),
        (
            r#"
            <App
                a="a"
                {...this.props}
                a={{foo: 'bar'}}
                b="b"
            />;
        "#,
            None,
        ),
        ("<App a a />;", Some(serde_json::json!([{ "ignoreCase": true }]))), // still fails even if they're the same case and ignoreCase is true.
        ("<App A a />;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        ("<App a b c A />;", Some(serde_json::json!([{ "ignoreCase": true }]))),
        (r#"<App A="a" b="b" B="B" />;"#, Some(serde_json::json!([{ "ignoreCase": true }]))),
        (
            r#"<App inputProps="foo" InputProps="bar" />;"#,
            Some(serde_json::json!([{ "ignoreCase": true }])),
        ),
    ];

    Tester::new(JsxNoDuplicateProps::NAME, JsxNoDuplicateProps::PLUGIN, pass, fail)
        .test_and_snapshot();
}
