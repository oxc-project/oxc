use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{CompactStr, Span};
use rustc_hash::FxHashMap;
use serde::Deserialize;

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn no_restricted_imports_diagnostic(
    ctx: &LintContext,
    span: Span,
    message: Option<String>,
    source: &str,
) {
    let msg =
        message.unwrap_or_else(|| format!("'{source}' import is restricted from being used."));
    ctx.diagnostic(
        OxcDiagnostic::warn(msg).with_help("Remove the import statement.").with_label(span),
    );
}

#[derive(Debug, Default, Clone)]
pub struct NoRestrictedImports {
    paths: Vec<RestrictedPath>,
}

#[derive(Debug, Clone, Deserialize)]
struct RestrictedPath {
    name: String,
    #[serde(rename = "importNames")]
    import_names: Option<Vec<String>>,
    message: Option<String>,
}

declare_oxc_lint!(
    /// ### What it does
    /// This rule allows you to specify imports that you don’t want to use in your application.
    /// It applies to static imports only, not dynamic ones.
    ///
    /// ### Why is this bad?
    ///Some imports might not make sense in a particular environment. For example, Node.js’ fs module would not make sense in an environment that didn’t have a file system.
    ///
    /// Some modules provide similar or identical functionality, think lodash and underscore. Your project may have standardized on a module. You want to make sure that the other alternatives are not being used as this would unnecessarily bloat the project and provide a higher maintenance cost of two dependencies when one would suffice.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", {
    ///     "name": "disallowed-import",
    ///     "message": "Please use 'allowed-import' instead"
    /// }]*/
    ///
    /// import foo from 'disallowed-import';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// /*eslint no-restricted-imports: ["error", {"name": "fs"}]*/
    ///
    /// import crypto from 'crypto';
    /// export { foo } from "bar";
    /// ```
    NoRestrictedImports,
    style,
);

impl Rule for NoRestrictedImports {
    fn from_configuration(value: serde_json::Value) -> Self {
        let mut paths = Vec::new();

        if let Some(obj) = value.as_object() {
            if let Some(paths_value) = obj.get("paths") {
                if let Some(paths_array) = paths_value.as_array() {
                    for path_value in paths_array {
                        if let Ok(path) = serde_json::from_value(path_value.clone()) {
                            paths.push(path);
                        }
                    }
                }
            }
        }

        Self { paths }
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let mut side_effect_import_map: FxHashMap<&CompactStr, Vec<Span>> = FxHashMap::default();

        for path in &self.paths {
            for entry in &module_record.import_entries {
                let source = entry.module_request.name();
                let span = entry.module_request.span();

                if source == path.name.as_str() {
                    if let Some(import_names) = &path.import_names {
                        match &entry.import_name {
                            ImportImportName::Name(import) => {
                                if !import_names.contains(&import.name().to_string()) {
                                    no_restricted_imports_diagnostic(
                                        ctx,
                                        span,
                                        path.message.clone(),
                                        source,
                                    );
                                    return;
                                }
                            }
                            ImportImportName::Default(_) | ImportImportName::NamespaceObject => {
                                if !import_names.contains(&entry.local_name.name().to_string()) {
                                    no_restricted_imports_diagnostic(
                                        ctx,
                                        span,
                                        path.message.clone(),
                                        source,
                                    );
                                    return;
                                }
                            }
                        }
                    } else {
                        no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                    }
                }
            }

            for (source, requests) in &module_record.requested_modules {
                for request in requests {
                    if request.is_import && module_record.import_entries.is_empty() {
                        side_effect_import_map.entry(source).or_default().push(request.span);
                    }
                }
            }

            for (source, spans) in &side_effect_import_map {
                if source.as_str() == path.name.as_str() {
                    if let Some(span) = spans.iter().next() {
                        no_restricted_imports_diagnostic(ctx, *span, path.message.clone(), source);
                    }
                    return;
                }
            }

            for entry in &module_record.local_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;

                    if source == path.name.as_str() {
                        no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                        return;
                    }
                }
            }
            for entry in &module_record.indirect_export_entries {
                if let Some(module_request) = &entry.module_request {
                    let source = module_request.name();
                    let span = entry.span;

                    if source == path.name.as_str() {
                        no_restricted_imports_diagnostic(ctx, span, path.message.clone(), source);
                        return;
                    }
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // Basic cases - no matches
        (
            r#"import os from "os";"#,
            Some(serde_json::json!({
                "paths": [{ "name": "fs" }]
            })),
        ),
        (
            r#"import fs from "fs";"#,
            Some(serde_json::json!({
                "paths": [{ "name": "crypto" }]
            })),
        ),
        (
            r#"import path from "path";"#,
            Some(serde_json::json!({
                "paths": [
                    { "name": "crypto" },
                    { "name": "stream" },
                    { "name": "os" }
                ]
            })),
        ),
        // Testing with import names
        (
            r#"import AllowedObject from "foo";"#,
            Some(serde_json::json!({
                "paths": [{
                    "name": "foo",
                    "importNames": ["AllowedObject"]
                }]
            })),
        ),
        // Testing relative paths
        (
            "import relative from '../foo';",
            Some(serde_json::json!({
                "paths": [{ "name": "../notFoo" }]
            })),
        ),
        // Multiple restricted imports
        (
            r#"import { DisallowedObjectOne, DisallowedObjectTwo } from "foo";"#,
            Some(serde_json::json!({
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObjectOne", "DisallowedObjectTwo"],
                }]
            })),
        ),
    ];

    let fail = vec![
        // Basic restrictions
        (
            r#"import "fs""#,
            Some(serde_json::json!({
                "paths": [{ "name": "fs" }]
            })),
        ),
        // With custom message
        (
            r#"import withGitignores from "foo";"#,
            Some(serde_json::json!({
                "paths": [{
                    "name": "foo",
                    "message": "Please import from 'bar' instead."
                }]
            })),
        ),
        // Restricting default import
        (
            r#"import DisallowedObject from "foo";"#,
            Some(serde_json::json!({
                "paths": [{
                    "name": "foo",
                    "importNames": ["default"],
                    "message": "Please import the default import of 'foo' from /bar/ instead."
                }]
            })),
        ),
        // Namespace imports
        (
            r#"import * as All from "foo";"#,
            Some(serde_json::json!({
                "paths": [{
                    "name": "foo",
                    "importNames": ["DisallowedObject"],
                    "message": "Please import 'DisallowedObject' from /bar/ instead."
                }]
            })),
        ),
        // Export restrictions
        (
            r#"export { something } from "fs";"#,
            Some(serde_json::json!({
                "paths": [{ "name": "fs" }]
            })),
        ),
        // Complex case with multiple restrictions
        (
            r#"import { foo, bar, baz } from "mod""#,
            Some(serde_json::json!({
                "paths": [
                    {
                        "name": "mod",
                        "importNames": ["foo"],
                        "message": "Import foo from qux instead."
                    },
                    {
                        "name": "mod",
                        "importNames": ["baz"],
                        "message": "Import baz from qux instead."
                    }
                ]
            })),
        ),
    ];

    Tester::new(NoRestrictedImports::NAME, NoRestrictedImports::CATEGORY, pass, fail)
        .test_and_snapshot();
}
