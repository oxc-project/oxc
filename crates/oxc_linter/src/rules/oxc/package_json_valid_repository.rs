use super::json_utils::is_json_file;

use lazy_regex::{Lazy, Regex, lazy_regex};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonObject, JsonValue, parse_json},
    rule::Rule,
};

static OWNER_REPOSITORY_REGEX: Lazy<Regex> = lazy_regex!(r"^[^/\s]+/[^/\s]+$");
static PROVIDER_SHORTHAND_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:bitbucket|gist|github|gitlab):\S+$");
static URL_REPOSITORY_REGEX: Lazy<Regex> =
    lazy_regex!(r"^(?:git\+)?(?:https?|ssh|git)://[^/\s]+/\S+$");
static SCP_REPOSITORY_REGEX: Lazy<Regex> = lazy_regex!(r"^git@[^:\s]+:\S+$");

fn invalid_package_json_repository_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `repository` field in package.json is invalid.")
        .with_help("Use a valid repository shorthand, URL, or `{ type, url }` object.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidRepository;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `repository` field in package.json files.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid repository metadata breaks package discoverability and can
    /// confuse tooling that links a package back to its source.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "repository": "github:" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "repository": "github:npm/cli" }
    /// ```
    PackageJsonValidRepository,
    oxc,
    correctness
);

impl Rule for PackageJsonValidRepository {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };
        let Some(prop) = object.get_property("repository") else {
            return;
        };

        if is_valid_repository_value(&prop.value) {
            return;
        }

        ctx.diagnostic(invalid_package_json_repository_diagnostic(prop.value.span()));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn is_valid_repository_value(value: &JsonValue<'_>) -> bool {
    match value {
        JsonValue::String(value, _) => is_valid_repository_locator(value),
        JsonValue::Object(object) => is_valid_repository_object(object),
        _ => false,
    }
}

fn is_valid_repository_object(object: &JsonObject<'_>) -> bool {
    let type_is_valid =
        matches!(object.get("type"), Some(JsonValue::String(value, _)) if !value.trim().is_empty());
    let url_is_valid = matches!(object.get("url"), Some(JsonValue::String(value, _)) if is_valid_repository_locator(value));
    let directory_is_valid = match object.get("directory") {
        None => true,
        Some(JsonValue::String(value, _)) => !value.trim().is_empty(),
        Some(_) => false,
    };

    type_is_valid && url_is_valid && directory_is_valid
}

fn is_valid_repository_locator(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && (OWNER_REPOSITORY_REGEX.is_match(trimmed)
            || PROVIDER_SHORTHAND_REGEX.is_match(trimmed)
            || URL_REPOSITORY_REGEX.is_match(trimmed)
            || SCP_REPOSITORY_REGEX.is_match(trimmed))
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"repository":"npm/example"}"#,
        r#"{"repository":"github:npm/example"}"#,
        r#"{"repository":"gist:11081aaa281"}"#,
        r#"{"repository":"git+https://github.com/npm/example.git"}"#,
        r#"{"repository":"git@github.com:npm/example.git"}"#,
        r#"{"repository":{"type":"git","url":"https://github.com/npm/example"}}"#,
        r#"{"repository":{"type":"git","url":"https://github.com/npm/example","directory":"packages/core"}}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"repository":1}"#,
        r#"{"repository":""}"#,
        r#"{"repository":"github:"}"#,
        r#"{"repository":"https://github.com"}"#,
        r#"{"repository":{"type":"","url":"https://github.com/npm/example"}}"#,
        r#"{"repository":{"type":"git","url":1}}"#,
        r#"{"repository":{"type":"git","url":"https://github.com/npm/example","directory":""}}"#,
    ];

    Tester::new(PackageJsonValidRepository::NAME, PackageJsonValidRepository::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
