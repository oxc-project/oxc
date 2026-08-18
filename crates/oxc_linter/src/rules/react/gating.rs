use oxc_react_compiler::{
    DynamicGatingConfig, ErrorCategory, GatingConfig as CompilerGatingConfig,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::{ContextHost, LintContext},
    rule::{DefaultRuleConfig, Rule},
    utils::{
        react_compiler_plugin_options, run_react_compiler_rule_with_options,
        should_run_react_compiler,
    },
};

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

    fn run_once(&self, ctx: &LintContext) {
        let Some(dynamic_gating) = &self.0.dynamic_gating else {
            return;
        };

        let mut options = react_compiler_plugin_options();
        options.gating = self.0.gating.as_ref().map(|gating| CompilerGatingConfig {
            source: gating.source.clone(),
            import_specifier_name: gating.import_specifier_name.clone(),
        });
        options.dynamic_gating =
            Some(DynamicGatingConfig { source: dynamic_gating.source.clone() });
        run_react_compiler_rule_with_options(ctx, ErrorCategory::Gating, options);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        should_run_react_compiler(ctx)
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
            "function Component() { return <div />; }",
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
}
