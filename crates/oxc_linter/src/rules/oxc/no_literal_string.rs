use lazy_regex::Regex;
use schemars::JsonSchema;
use serde::Deserialize;

use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::get_element_type,
};

fn no_literal_string_diagnostic(span: Span, text: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Literal JSX text should be translated: `{text}`."))
        .with_help(
            "Wrap user-facing text in your translation layer instead of rendering it inline.",
        )
        .with_label(span)
}

#[derive(Debug, Clone, Default, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct PatternGroup {
    include: Vec<String>,
    exclude: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct NoLiteralStringConfig {
    framework: String,
    mode: String,
    #[serde(rename = "jsx-components")]
    jsx_components: PatternGroup,
    #[serde(rename = "jsx-attributes")]
    jsx_attributes: PatternGroup,
    words: PatternGroup,
    callees: PatternGroup,
}

impl Default for NoLiteralStringConfig {
    fn default() -> Self {
        Self {
            framework: "react".to_string(),
            mode: "jsx-text-only".to_string(),
            jsx_components: PatternGroup {
                include: Vec::new(),
                exclude: vec!["Trans".to_string()],
            },
            jsx_attributes: PatternGroup::default(),
            words: PatternGroup::default(),
            callees: PatternGroup::default(),
        }
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct NoLiteralString(Box<NoLiteralStringConfig>);

impl std::ops::Deref for NoLiteralString {
    type Target = NoLiteralStringConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Flags user-facing literal JSX text that should come from a translation layer.
    ///
    /// ### Why is this bad?
    ///
    /// Inline UI strings are easy to miss during localization work and make
    /// it harder to keep translated interfaces complete.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```tsx
    /// <button>Save</button>
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```tsx
    /// <button>{t("actions.save")}</button>
    /// ```
    NoLiteralString,
    oxc,
    pedantic,
    none,
    config = NoLiteralStringConfig
);

impl Rule for NoLiteralString {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::JSXText(jsx_text) = node.kind() else {
            return;
        };

        if self.framework != "react" || self.mode != "jsx-text-only" {
            return;
        }

        let trimmed = jsx_text.value.as_str().trim();
        if trimmed.is_empty() {
            return;
        }

        if should_skip(&self.words, trimmed)
            || is_in_excluded_component(node.id(), ctx, &self.jsx_components)
        {
            return;
        }

        ctx.diagnostic(no_literal_string_diagnostic(jsx_text.span, trimmed));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.source_type().is_jsx()
    }
}

fn is_in_excluded_component(
    node_id: oxc_syntax::node::NodeId,
    ctx: &LintContext<'_>,
    components: &PatternGroup,
) -> bool {
    ctx.nodes().ancestors(node_id).any(|ancestor| {
        let AstKind::JSXElement(element) = ancestor.kind() else {
            return false;
        };
        let name = get_element_type(ctx, &element.opening_element);
        should_skip(components, name.as_ref())
    })
}

fn should_skip(patterns: &PatternGroup, text: &str) -> bool {
    if patterns.include.is_empty() && patterns.exclude.is_empty() {
        return false;
    }

    if !patterns.include.is_empty() && match_patterns(&patterns.include, text) {
        return false;
    }

    if !patterns.exclude.is_empty() && !match_patterns(&patterns.exclude, text) {
        return false;
    }

    true
}

fn match_patterns(patterns: &[String], text: &str) -> bool {
    patterns.iter().any(|pattern| matches_pattern(pattern, text))
}

fn matches_pattern(pattern: &str, text: &str) -> bool {
    let regex_source = format!("(^|\\.){pattern}{}", if pattern.ends_with('$') { "" } else { "$" });
    Regex::new(&regex_source).is_ok_and(|regex| regex.is_match(text))
}

#[test]
fn test() {
    use serde_json::{Value, json};

    use crate::tester::Tester;

    let config = Some(json!([{
        "framework": "react",
        "mode": "jsx-text-only",
        "words": {
            "exclude": ["\\d+", "[A-Z_-]+", "\\S+\\/\\S+", "\\s+"]
        },
        "callees": {
            "exclude": ["t", "i18n\\.t", "console\\.\\w+", "require", "import"]
        }
    }]));

    let pass: Vec<(&str, Option<Value>)> = vec![
        ("const el = <div>{t(\"actions.save\")}</div>;", config.clone()),
        ("const el = <div>{\"Save\"}</div>;", config.clone()),
        ("const el = <div>12345</div>;", config.clone()),
        ("const el = <div>API_URL</div>;", config.clone()),
        ("const el = <div>foo/bar</div>;", config.clone()),
        ("const el = <div>   </div>;", config.clone()),
        ("const el = <Trans>Hello world</Trans>;", config.clone()),
        ("const el = <I18n.Trans>Hello world</I18n.Trans>;", config.clone()),
    ];

    let fail: Vec<(&str, Option<Value>)> = vec![
        ("const el = <div>Hello world</div>;", config.clone()),
        ("const el = <><span>Save</span></>;", config.clone()),
        ("const el = <Button>Submit</Button>;", config.clone()),
    ];

    Tester::new(NoLiteralString::NAME, NoLiteralString::PLUGIN, pass, fail)
        .change_rule_path_extension("tsx")
        .test_and_snapshot();
}
