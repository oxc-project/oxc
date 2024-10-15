use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{get_element_type, is_hidden_from_screen_reader, object_has_accessible_child},
    AstNode,
};

fn heading_has_content_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(
        "Headings must have content and the content must be accessible by a screen reader.",
    )
    .with_help("Provide screen reader accessible content when using heading elements.")
    .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct HeadingHasContent(Box<HeadingHasContentConfig>);

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HeadingHasContentConfig {
    components: Option<Vec<CompactStr>>,
}

impl std::ops::Deref for HeadingHasContent {
    type Target = HeadingHasContentConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforce that heading elements (h1, h2, etc.) have content and
    /// that the content is accessible to screen readers.
    /// Accessible means that it is not hidden using the aria-hidden prop.
    ///
    /// ### Why is this bad?
    ///
    /// Screen readers alert users to the presence of a heading tag.
    /// If the heading is empty or the text cannot be accessed,
    /// this could either confuse users or even prevent them
    /// from accessing information on the page's structure.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <h1 />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <h1>Foo</h1>
    /// ```
    HeadingHasContent,
    correctness
);

// always including <h1> thru <h6>
const DEFAULT_COMPONENTS: [&str; 6] = ["h1", "h2", "h3", "h4", "h5", "h6"];

impl Rule for HeadingHasContent {
    fn from_configuration(value: serde_json::Value) -> Self {
        Self(Box::new(HeadingHasContentConfig {
            components: value
                .get(0)
                .and_then(|v| v.get("components"))
                .and_then(serde_json::Value::as_array)
                .map(|v| {
                    v.iter().filter_map(serde_json::Value::as_str).map(CompactStr::from).collect()
                }),
        }))
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        // let JSXElementName::Identifier(iden) = &jsx_el.name else {
        //     return;
        // };

        // let name = iden.name.as_str();
        let Some(name) = &get_element_type(ctx, jsx_el) else {
            return;
        };

        if !DEFAULT_COMPONENTS.iter().any(|&comp| comp == name)
            && !self
                .components
                .as_ref()
                .is_some_and(|components| components.iter().any(|comp| comp == name))
        {
            return;
        }

        let maybe_parent = ctx.nodes().parent_node(node.id()).map(oxc_semantic::AstNode::kind);
        if let Some(AstKind::JSXElement(parent)) = maybe_parent {
            if object_has_accessible_child(ctx, parent) {
                return;
            }
        }

        if is_hidden_from_screen_reader(ctx, jsx_el) {
            return;
        }

        ctx.diagnostic(heading_has_content_diagnostic(jsx_el.span));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn components() -> serde_json::Value {
        serde_json::json!([{
            "components": ["Heading", "Title"],
        }])
    }

    fn settings() -> serde_json::Value {
        serde_json::json!({
          "settings": { "jsx-a11y": {
            "components": {
              "CustomInput": "input",
              "Title": "h1",
              "Heading": "h2",
            },
          } }
        })
    }

    let pass = vec![
        // DEFAULT ELEMENT TESTS
        (r"<h1>Foo</h1>", None, None),
        (r"<h2>Foo</h2>", None, None),
        (r"<h3>Foo</h3>", None, None),
        (r"<h4>Foo</h4>", None, None),
        (r"<h5>Foo</h5>", None, None),
        (r"<h6>Foo</h6>", None, None),
        (r"<h6>123</h6>", None, None),
        (r"<h1><Bar /></h1>", None, None),
        (r"<h1>{foo}</h1>", None, None),
        (r"<h1>{foo.bar}</h1>", None, None),
        (r#"<h1 dangerouslySetInnerHTML={{ __html: "foo" }} />"#, None, None),
        (r"<h1 children={children} />", None, None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS OPTION
        (r"<Heading>Foo</Heading>", Some(components()), None),
        (r"<Title>Foo</Title>", Some(components()), None),
        (r"<Heading><Bar /></Heading>", Some(components()), None),
        (r"<Heading>{foo}</Heading>", Some(components()), None),
        (r"<Heading>{foo.bar}</Heading>", Some(components()), None),
        (r#"<Heading dangerouslySetInnerHTML={{ __html: "foo" }} />"#, Some(components()), None),
        (r"<Heading children={children} />", Some(components()), None),
        (r"<h1 aria-hidden />", Some(components()), None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS SETTINGS
        (r"<Heading>Foo</Heading>", None, Some(settings())),
        (r#"<h1><CustomInput type="hidden" /></h1>"#, None, None),
    ];

    let fail = vec![
        // DEFAULT ELEMENT TESTS
        (r"<h1 />", None, None),
        (r"<h1><Bar aria-hidden /></h1>", None, None),
        (r"<h1>{undefined}</h1>", None, None),
        (r"<h1><></></h1>", None, None),
        (r#"<h1><input type="hidden" /></h1>"#, None, None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS OPTION
        (r"<Heading />", Some(components()), None),
        (r"<Heading><Bar aria-hidden /></Heading>", Some(components()), None),
        (r"<Heading>{undefined}</Heading>", Some(components()), None),
        // CUSTOM ELEMENT TESTS FOR COMPONENTS SETTINGS
        (r"<Heading />", None, Some(settings())),
        // TODO: This should be failed but pass for now
        // (r#"<h1><CustomInput type="hidden" /></h1>"#, None, Some(settings())),
    ];

    Tester::new(HeadingHasContent::NAME, pass, fail).with_jsx_a11y_plugin(true).test_and_snapshot();
}
