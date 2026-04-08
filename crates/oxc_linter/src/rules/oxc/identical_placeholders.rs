use std::{collections::BTreeMap, collections::BTreeSet, path::Path};

use super::json_utils::{file_start_span, is_json_file, resolve_reference_path};

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::{DefaultRuleConfig, Rule},
    utils::read_to_string,
};

static PLACEHOLDER_REGEX: Lazy<Regex> = lazy_regex!(r"\{[^}]+\}");

fn placeholder_mismatch_diagnostic(
    reference_path: &Path,
    key: &str,
    summary: &str,
    span: Span,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Placeholders at `{key}` do not match reference locale `{}`.",
        reference_path.display()
    ))
    .with_help(summary.to_string())
    .with_label(span)
}

fn unreadable_reference_diagnostic(
    reference_path: &Path,
    error: &std::io::Error,
    span: Span,
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
pub struct IdenticalPlaceholdersConfig {
    file_path: BTreeMap<String, String>,
}

#[derive(Debug, Default, Clone)]
pub struct IdenticalPlaceholders(Box<IdenticalPlaceholdersConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Ensures locale JSON files use the same ICU/i18n placeholders as a
    /// configured reference catalog.
    ///
    /// ### Why is this bad?
    ///
    /// Missing or extra placeholders in translations cause runtime formatting
    /// errors or display raw placeholder syntax to end-users.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "greeting": "Bonjour" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "greeting": "Bonjour {name}" }
    /// ```
    IdenticalPlaceholders,
    oxc,
    correctness,
    config = IdenticalPlaceholdersConfig
);

impl Rule for IdenticalPlaceholders {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<IdenticalPlaceholdersConfig>>(value)
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
        let candidate_result = parse_json(source_text);
        let Some(candidate) = &candidate_result.root else {
            return;
        };

        let reference_path = resolve_reference_path(ctx.file_path(), raw_reference_path);
        if reference_path == ctx.file_path() {
            return;
        }

        let span = file_start_span(source_text);
        let reference_source = match read_to_string(&reference_path) {
            Ok(s) => s,
            Err(error) => {
                ctx.diagnostic(unreadable_reference_diagnostic(&reference_path, &error, span));
                return;
            }
        };
        let reference_result = parse_json(&reference_source);
        let Some(reference) = &reference_result.root else {
            return;
        };

        compare_placeholders(ctx, &reference_path, reference, candidate, "");
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host() && is_json_file(ctx.file_path())
    }
}

fn compare_placeholders(
    ctx: &LintContext<'_>,
    reference_path: &Path,
    reference: &JsonValue<'_>,
    candidate: &JsonValue<'_>,
    path: &str,
) {
    match (reference, candidate) {
        (JsonValue::Object(ref_obj), JsonValue::Object(cand_obj)) => {
            for ref_prop in &ref_obj.properties {
                let child_path = if path.is_empty() {
                    ref_prop.key.to_string()
                } else {
                    format!("{path}.{}", ref_prop.key)
                };

                if let Some(cand_value) = cand_obj.get(ref_prop.key) {
                    compare_placeholders(
                        ctx,
                        reference_path,
                        &ref_prop.value,
                        cand_value,
                        &child_path,
                    );
                }
            }
        }
        (JsonValue::String(ref_str, _), JsonValue::String(cand_str, cand_span)) => {
            let ref_placeholders = extract_placeholders(ref_str);
            let cand_placeholders = extract_placeholders(cand_str);

            if ref_placeholders != cand_placeholders {
                let missing: Vec<_> =
                    ref_placeholders.difference(&cand_placeholders).cloned().collect();
                let extra: Vec<_> =
                    cand_placeholders.difference(&ref_placeholders).cloned().collect();

                let mut parts = Vec::new();
                if !missing.is_empty() {
                    parts.push(format!("missing: {}", missing.join(", ")));
                }
                if !extra.is_empty() {
                    parts.push(format!("extra: {}", extra.join(", ")));
                }

                ctx.diagnostic(placeholder_mismatch_diagnostic(
                    reference_path,
                    path,
                    &parts.join("; "),
                    *cand_span,
                ));
            }
        }
        _ => {}
    }
}

fn extract_placeholders(message: &str) -> BTreeSet<String> {
    PLACEHOLDER_REGEX.find_iter(message).map(|m| m.as_str().to_string()).collect()
}

#[test]
fn test() {
    use std::path::PathBuf;

    use serde_json::json;

    use crate::tester::Tester;

    let config = Some(json!([{
        "filePath": {
            "messages.json": "../en/messages.json"
        }
    }]));

    let pass = vec![
        // Placeholders match reference
        (
            r#"{"greeting":"Bonjour {name}","count":"{count} éléments","simple":"Tableau de bord"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/fr/messages.json")),
        ),
        // Self-reference (skipped)
        (
            r#"{"greeting":"Hello {name}","count":"{count} items","simple":"Dashboard"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/en/messages.json")),
        ),
    ];

    let fail = vec![
        // Missing {name} placeholder
        (
            r#"{"greeting":"Bonjour","count":"{count} éléments","simple":"Tableau de bord"}"#,
            config.clone(),
            None,
            Some(PathBuf::from("i18n_catalog/fr/messages.json")),
        ),
        // Extra {extra} placeholder
        (
            r#"{"greeting":"Bonjour {name} {extra}","count":"{count} éléments","simple":"Tableau de bord"}"#,
            config,
            None,
            Some(PathBuf::from("i18n_catalog/fr/messages.json")),
        ),
    ];

    Tester::new(IdenticalPlaceholders::NAME, IdenticalPlaceholders::PLUGIN, pass, fail)
        .change_rule_path_extension("json")
        .test_and_snapshot();
}
