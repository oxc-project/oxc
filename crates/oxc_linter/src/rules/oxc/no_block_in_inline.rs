use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::{AstKind, ast::JSXOpeningElement};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

const DEFAULT_BLOCK_ELEMENTS: [&str; 39] = [
    "address",
    "article",
    "aside",
    "blockquote",
    "dd",
    "details",
    "dialog",
    "div",
    "dl",
    "dt",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "header",
    "hr",
    "li",
    "main",
    "nav",
    "ol",
    "p",
    "pre",
    "section",
    "summary",
    "table",
    "tbody",
    "td",
    "tfoot",
    "th",
    "thead",
    "tr",
    "ul",
];

const DEFAULT_INLINE_ELEMENTS: [&str; 18] = [
    "a", "abbr", "b", "button", "cite", "code", "em", "i", "label", "mark", "q", "small", "span",
    "strong", "sub", "sup", "time", "u",
];

fn no_block_in_inline_diagnostic(span: Span, block: &str, inline: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Block-level element `<{block}>` must not be nested inside inline element `<{inline}>`."
    ))
    .with_help(
        "Use a block parent instead, or replace the nested block element with inline markup.",
    )
    .with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
#[expect(clippy::struct_field_names)]
pub struct NoBlockInInlineConfig {
    block_elements: Option<Vec<String>>,
    inline_elements: Option<Vec<String>>,
    additional_block_elements: Vec<String>,
    additional_inline_elements: Vec<String>,
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoBlockInInline(Box<NoBlockInInlineConfig>);

impl std::ops::Deref for NoBlockInInline {
    type Target = NoBlockInInlineConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Forbids block-level HTML elements nested inside inline HTML elements in JSX.
    ///
    /// ### Why is this bad?
    ///
    /// Block-level HTML inside inline HTML produces invalid DOM structure and can
    /// lead to layout and accessibility issues.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```tsx
    /// <span><div /></span>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```tsx
    /// <div><span /></div>
    /// ```
    NoBlockInInline,
    oxc,
    correctness,
    none,
    config = NoBlockInInlineConfig
);

impl Rule for NoBlockInInline {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXElement(element) = node.kind() else {
            return;
        };

        let Some(block_name) = get_html_element_name(&element.opening_element) else {
            return;
        };

        if !self.is_block_element(block_name) {
            return;
        }

        for ancestor in ctx.nodes().ancestors(node.id()) {
            let AstKind::JSXElement(parent_element) = ancestor.kind() else {
                continue;
            };

            let Some(inline_name) = get_html_element_name(&parent_element.opening_element) else {
                continue;
            };

            if self.is_inline_element(inline_name) {
                ctx.diagnostic(no_block_in_inline_diagnostic(
                    element.opening_element.span,
                    block_name,
                    inline_name,
                ));
                return;
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

impl NoBlockInInline {
    fn is_block_element(&self, name: &str) -> bool {
        self.block_elements.as_ref().map_or_else(
            || {
                DEFAULT_BLOCK_ELEMENTS.contains(&name)
                    || self.additional_block_elements.iter().any(|candidate| candidate == name)
            },
            |elements| elements.iter().any(|candidate| candidate == name),
        )
    }

    fn is_inline_element(&self, name: &str) -> bool {
        self.inline_elements.as_ref().map_or_else(
            || {
                DEFAULT_INLINE_ELEMENTS.contains(&name)
                    || self.additional_inline_elements.iter().any(|candidate| candidate == name)
            },
            |elements| elements.iter().any(|candidate| candidate == name),
        )
    }
}

fn get_html_element_name<'a>(opening_element: &JSXOpeningElement<'a>) -> Option<&'a str> {
    let oxc_ast::ast::JSXElementName::Identifier(identifier) = &opening_element.name else {
        return None;
    };

    let name = identifier.name.as_str();
    name.chars().next().is_some_and(|character| character.is_ascii_lowercase()).then_some(name)
}

#[test]
fn test() {
    use serde_json::Value;
    use serde_json::json;

    use crate::tester::Tester;

    let pass: Vec<(&str, Option<Value>)> = vec![
        ("const el = <div><span>Hello</span></div>;", None),
        ("const el = <span><strong>Hello</strong></span>;", None),
        ("const el = <div><section><p>Hello</p></section></div>;", None),
        ("const el = <Inline><div /></Inline>;", None),
        ("const el = <span><motion.div /></span>;", None),
        (
            "const el = <span><div /></span>;",
            Some(
                json!([{ "blockElements": ["custom-block"], "inlineElements": ["custom-inline"] }]),
            ),
        ),
    ];

    let fail: Vec<(&str, Option<Value>)> = vec![
        ("const el = <span><div /></span>;", None),
        ("const el = <a><p>bad</p></a>;", None),
        ("const el = <span><em><section /></em></span>;", None),
        (
            "const el = <custom-inline><custom-block /></custom-inline>;",
            Some(
                json!([{ "additionalBlockElements": ["custom-block"], "additionalInlineElements": ["custom-inline"] }]),
            ),
        ),
    ];

    Tester::new(NoBlockInInline::NAME, NoBlockInInline::PLUGIN, pass, fail).test_and_snapshot();
}
