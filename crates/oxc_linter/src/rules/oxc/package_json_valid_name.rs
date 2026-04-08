use super::json_utils::is_json_file;

use cow_utils::CowUtils;
use nodejs_built_in_modules::is_nodejs_builtin_module;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn invalid_package_json_name_diagnostic(complaints: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Invalid npm package name: {complaints}."))
        .with_help("Use a valid npm package name, or mark the package as private if it is not meant to be published.")
        .with_label(span)
}

fn non_string_package_json_name_diagnostic(span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("The `name` field in package.json must be a string.")
        .with_help("Use a valid npm package name string.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonValidName;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Validates the `name` field in package.json files as an npm package name.
    ///
    /// ### Why is this bad?
    ///
    /// Invalid package names break npm metadata expectations and publishing
    /// workflows.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// { "name": "HTTP" }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// { "name": "@scope/demo-package" }
    /// ```
    PackageJsonValidName,
    oxc,
    correctness
);

impl Rule for PackageJsonValidName {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(object)) = &result.root else {
            return;
        };

        if is_private_package(object) {
            return;
        }

        let Some(name_prop) = object.get_property("name") else {
            return;
        };

        let JsonValue::String(name, _) = &name_prop.value else {
            ctx.diagnostic(non_string_package_json_name_diagnostic(name_prop.value.span()));
            return;
        };

        let complaints = validate_package_name(name);
        if complaints.is_empty() {
            return;
        }

        ctx.diagnostic(invalid_package_json_name_diagnostic(
            &complaints.join("; "),
            name_prop.value.span(),
        ));
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

fn is_private_package(object: &crate::json_parser::JsonObject<'_>) -> bool {
    matches!(object.get("private"), Some(JsonValue::Boolean(true, _)))
}

fn validate_package_name(name: &str) -> Vec<String> {
    let mut complaints = Vec::new();

    if name.is_empty() {
        complaints.push("name length must be greater than zero".to_string());
    }

    if name.starts_with('.') {
        complaints.push("name cannot start with a period".to_string());
    }

    if name.starts_with('-') {
        complaints.push("name cannot start with a hyphen".to_string());
    }

    if name.starts_with('_') {
        complaints.push("name cannot start with an underscore".to_string());
    }

    if name.trim() != name {
        complaints.push("name cannot contain leading or trailing spaces".to_string());
    }

    let lowercase_name = name.cow_to_ascii_lowercase();
    if matches!(&*lowercase_name, "node_modules" | "favicon.ico") {
        complaints.push(format!("{lowercase_name} is not a valid package name"));
    }

    if is_nodejs_builtin_module(&lowercase_name) {
        complaints.push(format!("{name} is a core module name"));
    }

    if name.len() > 214 {
        complaints.push("name can no longer contain more than 214 characters".to_string());
    }

    if name.cow_to_ascii_lowercase() != name {
        complaints.push("name can no longer contain capital letters".to_string());
    }

    let leaf_name = name.rsplit('/').next().unwrap_or(name);
    if leaf_name.contains(['~', '\'', '!', '(', ')', '*']) {
        complaints.push("name can no longer contain special characters (\"~'!()*\")".to_string());
    }

    if let Some(complaint) = url_friendly_name_complaint(name) {
        complaints.push(complaint);
    }

    complaints
}

fn url_friendly_name_complaint(name: &str) -> Option<String> {
    if is_url_friendly_segment(name) {
        return None;
    }

    let Some(scoped_name) = name.strip_prefix('@') else {
        return Some("name can only contain URL-friendly characters".to_string());
    };
    let Some((scope, package)) = scoped_name.split_once('/') else {
        return Some("name can only contain URL-friendly characters".to_string());
    };

    if scope.is_empty() || package.is_empty() || package.contains('/') {
        return Some("name can only contain URL-friendly characters".to_string());
    }

    if package.starts_with('.') {
        return Some("name cannot start with a period".to_string());
    }

    if is_url_friendly_segment(scope) && is_url_friendly_segment(package) {
        None
    } else {
        Some("name can only contain URL-friendly characters".to_string())
    }
}

fn is_url_friendly_segment(segment: &str) -> bool {
    !segment.is_empty()
        && !segment.contains('/')
        && segment.chars().all(|character| {
            character.is_ascii_alphanumeric()
                || matches!(character, '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')')
        })
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"name":"demo"}"#,
        r#"{"name":"demo-package"}"#,
        r#"{"name":"demo.package_name"}"#,
        r#"{"name":"@scope/demo"}"#,
        r#"{"name":"HTTP","private":true}"#,
        r#"{"version":"1.0.0"}"#,
    ];

    let fail = vec![
        r#"{"name":1}"#,
        // string "true" is NOT treated as private — only boolean true skips validation
        r#"{"name":"HTTP","private":"true"}"#,
        r#"{"name":""}"#,
        r#"{"name":"HTTP"}"#,
        r#"{"name":"node_modules"}"#,
        r#"{"name":" demo"}"#,
        r#"{"name":"demo!"}"#,
        r#"{"name":"demo package"}"#,
        r#"{"name":"_demo"}"#,
        r#"{"name":"@scope/.demo"}"#,
    ];

    Tester::new(PackageJsonValidName::NAME, PackageJsonValidName::PLUGIN, pass, fail)
        .change_rule_path("package.json")
        .test_and_snapshot();
}
