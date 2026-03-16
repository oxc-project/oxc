use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    LintContext,
    context::ContextHost,
    rule::{DefaultRuleConfig, Rule},
    utils::get_element_type,
};

fn no_distracting_elements_diagnostic(span: Span, element: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Do not use `<{element}>` elements as they can create visual accessibility issues and are deprecated."
    ))
        .with_help(format!(
            "Replace the `<{element}>` element with alternative, more accessible ways to achieve your desired visual effects."
        ))
        .with_label(span)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum DistractingElement {
    Marquee,
    Blink,
}

fn default_elements() -> Vec<DistractingElement> {
    vec![DistractingElement::Marquee, DistractingElement::Blink]
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct NoDistractingElementsConfig {
    /// List of distracting elements to check for.
    #[serde(default = "default_elements")]
    elements: Vec<DistractingElement>,
}

impl Default for NoDistractingElementsConfig {
    fn default() -> Self {
        Self { elements: default_elements() }
    }
}

#[derive(Debug, Clone)]
pub struct NoDistractingElements {
    check_marquee: bool,
    check_blink: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces that no distracting elements are used.
    ///
    /// ### Why is this bad?
    ///
    /// Elements that can be visually distracting can cause accessibility issues
    /// with visually impaired users. Such elements are most likely deprecated,
    /// and should be avoided. By default, `<marquee>` and `<blink>` elements
    /// are visually distracting and can trigger vestibular disorders.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```jsx
    /// <marquee />
    /// <marquee {...props} />
    /// <marquee lang={undefined} />
    /// <blink />
    /// <blink {...props} />
    /// <blink foo={undefined} />
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```jsx
    /// <div />
    /// <Marquee />
    /// <Blink />
    /// ```
    NoDistractingElements,
    jsx_a11y,
    correctness,
    config = NoDistractingElementsConfig,
);

impl Rule for NoDistractingElements {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        let config =
            serde_json::from_value::<DefaultRuleConfig<NoDistractingElementsConfig>>(value)
                .map(DefaultRuleConfig::into_inner)?;

        Ok(Self {
            check_marquee: config.elements.contains(&DistractingElement::Marquee),
            check_blink: config.elements.contains(&DistractingElement::Blink),
        })
    }

    fn should_run(&self, _ctx: &ContextHost) -> bool {
        self.check_marquee || self.check_blink
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXOpeningElement(jsx_el) = node.kind() else {
            return;
        };

        let element_type = get_element_type(ctx, jsx_el);

        match element_type.as_ref() {
            "marquee" if self.check_marquee => {
                ctx.diagnostic(no_distracting_elements_diagnostic(jsx_el.name.span(), "marquee"));
            }
            "blink" if self.check_blink => {
                ctx.diagnostic(no_distracting_elements_diagnostic(jsx_el.name.span(), "blink"));
            }
            _ => {}
        }
    }
}

impl Default for NoDistractingElements {
    fn default() -> Self {
        Self { check_marquee: true, check_blink: true }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    fn settings() -> serde_json::Value {
        serde_json::json!({
            "settings": { "jsx-a11y": {
                "components": {
                    "Blink": "blink",
                    "Marquee": "marquee"
                }
            } }
        })
    }

    let pass = vec![
        (r"<div />", None, None),
        (r"<Marquee />", None, None),
        (r"<div marquee />", None, None),
        (r"<Blink />", None, None),
        (r"<div blink />", None, None),
        (r"<marquee />", Some(serde_json::json!([{ "elements": [] }])), None),
        (r"<blink />", Some(serde_json::json!([{ "elements": ["marquee"] }])), None),
    ];

    let fail = vec![
        (r"<marquee />", None, None),
        (r"<marquee {...props} />", None, None),
        (r"<marquee lang={undefined} />", None, None),
        (r"<blink />", None, None),
        (r"<blink {...props} />", None, None),
        (r"<blink foo={undefined} />", None, None),
        (r"<marquee />", Some(serde_json::json!([{ "elements": ["marquee"] }])), None),
        (r"<blink />", Some(serde_json::json!([{ "elements": ["blink"] }])), None),
        (r"<Blink />", None, Some(settings())),
        (r"<Marquee />", None, Some(settings())),
    ];

    Tester::new(NoDistractingElements::NAME, NoDistractingElements::PLUGIN, pass, fail)
        .test_and_snapshot();
}
