use oxc_ast::{AstKind, ast::FunctionBody};
use oxc_diagnostics::OxcDiagnostic;
use oxc_span::Span;
use oxc_syntax::{identifier::is_identifier_name, keyword::is_reserved_keyword};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AstNode,
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
};

const GATING_GUIDANCE: &str = "React Compiler could not continue with this configuration. Additional guidance: https://react.dev/reference/eslint-plugin-react-hooks/lints/gating";

fn invalid_gating_directive(directive: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Dynamic gating directive is not a valid JavaScript identifier")
        .with_help(format!("Found '{directive}'"))
        .with_label(span.primary_label("Invalid gating condition"))
        .with_note(GATING_GUIDANCE)
}

fn multiple_gating_directives<'a>(
    directives: impl ExactSizeIterator<Item = (&'a str, Span)>,
) -> OxcDiagnostic {
    let mut names = Vec::with_capacity(directives.len());
    let mut labels = Vec::with_capacity(directives.len());
    for (index, (name, span)) in directives.enumerate() {
        names.push(name);
        labels.push(if index == 0 {
            span.primary_label("First gating directive")
        } else {
            span.label("Additional gating directive")
        });
    }

    OxcDiagnostic::warn("Multiple dynamic gating directives found")
        .with_help(format!("Expected a single directive but found [{}]", names.join(", ")))
        .with_labels(labels)
        .with_note(GATING_GUIDANCE)
}

fn parse_dynamic_gating_directive(value: &str) -> Option<&str> {
    let condition = value.strip_prefix("use memo if(")?.strip_suffix(')')?;
    (!condition.contains(')')).then_some(condition)
}

fn is_valid_identifier(value: &str) -> bool {
    is_identifier_name(value) && !is_reserved_keyword(value)
}

fn check_dynamic_gating_directives(body: &FunctionBody, ctx: &LintContext) {
    let mut matches = Vec::new();
    let mut invalid = false;

    for directive in &body.directives {
        let value = directive.expression.value.as_str();
        let Some(condition) = parse_dynamic_gating_directive(value) else { continue };

        if is_valid_identifier(condition) {
            matches.push((value, directive.expression.span));
        } else {
            invalid = true;
            ctx.diagnostic(invalid_gating_directive(value, directive.expression.span));
        }
    }

    if !invalid && matches.len() > 1 {
        ctx.diagnostic(multiple_gating_directives(matches.into_iter()));
    }
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Gating(Box<GatingOptions>);

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct GatingOptions {
    /// React Compiler feature-flag import configuration to validate.
    /// Oxlint does not emit gated code.
    gating: Option<GatingImport>,

    /// Enable validation of `"use memo if(...)"` directives.
    dynamic_gating: Option<DynamicGatingImport>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GatingImport {
    /// Module that exports the gating function.
    source: String,

    /// Name of the imported function that guards compiled functions.
    import_specifier_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DynamicGatingImport {
    /// Module that exports flags referenced by dynamic gating directives.
    source: String,
}

declare_react_compiler_lint!(
    /// ### What it does
    ///
    /// Validates React Compiler gating configuration and reports invalid
    /// dynamic gating directives when `dynamicGating` is configured.
    ///
    upstream = "gating",
    ///
    /// ### Why is this bad?
    ///
    /// An invalid gating directive prevents React Compiler from selecting the
    /// compiled component at runtime.
    Gating,
    react,
    correctness,
    config = GatingOptions,
    version = "next",
    short_description = "Validates React Compiler gating configuration and directives.",
);

impl Rule for Gating {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::FunctionBody(body) = node.kind() else { return };
        check_dynamic_gating_directives(body, ctx);
    }

    fn should_run(&self, _ctx: &ContextHost) -> bool {
        self.0.dynamic_gating.is_some()
    }
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

    let pass = vec![
        (
            "
function Component(props) {
  return <div>{props.text}</div>;
}
",
            None,
        ),
        // Dynamic gating directives are ignored unless configured.
        (
            "
function Component() {
  'use memo if(true)';
  return <div />;
}
",
            None,
        ),
        // An empty option object does not enable dynamic gating validation.
        (
            "
const Component = () => {
  'use memo if(true)';
  return <div />;
};
",
            Some(json!([{}])),
        ),
        (
            "
function Component() {
  'use memo if(isCompilerEnabled)';
  return <div />;
}
",
            Some(json!([{ "dynamicGating": { "source": "feature-flags" } }])),
        ),
        (
            "
function Component() {
  'use memo if(true)';
  return <div />;
}
",
            Some(json!([{
                "gating": {
                    "source": "feature-flags",
                    "importSpecifierName": "isCompilerEnabled"
                }
            }])),
        ),
    ];

    let fail = vec![
        // The gating condition must be an identifier.
        (
            "
function Component() {
  'use memo if(true)';
  return <div />;
}
",
            Some(json!([{ "dynamicGating": { "source": "feature-flags" } }])),
        ),
        // Arrow function bodies use the same directive validation path.
        (
            "
const Component = () => {
  'use memo if(true)';
  return <div />;
};
",
            Some(json!([{ "dynamicGating": { "source": "feature-flags" } }])),
        ),
        // A function can only have one dynamic gating directive.
        (
            "
function Component() {
  'use memo if(isCompilerEnabled)';
  'use memo if(isNewCompilerEnabled)';
  return <div />;
}
",
            Some(json!([{ "dynamicGating": { "source": "feature-flags" } }])),
        ),
    ];

    Tester::new(Gating::NAME, Gating::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_configuration() {
    use serde_json::json;

    assert!(
        Gating::from_configuration(json!([{
            "gating": {
                "source": "feature-flags",
                "importSpecifierName": "isCompilerEnabled"
            },
            "dynamicGating": { "source": "feature-flags" }
        }]))
        .is_ok()
    );
    assert!(
        Gating::from_configuration(json!([{
            "gating": { "importSpecifierName": "isCompilerEnabled" }
        }]))
        .is_err()
    );
    assert!(
        Gating::from_configuration(json!([{
            "dynamicGating": {}
        }]))
        .is_err()
    );
}
