use cow_utils::CowUtils;
use rustc_hash::FxHashMap;
use serde::Serialize;

use oxc_diagnostics::{
    Error, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info, InfoPosition},
};
use oxc_linter::rules::{RULES, RuleEnum};

use crate::output_formatter::InternalFormatter;

const SARIF_VERSION: &str = "2.1.0";
const SARIF_SCHEMA: &str =
    "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json";
const OXLINT_INFORMATION_URI: &str = "https://oxc.rs/docs/guide/usage/linter.html";
const SYNTHETIC_ARTIFACT_RULE_ID: &str = "OXL0001";
const CONFIGURATION_NOTIFICATION_ID: &str = "OXL0999";

#[derive(Debug, Default)]
pub struct SarifOutputFormatter;

impl InternalFormatter for SarifOutputFormatter {
    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::<SarifReporter>::default()
    }
}

#[derive(Debug, Default)]
struct SarifReporter {
    diagnostics: Vec<Error>,
}

impl DiagnosticReporter for SarifReporter {
    fn finish(&mut self, _: &DiagnosticResult) -> Option<String> {
        Some(format_sarif(&mut self.diagnostics))
    }

    fn supports_minified_file_fallback(&self) -> bool {
        false
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.diagnostics.push(error);
        None
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLog {
    version: &'static str,
    #[serde(rename = "$schema")]
    schema: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRun {
    tool: SarifTool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    artifacts: Vec<SarifArtifact>,
    results: Vec<SarifResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invocations: Option<Vec<SarifInvocation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_kind: Option<&'static str>,
}

#[derive(Debug, Serialize)]
struct SarifTool {
    driver: SarifToolComponent,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifToolComponent {
    name: &'static str,
    version: &'static str,
    semantic_version: &'static str,
    information_uri: &'static str,
    rules: Vec<SarifReportingDescriptor>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notifications: Vec<SarifReportingDescriptor>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifReportingDescriptor {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short_description: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    full_description: Option<SarifMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<SarifRuleProperties>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRuleProperties {
    category: String,
    plugin: String,
    fix: String,
}

#[derive(Debug, Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Serialize)]
struct SarifArtifact {
    location: SarifArtifactLocation,
}

#[derive(Debug, Serialize)]
struct SarifArtifactLocation {
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_index: Option<usize>,
    level: &'static str,
    message: SarifMessage,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_column: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifInvocation {
    execution_successful: bool,
    tool_configuration_notifications: Vec<SarifNotification>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifNotification {
    descriptor: SarifReportingDescriptorReference,
    level: &'static str,
    message: SarifMessage,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    locations: Vec<SarifLocation>,
}

#[derive(Debug, Serialize)]
struct SarifReportingDescriptorReference {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
}

#[derive(Debug, Default)]
struct ArtifactRegistry {
    artifacts: Vec<SarifArtifact>,
    indices: FxHashMap<String, usize>,
}

impl ArtifactRegistry {
    fn get_or_insert(&mut self, uri: String) -> usize {
        if let Some(index) = self.indices.get(&uri) {
            return *index;
        }

        let index = self.artifacts.len();
        self.indices.insert(uri.clone(), index);
        self.artifacts.push(SarifArtifact { location: SarifArtifactLocation { uri, index: None } });
        index
    }
}

struct SarifBuilder {
    artifacts: ArtifactRegistry,
    rule_lookup: Option<FxHashMap<String, usize>>,
    rule_indices: FxHashMap<String, usize>,
    rules: Vec<SarifReportingDescriptor>,
    results: Vec<SarifResult>,
    notifications: Vec<SarifNotification>,
    has_text_result_location: bool,
    execution_successful: bool,
}

impl SarifBuilder {
    fn new() -> Self {
        Self {
            artifacts: ArtifactRegistry::default(),
            rule_lookup: None,
            rule_indices: FxHashMap::default(),
            rules: Vec::new(),
            results: Vec::new(),
            notifications: Vec::new(),
            has_text_result_location: false,
            execution_successful: true,
        }
    }

    fn add_diagnostic(&mut self, diagnostic: &Error) {
        let info = Info::new(diagnostic);
        let severity = diagnostic.severity().unwrap_or(info.severity);
        let level = sarif_level(severity);
        let message = sarif_message(if info.message.is_empty() {
            diagnostic.to_string()
        } else {
            info.message.clone()
        });
        let location = self.location_from_info(&info);

        if let Some(rule_id) = info.rule_id {
            let rule_index = self.get_rule_index(&rule_id);
            self.push_result(rule_id, rule_index, level, message, location);
        } else if location.is_some() {
            let rule_index = self.get_synthetic_artifact_rule_index();
            self.push_result(
                SYNTHETIC_ARTIFACT_RULE_ID.to_string(),
                Some(rule_index),
                level,
                message,
                location,
            );
        } else {
            if matches!(severity, Severity::Error) {
                self.execution_successful = false;
            }
            self.notifications.push(SarifNotification {
                descriptor: SarifReportingDescriptorReference {
                    id: CONFIGURATION_NOTIFICATION_ID.to_string(),
                    index: Some(0),
                },
                level,
                message,
                locations: Vec::new(),
            });
        }
    }

    fn location_from_info(&mut self, info: &Info) -> Option<SarifLocation> {
        if info.filename.is_empty() {
            return None;
        }

        let uri = normalize_uri(&info.filename);
        let index = self.artifacts.get_or_insert(uri.clone());
        Some(SarifLocation {
            physical_location: SarifPhysicalLocation {
                artifact_location: SarifArtifactLocation { uri, index: Some(index) },
                region: region_from_positions(&info.start, &info.end),
            },
        })
    }

    fn push_result(
        &mut self,
        rule_id: String,
        rule_index: Option<usize>,
        level: &'static str,
        message: SarifMessage,
        location: Option<SarifLocation>,
    ) {
        if location.as_ref().is_some_and(|location| location.physical_location.region.is_some()) {
            self.has_text_result_location = true;
        }

        // TODO: Emit `suppressions` once oxlint exposes bulk suppression data to formatters.
        self.results.push(SarifResult {
            rule_id,
            rule_index,
            level,
            message,
            locations: location.into_iter().collect(),
        });
    }

    fn get_rule_index(&mut self, rule_id: &str) -> Option<usize> {
        if let Some(index) = self.rule_indices.get(rule_id) {
            return Some(*index);
        }

        let rule_lookup = self.rule_lookup.get_or_insert_with(build_rule_lookup);
        let rule = RULES.get(*rule_lookup.get(rule_id)?)?;
        let index = self.rules.len();
        self.rules.push(rule_descriptor(rule_id, rule));
        self.rule_indices.insert(rule_id.to_string(), index);
        Some(index)
    }

    fn get_synthetic_artifact_rule_index(&mut self) -> usize {
        if let Some(index) = self.rule_indices.get(SYNTHETIC_ARTIFACT_RULE_ID) {
            return *index;
        }

        let index = self.rules.len();
        self.rules.push(synthetic_artifact_rule_descriptor());
        self.rule_indices.insert(SYNTHETIC_ARTIFACT_RULE_ID.to_string(), index);
        index
    }

    fn finish(self) -> SarifLog {
        let has_notifications = !self.notifications.is_empty();
        let invocations = has_notifications.then(|| {
            vec![SarifInvocation {
                execution_successful: self.execution_successful,
                tool_configuration_notifications: self.notifications,
            }]
        });
        let notifications =
            has_notifications.then(configuration_notification_descriptor).into_iter().collect();

        SarifLog {
            version: SARIF_VERSION,
            schema: SARIF_SCHEMA,
            runs: vec![SarifRun {
                tool: SarifTool {
                    driver: SarifToolComponent {
                        name: "oxlint",
                        version: env!("CARGO_PKG_VERSION"),
                        semantic_version: env!("CARGO_PKG_VERSION"),
                        information_uri: OXLINT_INFORMATION_URI,
                        rules: self.rules,
                        notifications,
                    },
                },
                artifacts: self.artifacts.artifacts,
                results: self.results,
                invocations,
                column_kind: self.has_text_result_location.then_some("unicodeCodePoints"),
            }],
        }
    }
}

fn build_rule_lookup() -> FxHashMap<String, usize> {
    RULES
        .iter()
        .enumerate()
        .map(|(index, rule)| (format!("{}({})", rule.plugin_name(), rule.name()), index))
        .collect()
}

fn rule_descriptor(rule_id: &str, rule: &RuleEnum) -> SarifReportingDescriptor {
    SarifReportingDescriptor {
        id: rule_id.to_string(),
        name: Some(rule.name().to_string()),
        short_description: None,
        full_description: None,
        help_uri: Some(format!(
            "https://oxc.rs/docs/guide/usage/linter/rules/{}/{}.html",
            rule.plugin_name(),
            rule.name()
        )),
        properties: Some(SarifRuleProperties {
            category: rule.category().as_str().to_string(),
            plugin: rule.plugin_name().to_string(),
            fix: rule.fix().to_string(),
        }),
    }
}

fn synthetic_artifact_rule_descriptor() -> SarifReportingDescriptor {
    SarifReportingDescriptor {
        id: SYNTHETIC_ARTIFACT_RULE_ID.to_string(),
        name: Some("oxlint-diagnostic".to_string()),
        short_description: Some(sarif_message("Oxlint diagnostic")),
        full_description: Some(sarif_message(
            "An oxlint diagnostic that is associated with an artifact but not with a lint rule.",
        )),
        help_uri: None,
        properties: None,
    }
}

fn configuration_notification_descriptor() -> SarifReportingDescriptor {
    SarifReportingDescriptor {
        id: CONFIGURATION_NOTIFICATION_ID.to_string(),
        name: Some("oxlint-configuration".to_string()),
        short_description: Some(sarif_message("Oxlint configuration or execution diagnostic")),
        full_description: Some(sarif_message(
            "An oxlint diagnostic that is not associated with a specific artifact.",
        )),
        help_uri: None,
        properties: None,
    }
}

fn region_from_positions(start: &InfoPosition, end: &InfoPosition) -> Option<SarifRegion> {
    if start.line == 0 {
        return None;
    }

    Some(SarifRegion {
        start_line: start.line,
        start_column: nonzero(start.column),
        end_line: nonzero(end.line),
        end_column: nonzero(end.column),
    })
}

fn normalize_uri(filename: &str) -> String {
    filename.cow_replace('\\', "/").into_owned()
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Advice => "note",
    }
}

fn sarif_message(text: impl Into<String>) -> SarifMessage {
    SarifMessage { text: text.into() }
}

fn nonzero(value: usize) -> Option<usize> {
    (value != 0).then_some(value)
}

fn format_sarif(diagnostics: &mut Vec<Error>) -> String {
    let mut builder = SarifBuilder::new();
    for diagnostic in diagnostics.drain(..) {
        builder.add_diagnostic(&diagnostic);
    }

    serde_json::to_string_pretty(&builder.finish()).expect("Failed to serialize")
}

#[cfg(test)]
mod test {
    use oxc_diagnostics::{Error, NamedSource, OxcDiagnostic, Severity};
    use oxc_span::Span;

    use super::format_sarif;

    fn render(diagnostics: Vec<Error>) -> serde_json::Value {
        let mut diagnostics = diagnostics;
        serde_json::from_str(&format_sarif(&mut diagnostics)).unwrap()
    }

    #[test]
    fn empty_run_shape() {
        let output = render(Vec::new());
        let run = &output["runs"][0];

        assert_eq!(output["version"], "2.1.0");
        assert_eq!(
            output["$schema"],
            "https://docs.oasis-open.org/sarif/sarif/v2.1.0/errata01/os/schemas/sarif-schema-2.1.0.json"
        );
        assert_eq!(run["tool"]["driver"]["name"], "oxlint");
        assert!(run["tool"]["driver"]["version"].is_string());
        assert_eq!(run["tool"]["driver"]["rules"].as_array().unwrap().len(), 0);
        assert_eq!(run["results"].as_array().unwrap().len(), 0);
        assert!(run.get("artifacts").is_none());
        assert!(run.get("invocations").is_none());
        assert!(run.get("columnKind").is_none());
    }

    #[test]
    fn lint_diagnostic_uses_rule_metadata() {
        let error = OxcDiagnostic::warn("Unexpected debugger statement")
            .with_error_code("eslint", "no-debugger")
            .with_label(Span::new(0, 8))
            .with_source_code(NamedSource::new("test.ts", "debugger;"));

        let output = render(vec![error]);
        let run = &output["runs"][0];
        let result = &run["results"][0];
        let rule = &run["tool"]["driver"]["rules"][0];
        let artifact = &run["artifacts"][0];
        let artifact_location = &result["locations"][0]["physicalLocation"]["artifactLocation"];
        let region = &result["locations"][0]["physicalLocation"]["region"];

        assert_eq!(result["ruleId"], "eslint(no-debugger)");
        assert_eq!(result["ruleIndex"], 0);
        assert_eq!(result["level"], "warning");
        assert_eq!(result["message"]["text"], "Unexpected debugger statement");
        assert!(result.get("suppressions").is_none());
        assert_eq!(rule["id"], "eslint(no-debugger)");
        assert_eq!(rule["name"], "no-debugger");
        assert_eq!(rule["properties"]["plugin"], "eslint");
        assert_eq!(
            rule["helpUri"],
            "https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-debugger.html"
        );
        assert_eq!(artifact["location"]["uri"], "test.ts");
        assert_eq!(artifact_location["uri"], "test.ts");
        assert_eq!(artifact_location["index"], 0);
        assert_eq!(region["startLine"], 1);
        assert_eq!(region["startColumn"], 1);
        assert_eq!(region["endLine"], 1);
        assert_eq!(region["endColumn"], 9);
        assert_eq!(run["columnKind"], "unicodeCodePoints");
    }

    #[test]
    fn artifact_diagnostic_without_rule_uses_synthetic_rule() {
        let error = OxcDiagnostic::error("Expected `;` but found `:`")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("parser-error.js", ":"));

        let output = render(vec![error]);
        let run = &output["runs"][0];
        let result = &run["results"][0];
        let rule = &run["tool"]["driver"]["rules"][0];

        assert_eq!(result["ruleId"], "OXL0001");
        assert_eq!(result["ruleIndex"], 0);
        assert_eq!(result["level"], "error");
        assert_eq!(result["message"]["text"], "Expected `;` but found `:`");
        assert_eq!(rule["id"], "OXL0001");
        assert!(run.get("invocations").is_none());
    }

    #[test]
    fn diagnostic_without_artifact_uses_configuration_notification() {
        let error: Error = OxcDiagnostic::error("Failed to parse configuration").into();

        let output = render(vec![error]);
        let run = &output["runs"][0];
        let notification = &run["invocations"][0]["toolConfigurationNotifications"][0];
        let descriptor = &run["tool"]["driver"]["notifications"][0];

        assert_eq!(run["results"].as_array().unwrap().len(), 0);
        assert_eq!(run["invocations"][0]["executionSuccessful"], false);
        assert_eq!(notification["descriptor"]["id"], "OXL0999");
        assert_eq!(notification["descriptor"]["index"], 0);
        assert_eq!(notification["level"], "error");
        assert_eq!(notification["message"]["text"], "Failed to parse configuration");
        assert!(notification.get("locations").is_none());
        assert_eq!(descriptor["id"], "OXL0999");
    }

    #[test]
    fn advice_maps_to_note() {
        let error = OxcDiagnostic::warn("Consider simplifying this expression")
            .with_severity(Severity::Advice)
            .with_error_code("oxc", "test-advice")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("test.js", "x"));

        let output = render(vec![error]);
        let result = &output["runs"][0]["results"][0];

        assert_eq!(result["ruleId"], "oxc(test-advice)");
        assert!(result.get("ruleIndex").is_none());
        assert_eq!(result["level"], "note");
    }
}
