use super::json_utils::is_json_file;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    json_parser::{JsonValue, parse_json},
    rule::Rule,
};

fn duplicate_dependency_diagnostic(name: &str, span: oxc_span::Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("`{name}` is listed in both `dependencies` and `devDependencies`."))
        .with_help("Remove the package from one of the dependency groups.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PackageJsonNoDuplicateDependencies;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Detects packages listed in both `dependencies` and `devDependencies`.
    ///
    /// ### Why is this bad?
    ///
    /// Listing a package in both groups is redundant and confusing. npm
    /// ignores `devDependencies` entries that overlap `dependencies`, but the
    /// intent becomes unclear and can mask version conflicts.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```json
    /// {
    ///   "dependencies": { "lodash": "^4.17.0" },
    ///   "devDependencies": { "lodash": "^4.17.0" }
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```json
    /// {
    ///   "dependencies": { "lodash": "^4.17.0" },
    ///   "devDependencies": { "vitest": "^1.0.0" }
    /// }
    /// ```
    PackageJsonNoDuplicateDependencies,
    oxc,
    correctness
);

impl Rule for PackageJsonNoDuplicateDependencies {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let source_text = ctx.full_source_text();
        let result = parse_json(source_text);
        let Some(JsonValue::Object(root)) = &result.root else {
            return;
        };

        let Some(deps) = root.get("dependencies") else {
            return;
        };
        let JsonValue::Object(deps) = deps else {
            return;
        };

        let Some(dev_deps_prop) = root.get_property("devDependencies") else {
            return;
        };
        let JsonValue::Object(dev_deps) = &dev_deps_prop.value else {
            return;
        };

        for entry in &dev_deps.properties {
            if deps.get_property(entry.key).is_some() {
                ctx.diagnostic(duplicate_dependency_diagnostic(entry.key, entry.key_span));
            }
        }
    }

    fn should_run(&self, ctx: &crate::rules::ContextHost) -> bool {
        ctx.is_first_sub_host()
            && is_json_file(ctx.file_path())
            && ctx.file_path().file_name().is_some_and(|name| name == "package.json")
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"{"dependencies":{"lodash":"^4.17.0"},"devDependencies":{"vitest":"^1.0.0"}}"#,
        r#"{"dependencies":{"lodash":"^4.17.0"}}"#,
        r#"{"devDependencies":{"vitest":"^1.0.0"}}"#,
        r#"{"name":"demo"}"#,
    ];

    let fail = vec![
        r#"{"dependencies":{"lodash":"^4.17.0"},"devDependencies":{"lodash":"^4.17.0"}}"#,
        r#"{"dependencies":{"lodash":"^4.17.0","react":"^18.0.0"},"devDependencies":{"lodash":"^4.17.0","vitest":"^1.0.0"}}"#,
    ];

    Tester::new(
        PackageJsonNoDuplicateDependencies::NAME,
        PackageJsonNoDuplicateDependencies::PLUGIN,
        pass,
        fail,
    )
    .change_rule_path("package.json")
    .test_and_snapshot();
}
