use std::{collections::BTreeMap, path::Path};

use super::json_utils::{
    JsonShapeDiff, compare_json_shapes, file_start_span, is_json_file, resolve_reference_path,
};

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
    utils::read_to_string,
};

fn identical_keys_diagnostic(
    reference_path: &Path,
    summary: &str,
    span: oxc_span::Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "JSON keys do not match the reference locale file `{}`.",
        reference_path.display()
    ))
    .with_help(summary.to_string())
    .with_label(span)
}

fn unreadable_reference_diagnostic(
    reference_path: &Path,
    error: &std::io::Error,
    span: oxc_span::Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Configured reference locale file `{}` could not be read.",
        reference_path.display()
    ))
    .with_help(error.to_string())
    .with_label(span)
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct IdenticalKeysConfig {
    file_path: BTreeMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct IdenticalKeys(Box<IdenticalKeysConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures locale JSON files have the same key structure as a configured
    /// reference catalog.
    ///
    /// ### Why is this bad?
    ///
    /// Missing or extra locale keys create partial translations and hard to
    /// debug runtime fallbacks.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "title": "Dashboard" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "title": "Dashboard", "subtitle": "Ready" }
    /// ```
    IdenticalKeys,
    oxc,
    correctness,
    config = IdenticalKeysConfig
);

impl Rule for IdenticalKeys {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<IdenticalKeysConfig>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(|config| Self(Box::new(config)))
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(file_name) = ctx.file_path().file_name().and_then(|name| name.to_str()) else {
            return;
        };
        let Some(raw_reference_path) = self.0.file_path.get(file_name) else {
            return;
        };

        let source_text = ctx.full_source_text();
        let Ok(candidate) = serde_json::from_str::<Value>(source_text) else {
            return;
        };

        let reference_path = resolve_reference_path(ctx.file_path(), raw_reference_path);
        if reference_path == ctx.file_path() {
            return;
        }

        let span = file_start_span(source_text);
        let reference_source = match read_to_string(&reference_path) {
            Ok(reference_source) => reference_source,
            Err(error) => {
                ctx.diagnostic(unreadable_reference_diagnostic(&reference_path, &error, span));
                return;
            }
        };
        let Ok(reference) = serde_json::from_str::<Value>(&reference_source) else {
            return;
        };

        let mut diff = JsonShapeDiff::default();
        compare_json_shapes(&reference, &candidate, "", &mut diff);

        if diff.missing.is_empty() && diff.extra.is_empty() && diff.type_mismatches.is_empty() {
            return;
        }

        ctx.diagnostic(identical_keys_diagnostic(&reference_path, &build_summary(&diff), span));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_json_file(ctx.file_path())
    }
}

fn build_summary(diff: &JsonShapeDiff) -> String {
    let mut parts = Vec::new();

    if !diff.missing.is_empty() {
        parts.push(format!(
            "Missing {} key(s): {}",
            diff.missing.len(),
            summarize_paths(&diff.missing)
        ));
    }

    if !diff.extra.is_empty() {
        parts.push(format!("Extra {} key(s): {}", diff.extra.len(), summarize_paths(&diff.extra)));
    }

    if !diff.type_mismatches.is_empty() {
        parts.push(format!(
            "Type mismatches at {} path(s): {}",
            diff.type_mismatches.len(),
            summarize_paths(&diff.type_mismatches)
        ));
    }

    parts.join(" ")
}

fn summarize_paths(paths: &[String]) -> String {
    const MAX_PATHS: usize = 5;

    let mut summary = paths.iter().take(MAX_PATHS).cloned().collect::<Vec<_>>().join(", ");
    if paths.len() > MAX_PATHS {
        summary.push_str(&format!(", and {} more", paths.len() - MAX_PATHS));
    }
    summary
}

#[test]
fn test() {
    use std::path::PathBuf;

    use serde_json::json;

    use crate::tester::Tester;

    let config = Some(json!([{
        "filePath": {
            "common.json": "../en/common.json"
        }
    }]));

    let pass = vec![
        (
            r#"{"actions":{"cancel":"Annuler","save":"Enregistrer"},"title":"Tableau de bord"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/fr/common.json")),
        ),
        (
            r#"{"actions":{"cancel":"Dashboard","save":"Save"},"title":"Dashboard"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/en/common.json")),
        ),
    ];

    let fail = vec![
        (
            r#"{"actions":{"cancel":"Annuler"},"title":"Tableau de bord"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/fr/common.json")),
        ),
        (
            r#"{"actions":{"cancel":"Annuler","save":"Enregistrer","submit":"Envoyer"},"title":"Tableau de bord"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/fr/common.json")),
        ),
        (
            r#"{"actions":"Annuler","title":"Tableau de bord"}"#,
            config,
            None,
            Some(PathBuf::from("i18n_catalog/fr/common.json")),
        ),
    ];

    Tester::new(IdenticalKeys::NAME, IdenticalKeys::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
