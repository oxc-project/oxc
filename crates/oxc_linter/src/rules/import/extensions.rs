use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use serde_json::Value;

use crate::{
    context::LintContext,
    module_record::{ExportEntry, ImportEntry, NameSpan},
    rule::Rule,
};

fn extension_should_not_be_included_in_import_diagnostic(
    span: Span,
    extension: &str,
) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(format!(
        "File extension {extension} should not be included in the import declaration."
    ))
    .with_help("Remove the file extension from this import.")
    .with_label(span)
}

fn extension_missing_from_import_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing file extension in import declaration")
        .with_help("Add a file extension to this import.")
        .with_label(span)
}

fn extension_should_not_be_included_in_export_diagnostic(
    span: Span,
    extension: &str,
) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn(format!(
        "File extension {extension} should not be included in the export declaration."
    ))
    .with_help("Remove the file extension from this export.")
    .with_label(span)
}

fn extension_missing_from_export_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Missing file extension in export declaration")
        .with_help("Add a file extension to this export.")
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq)]
enum FileExtensionConfig {
    Always,
    #[default]
    Never,
}

impl FileExtensionConfig {
    pub fn from(str: &str) -> FileExtensionConfig {
        match str {
            "always" | "ignorePackages" => FileExtensionConfig::Always,
            "never" => FileExtensionConfig::Never,
            _ => FileExtensionConfig::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExtensionsConfig {
    ignore_packages: bool,
    require_extension: Option<FileExtensionConfig>,
    check_type_imports: bool,
    js: FileExtensionConfig,
    jsx: FileExtensionConfig,
    ts: FileExtensionConfig,
    tsx: FileExtensionConfig,
    json: FileExtensionConfig,
}

impl ExtensionsConfig {
    pub fn get_always_file_types(&self) -> Vec<&str> {
        let mut result: Vec<&str> = vec![];

        if matches!(self.js, FileExtensionConfig::Always) {
            result.push("js");
        }

        if matches!(self.jsx, FileExtensionConfig::Always) {
            result.push("jsx");
        }

        if matches!(self.ts, FileExtensionConfig::Always) {
            result.push("ts");
        }

        if matches!(self.tsx, FileExtensionConfig::Always) {
            result.push("tsx");
        }

        if matches!(self.json, FileExtensionConfig::Always) {
            result.push("json");
        }

        result
    }
    pub fn get_never_file_types(&self) -> Vec<&str> {
        let mut result: Vec<&str> = vec![];

        if matches!(self.js, FileExtensionConfig::Never) {
            result.push("js");
        }

        if matches!(self.jsx, FileExtensionConfig::Never) {
            result.push("jsx");
        }

        if matches!(self.ts, FileExtensionConfig::Never) {
            result.push("ts");
        }

        if matches!(self.tsx, FileExtensionConfig::Never) {
            result.push("tsx");
        }

        if matches!(self.json, FileExtensionConfig::Never) {
            result.push("json");
        }

        result
    }
}

#[derive(Debug, Default, Clone)]
pub struct Extensions(Box<ExtensionsConfig>);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Some file resolve algorithms allow you to omit the file extension within the import source path.
    /// For example the node resolver (which does not yet support ESM/import) can resolve ./foo/bar to the absolute path /User/someone/foo/bar.js because the .js extension is resolved automatically by default in CJS.
    /// Depending on the resolver you can configure more extensions to get resolved automatically.
    /// In order to provide a consistent use of file extensions across your code base, this rule can enforce or disallow the use of certain file extensions.
    ///
    /// ### Why is this bad?
    ///
    /// ESM-based file resolve algorithms (e.g., the one that Vite provides) recommend specifying the file extension to improve performance.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    ///
    /// The following patterns are considered problems when configuration set to "always":
    /// ```js
    /// import foo from './foo';
    /// import bar from './bar';
    /// import Component from './Component';
    /// import foo from '@/foo';
    /// ```
    ///
    /// The following patterns are considered problems when configuration set to "never":
    /// ```js
    /// import foo from './foo.js';
    /// import bar from './bar.json';
    /// import Component from './Component.jsx';
    /// import express from 'express/index.js';
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    ///
    /// The following patterns are not considered problems when configuration set to "always":
    ///
    /// ```js
    /// import foo from './foo.js';
    /// import bar from './bar.json';
    /// import Component from './Component.jsx';
    /// import * as path from 'path';
    /// import foo from '@/foo.js';
    /// ```
    ///
    /// The following patterns are not considered problems when configuration set to "never":
    /// ```js
    /// import foo from './foo';
    /// import bar from './bar';
    /// import Component from './Component';
    /// import express from 'express/index';
    /// import * as path from 'path';
    /// ```
    Extensions,
    import,
    perf,
);

pub const BUILT_IN_NODE_MODULES: phf::Set<&'static str> = phf::phf_set![
    "node:assert",
    "assert",
    "node:assert/strict",
    "assert/strict",
    "node:async_hooks",
    "async_hooks",
    "node:buffer",
    "buffer",
    "node:child_process",
    "child_process",
    "node:cluster",
    "cluster",
    "node:console",
    "console",
    "node:constants",
    "constants",
    "node:crypto",
    "crypto",
    "node:dgram",
    "dgram",
    "node:diagnostics_channel",
    "diagnostics_channel",
    "node:dns",
    "dns",
    "node:dns/promises",
    "dns/promises",
    "node:domain",
    "domain",
    "node:events",
    "events",
    "node:fs",
    "fs",
    "node:fs/promises",
    "fs/promises",
    "node:http",
    "http",
    "node:http2",
    "http2",
    "node:https",
    "https",
    "node:inspector",
    "inspector",
    "node:inspector/promises",
    "inspector/promises",
    "node:module",
    "module",
    "node:net",
    "net",
    "node:os",
    "os",
    "node:path",
    "path",
    "node:path/posix",
    "path/posix",
    "node:path/win32",
    "path/win32",
    "node:perf_hooks",
    "perf_hooks",
    "node:process",
    "process",
    "node:querystring",
    "querystring",
    "node:quic",
    "node:readline",
    "readline",
    "node:readline/promises",
    "readline/promises",
    "node:repl",
    "repl",
    "node:sea",
    "node:sqlite",
    "node:stream",
    "stream",
    "node:stream/consumers",
    "stream/consumers",
    "node:stream/promises",
    "stream/promises",
    "node:stream/web",
    "stream/web",
    "node:string_decoder",
    "string_decoder",
    "node:test",
    "node:test/reporters",
    "node:timers",
    "timers",
    "node:timers/promises",
    "timers/promises",
    "node:tls",
    "tls",
    "node:trace_events",
    "trace_events",
    "node:tty",
    "tty",
    "node:url",
    "url",
    "node:util",
    "util",
    "node:util/types",
    "util/types",
    "node:v8",
    "v8",
    "node:vm",
    "vm",
    "node:wasi",
    "wasi",
    "node:worker_threads",
    "worker_threads",
    "node:zlib",
    "zlib"
];

fn process_import_record(
    import: &ImportEntry,
    config: &ExtensionsConfig,
    ctx: &LintContext,
    extensions: &Extensions,
) {
    let always_file_types = extensions.0.get_always_file_types();
    let never_file_types = extensions.0.get_never_file_types();

    if import.is_type && !config.check_type_imports {
        return;
    }

    let import_name = import.module_request.name();

    let is_builtin_node_module =
        BUILT_IN_NODE_MODULES.contains(import_name) || ctx.globals().is_enabled(import_name);

    let is_package = import_name.starts_with('@')
        || (!import_name.starts_with('.') && !import_name[1..].contains('/'));

    if is_builtin_node_module || (is_package && config.ignore_packages) {
        return;
    }

    let file_extension = get_file_extension_from_module_request(&import.module_request);

    let span = import.statement_span;
    if let Some(file_extension) = file_extension {
        if never_file_types.contains(&file_extension)
            || (!always_file_types.is_empty() && !always_file_types.contains(&file_extension))
        // should not have file extension
        {
            ctx.diagnostic(extension_should_not_be_included_in_import_diagnostic(
                span,
                file_extension,
            ));

            if file_extension.is_empty()
                && config.require_extension == Some(FileExtensionConfig::Always)
            {
                ctx.diagnostic(extension_missing_from_import_diagnostic(span));
            }
        }
    } else if config.require_extension == Some(FileExtensionConfig::Always) {
g        ctx.diagnostic(extension_missing_from_import_diagnostic(span));
    }
}

fn process_export_record(
    export: &ExportEntry,
    config: &ExtensionsConfig,
    ctx: &LintContext,
    extensions: &Extensions,
) {
    let always_file_types = extensions.0.get_always_file_types();
    let never_file_types = extensions.0.get_never_file_types();

    if export.module_request.is_none() {
        return;
    }

    if export.is_type && !config.check_type_imports {
        return;
    }

    let export_module_request = export.module_request.as_ref().unwrap();

    let export_name = export_module_request.name();

    let is_builtin_node_module =
        BUILT_IN_NODE_MODULES.contains(export_name) || ctx.globals().is_enabled(export_name);

    let is_package = export_name.starts_with('@')
        || (!export_name.starts_with('.') && !export_name[1..].contains('/'));

    if is_builtin_node_module || (is_package && config.ignore_packages) {
        return;
    }

    let file_extension = get_file_extension_from_module_request(export_module_request);

    let span = export.statement_span;

    if let Some(file_extension) = file_extension {
        if never_file_types.contains(&file_extension)
            || (!always_file_types.is_empty() && !always_file_types.contains(&file_extension))
        // should not have file extension
        {
            ctx.diagnostic(extension_should_not_be_included_in_export_diagnostic(
                span,
                file_extension,
            ));

            if file_extension.is_empty()
                && config.require_extension == Some(FileExtensionConfig::Always)
            {
                ctx.diagnostic(extension_missing_from_export_diagnostic(span));
            }
        }
    } else if config.require_extension == Some(FileExtensionConfig::Always) {
        ctx.diagnostic(extension_missing_from_export_diagnostic(span));
    }
}

impl Rule for Extensions {
    fn from_configuration(value: serde_json::Value) -> Self {
        if let Some(always_or_never) =
            value.get(0).and_then(Value::as_str).map(FileExtensionConfig::from)
        {
            let default = always_or_never;

            if let Some(val) = value.get(1) {
                let root = val.get("pattern").unwrap_or(val);

                let config: ExtensionsConfig = ExtensionsConfig {
                    ignore_packages: root
                        .get("ignorePackages")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                    require_extension: Some(default.clone()),
                    check_type_imports: root
                        .get("checkTypeImports")
                        .and_then(Value::as_bool)
                        .unwrap_or_default(),
                    js: root
                        .get("js")
                        .and_then(Value::as_str)
                        .map(FileExtensionConfig::from)
                        .unwrap_or(default.clone()),
                    jsx: root
                        .get("jsx")
                        .and_then(Value::as_str)
                        .map(FileExtensionConfig::from)
                        .unwrap_or(default.clone()),
                    ts: root
                        .get("ts")
                        .and_then(Value::as_str)
                        .map(FileExtensionConfig::from)
                        .unwrap_or(default.clone()),
                    tsx: root
                        .get("tsx")
                        .and_then(Value::as_str)
                        .map(FileExtensionConfig::from)
                        .unwrap_or(default.clone()),
                    json: root
                        .get("json")
                        .and_then(Value::as_str)
                        .map(FileExtensionConfig::from)
                        .unwrap_or(default),
                };

                Self(Box::new(config))
            } else {
                let config: ExtensionsConfig = ExtensionsConfig {
                    ignore_packages: value
                        .get("ignorePackages")
                        .and_then(Value::as_bool)
                        .unwrap_or(true),
                    check_type_imports: value
                        .get("checkTypeImports")
                        .and_then(Value::as_bool)
                        .unwrap_or_default(),
                    js: default.clone(),
                    jsx: default.clone(),
                    ts: default.clone(),
                    tsx: default.clone(),
                    json: default.clone(),
                    require_extension: Some(default),
                };

                Self(Box::new(config))
            }
        } else {
            let config = ExtensionsConfig {
                ignore_packages: value
                    .get("ignorePackages")
                    .and_then(Value::as_bool)
                    .unwrap_or(true),
                check_type_imports: value
                    .get("checkTypeImports")
                    .and_then(Value::as_bool)
                    .unwrap_or_default(),
                js: value
                    .get("js")
                    .and_then(Value::as_str)
                    .map(FileExtensionConfig::from)
                    .unwrap_or_default(),
                jsx: value
                    .get("jsx")
                    .and_then(Value::as_str)
                    .map(FileExtensionConfig::from)
                    .unwrap_or_default(),
                ts: value
                    .get("ts")
                    .and_then(Value::as_str)
                    .map(FileExtensionConfig::from)
                    .unwrap_or_default(),
                tsx: value
                    .get("tsx")
                    .and_then(Value::as_str)
                    .map(FileExtensionConfig::from)
                    .unwrap_or_default(),
                json: value
                    .get("json")
                    .and_then(Value::as_str)
                    .map(FileExtensionConfig::from)
                    .unwrap_or_default(),
                require_extension: None,
            };

            Self(Box::new(config))
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();

        let config = self.0.clone();

        for import in &module_record.import_entries {
            process_import_record(import, &config, ctx, self);
        }

        for export in &module_record.indirect_export_entries {
            process_export_record(export, &config, ctx, self);
        }

        for export in &module_record.star_export_entries {
            process_export_record(export, &config, ctx, self);
        }
    }
}

fn get_file_extension_from_module_request(module_request: &NameSpan) -> Option<&str> {
    if let Some((_, extension)) = module_request.name().rsplit_once('.') {
        if !extension.starts_with('/') {
            return extension.split('?').next();
        }
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::{Value, json};

    // let pass: Vec<(&str, Option<Value>)> = vec![];

    let pass: Vec<(&str, Option<Value>)> = vec![
        (r#"import a from "@/a""#, None),
        (r#"import a from "a""#, None),
        (r#"import dot from "./file.with.dot""#, None),
        (r#"import a from "a/index.js""#, Some(json!(["always"]))),
        (r#"import dot from "./file.with.dot.js""#, Some(json!(["always"]))),
        (
            r#"
                import a from "a";
                import packageConfig from "./package.json";
            "#,
            Some(json!({"json": "always", "js": "never"})),
        ),
        (
            r#"
                import lib from "./bar";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!(["never", { "jsx": "always", "json": "always"}])),
        ),
        // TODO: This test fails because of the presence of the .hbs file extension. In the original test from eslint-plugin-import, they apply the hbs file extension by passing settings to a linter that is constructed on the fly for test runs. Since oxc does not have a similar mechanism, I'm commenting out this test for now.

        // Link to eslint-plugin-import extensions rule unit tests:
        // https://github.com/import-js/eslint-plugin-import/blob/main/tests/src/rules/extensions.js
        // (
        //     r#"
        //         import bar from "./bar";
        //         import barjson from "./bar.json";
        //         import barhbs from "./bar.hbs";
        //     "#,
        //     Some(json!(["always", { "js": "never", "jsx": "never"}])),
        // ),
        (
            r#"
                import bar from "./bar.js";
                import pack from "./package";
            "#,
            Some(json!(["never", { "js": "always", "json": "never"}])),
        ),
        (r#"import path from "path";"#, None),
        (r#"import path from "path";"#, Some(json!(["never"]))),
        (r#"import path from "path";"#, Some(json!(["always"]))),
        (r#"import thing from "./fake-file.js";"#, Some(json!(["always"]))),
        (r#"import thing from "non-package";"#, Some(json!(["never"]))),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
                import express from "express";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        (
            r#"
                import foo from "./foo.js";
                import bar from "./bar.json";
                import Component from "./Component.jsx";
                import express from "express";
            "#,
            Some(json!(["always", { "ignorePackages": true}])),
        ),
        (
            r#"
                import foo from "./foo";
                import bar from "./bar";
                import Component from "./Component";
                import express from "express";
            "#,
            Some(json!(["never", { "ignorePackages": true}])),
        ),
        (
            r#"import exceljs from "exceljs""#,
            Some(json!(["always", { "js": "never", "jsx": "never"}])),
        ),
        (
            r#"
                export { foo } from "./foo.js";
                let bar; export { bar };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo";
                let bar; export { bar };
            "#,
            Some(json!(["never"])),
        ),
        // Root packages should be ignored and they are names not files
        (
            r#"
                import lib from "pkg.js";
                import lib2 from "pgk/package";
                import lib3 from "@name/pkg.js";
            "#,
            Some(json!(["never"])),
        ),
        // Query strings.
        (
            r#"
                import bare from "./foo?a=True.ext";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import bare from "./foo.js?a=True";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import lib from "pkg";
                import lib2 from "pgk/package.js";
                import lib3 from "@name/pkg";
            "#,
            Some(json!(["always"])),
        ),
        // Type import tests
        (
            r#"import type T from "./typescript-declare";"#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never"}]),
            ),
        ),
        (
            r#"export type { MyType } from "./typescript-declare";"#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"
                import type { MyType } from "./typescript-declare.ts";
            "#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare.ts";
            "#,
            Some(json!(["always", {"checkTypeImports": true}])),
        ),
    ];

    let fail: Vec<(&str, Option<Value>)> = vec![
        (r#"import a from "a/index.js""#, None),
        (r#"import dot from "./file.with.dot""#, Some(json!(["always"]))),
        (
            r#"
                import a from "a/index.js";
                import packageConfig from "./package";
            "#,
            Some(json!([{ "json": "always", "js": "never"}])),
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
        ),
        // TODO: fix this test
        // (r#"import "./bar.coffee""#, Some(json!(["never", { "js": "always", "jsx": "always" }]))),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["always", { "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import barjs from ".";
                import barjs2 from "..";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["never", { "json": "always", "js": "never", "jsx": "never" }])),
        ),
        (
            r#"
                import thing from "./fake-file.js";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import thing from "non-package/test";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import thing from "@name/pkg/test";
            "#,
            Some(json!(["always", {"ignorePackages": false}])),
        ),
        (
            r#"
                import thing from "@name/pkg/test.js";
            "#,
            Some(json!(["never",{"ignorePackages": false}])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component';
                import baz from 'foo/baz';
                import baw from '@scoped/baw/import';
                import chart from '@/configs/chart';
                import express from 'express';
            ",
            Some(json!(["always", { "ignorePackages": true }])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component';
                import baz from 'foo/baz';
                import baw from '@scoped/baw/import';
                import chart from '@/configs/chart';
                import express from 'express';
            ",
            Some(json!(["ignorePackages"])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component.jsx';
                import express from 'express';
            ",
            Some(json!(["never", { "ignorePackages": true }])),
        ),
        (
            r"
                import foo from './foo.js';
                import bar from './bar.json';
                import Component from './Component.jsx';
            ",
            Some(json!(["always", { "pattern": { "jsx": "never" } }])),
        ),
        // Exports
        (
            r#"
                export { foo } from "./foo";
                let bar; export { bar };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo.js";
                let bar; export { bar };
            "#,
            Some(json!(["never"])),
        ),
        // Query strings
        (r#"import withExtension from "./foo.js?a=True";"#, Some(json!(["never"]))),
        (r#"import withoutExtension from "./foo?a=True.ext";"#, Some(json!(["always"]))),
        // Require
        // (
        //     r#"
        //         const { foo } = require("./foo");
        //         export { foo };
        //     "#,
        //     Some(json!(["always"])),
        // ),
        // (
        //     r#"
        //         const { foo } = require("./foo".js);
        //         export { foo };
        //     "#,
        //     Some(json!(["never"])),
        // ),
        (
            r#"
                import foo from "@/ImNotAScopedModule";
                import chart from "@/configs/chart";
            "#,
            Some(json!(["always",{ "ignorePackages": false }])),
        ),
        // Export { } from
        (
            r#"
                export { foo } from "./foo";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export { foo } from "./foo.js";
            "#,
            Some(json!(["never"])),
        ),
        // Export * from
        (
            r#"
                export * from "./foo";
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                export * from "./foo.js";
            "#,
            Some(json!(["never"])),
        ),
        (
            r#"
                import foo from "@/ImNotAScopedModule.js";
            "#,
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        (
            r"
                import _ from 'lodash';
                import m from '@test-scope/some-module/index.js';
                import bar from './bar';
            ",
            Some(json!(["never",{ "ignorePackages": false }])),
        ),
        // Relative imports
        (
            r#"
                import * as test from ".";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        (
            r#"
                import * as test from "..";
            "#,
            Some(json!(["ignorePackages"])),
        ),
        // Type imports
        (
            r#"
                import T from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"
                export { MyType } from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
            ),
        ),
        (
            r#"
                import type T from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare";
            "#,
            Some(
                json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
            ),
        ),
        (
            r#"
                import type { MyType } from "./typescript-declare";
            "#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
        (
            r#"
                export type { MyType } from "./typescript-declare";
            "#,
            Some(json!(["always", { "checkTypeImports": true }])),
        ),
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
