use oxc_ast::{
    AstKind,
    ast::{Argument, CallExpression, Expression},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_resolver::NODEJS_BUILTINS;
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::RequestedModule;
use schemars::JsonSchema;
use serde::Serialize;
use serde_json::Value;

use crate::{context::LintContext, rule::Rule};

fn extension_should_not_be_included_in_diagnostic(
    span: Span,
    extension: &CompactStr,
    is_import: bool,
) -> OxcDiagnostic {
    let import_or_export = if is_import { "import" } else { "export" };

    OxcDiagnostic::warn(format!(
        r#"File extension "{extension}" should not be included in the {import_or_export} declaration."#
    ))
    .with_help(format!("Remove the file extension from this {import_or_export}."))
    .with_label(span)
}

fn extension_missing_diagnostic(span: Span, is_import: bool) -> OxcDiagnostic {
    let import_or_export = if is_import { "import" } else { "export" };

    OxcDiagnostic::warn(format!("Missing file extension in {import_or_export} declaration"))
        .with_help(format!("Add a file extension to this {import_or_export}."))
        .with_label(span)
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
enum FileExtensionConfig {
    Always,
    #[default]
    Never,
    IgnorePackages,
}

impl FileExtensionConfig {
    pub fn from(s: &str) -> FileExtensionConfig {
        match s {
            "always" => FileExtensionConfig::Always,
            "never" => FileExtensionConfig::Never,
            "ignorePackages" => FileExtensionConfig::IgnorePackages,
            _ => FileExtensionConfig::default(),
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ExtensionsConfig {
    /// Whether to ignore package imports (e.g., 'react', 'lodash') when enforcing extension rules.
    ignore_packages: bool,
    /// Configuration for requiring or disallowing file extensions in import/require statements.
    require_extension: Option<FileExtensionConfig>,
    /// Whether to check type imports when enforcing extension rules.
    check_type_imports: bool,
    /// Configuration for `.js` file extensions.
    js: FileExtensionConfig,
    /// Configuration for `.jsx` file extensions.
    jsx: FileExtensionConfig,
    /// Configuration for `.ts` file extensions.
    ts: FileExtensionConfig,
    /// Configuration for `.tsx` file extensions.
    tsx: FileExtensionConfig,
    /// Configuration for `.json` file extensions.
    json: FileExtensionConfig,
}

impl ExtensionsConfig {
    pub fn is_always(&self, ext: &str) -> bool {
        match ext {
            "js" => matches!(self.js, FileExtensionConfig::Always),
            "jsx" => matches!(self.jsx, FileExtensionConfig::Always),
            "ts" => matches!(self.ts, FileExtensionConfig::Always),
            "tsx" => matches!(self.tsx, FileExtensionConfig::Always),
            "json" => matches!(self.json, FileExtensionConfig::Always),
            _ => false,
        }
    }

    pub fn is_never(&self, ext: &str) -> bool {
        match ext {
            "js" => matches!(self.js, FileExtensionConfig::Never),
            "jsx" => matches!(self.jsx, FileExtensionConfig::Never),
            "ts" => matches!(self.ts, FileExtensionConfig::Never),
            "tsx" => matches!(self.tsx, FileExtensionConfig::Never),
            "json" => matches!(self.json, FileExtensionConfig::Never),
            _ => false,
        }
    }
}

impl Default for ExtensionsConfig {
    fn default() -> Self {
        Self {
            ignore_packages: true,
            require_extension: None,
            check_type_imports: false,
            js: FileExtensionConfig::Never,
            jsx: FileExtensionConfig::Never,
            ts: FileExtensionConfig::Never,
            tsx: FileExtensionConfig::Never,
            json: FileExtensionConfig::Never,
        }
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
    restriction,
    config = ExtensionsConfig,
);

impl Rule for Extensions {
    fn from_configuration(value: serde_json::Value) -> Self {
        if let Some(first_arg) = value.get(0).and_then(Value::as_str) {
            let default = FileExtensionConfig::from(first_arg);

            if let Some(val) = value.get(1) {
                let root = val.get("pattern").unwrap_or(val);

                let config = build_config(root, Some(&default));

                Self(Box::new(config))
            } else {
                let config = build_config(&value, Some(&default));

                Self(Box::new(config))
            }
        } else {
            let config = build_config(&value, None);
            Self(Box::new(config))
        }
    }

    fn run_once(&self, ctx: &LintContext) {
        let module_record = ctx.module_record();

        let config = self.0.clone();

        for node in ctx.nodes().iter() {
            if let AstKind::CallExpression(call_expr) = node.kind() {
                let Expression::Identifier(ident) = &call_expr.callee else {
                    return;
                };
                let func_name = ident.name.as_str();
                let count = call_expr.arguments.len();

                if matches!(func_name, "require") && count > 0 {
                    self.process_require_record(call_expr, ctx, config.require_extension.as_ref());
                }
            }
        }

        for (module_name, module) in &module_record.requested_modules {
            for module_item in module {
                self.process_module_record(
                    (module_name.clone(), module_item),
                    ctx,
                    config.require_extension.as_ref(),
                    config.check_type_imports,
                    config.ignore_packages,
                    module_item.is_import,
                );
            }
        }
    }
}

fn build_config(
    value: &serde_json::Value,
    default: Option<&FileExtensionConfig>,
) -> ExtensionsConfig {
    let config: ExtensionsConfig = ExtensionsConfig {
        ignore_packages: value.get("ignorePackages").and_then(Value::as_bool).unwrap_or(true),
        require_extension: default.cloned(),
        check_type_imports: value
            .get("checkTypeImports")
            .and_then(Value::as_bool)
            .unwrap_or_default(),
        js: value
            .get("js")
            .and_then(Value::as_str)
            .map(FileExtensionConfig::from)
            .unwrap_or(default.cloned().unwrap_or_default()),
        jsx: value
            .get("jsx")
            .and_then(Value::as_str)
            .map(FileExtensionConfig::from)
            .unwrap_or(default.cloned().unwrap_or_default()),

        ts: value
            .get("ts")
            .and_then(Value::as_str)
            .map(FileExtensionConfig::from)
            .unwrap_or(default.cloned().unwrap_or_default()),

        tsx: value
            .get("tsx")
            .and_then(Value::as_str)
            .map(FileExtensionConfig::from)
            .unwrap_or(default.cloned().unwrap_or_default()),

        json: value
            .get("json")
            .and_then(Value::as_str)
            .map(FileExtensionConfig::from)
            .unwrap_or(default.cloned().unwrap_or_default()),
    };

    config
}

impl Extensions {
    fn process_module_record(
        &self,
        module_record: (CompactStr, &RequestedModule),
        ctx: &LintContext,
        require_extension: Option<&FileExtensionConfig>,
        check_type_imports: bool,
        ignore_packages: bool,
        is_import: bool,
    ) {
        let config = &self.0;
        let (module_name, module) = module_record;

        if module.is_type && !check_type_imports {
            return;
        }

        let is_builtin_node_module = NODEJS_BUILTINS.binary_search(&module_name.as_str()).is_ok()
            || ctx.globals().is_enabled(module_name.as_str());

        let is_package = is_package_import(module_name.as_str());

        if is_builtin_node_module || (is_package && ignore_packages) {
            return;
        }

        let file_extension = get_file_extension_from_module_name(&module_name);

        let span = module.statement_span;

        if let Some(file_extension) = file_extension {
            let ext_str = file_extension.as_str();
            let should_flag = match require_extension {
                Some(FileExtensionConfig::Always) => {
                    config.is_never(ext_str) || !config.is_always(ext_str)
                }
                Some(FileExtensionConfig::Never) => !config.is_always(ext_str),
                _ => config.is_never(ext_str),
            };

            if should_flag {
                ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                    span,
                    &file_extension,
                    is_import,
                ));
            }
        } else if matches!(require_extension, Some(FileExtensionConfig::Always)) {
            ctx.diagnostic(extension_missing_diagnostic(span, is_import));
        } else if matches!(require_extension, Some(FileExtensionConfig::IgnorePackages)) {
            // When using ignorePackages, check if per-extension configs override it
            // If js/ts configs are set to Never (not IgnorePackages), then missing extensions are OK
            // Otherwise, require extension for non-packages
            if matches!(config.js, FileExtensionConfig::IgnorePackages)
                || matches!(config.ts, FileExtensionConfig::IgnorePackages)
            {
                ctx.diagnostic(extension_missing_diagnostic(span, is_import));
            }
        }
    }

    fn process_require_record(
        &self,
        call_expr: &CallExpression<'_>,
        ctx: &LintContext,
        require_extension: Option<&FileExtensionConfig>,
    ) {
        let config = &self.0;
        for argument in &call_expr.arguments {
            if let Argument::StringLiteral(s) = argument {
                let file_extension = get_file_extension_from_module_name(&s.value.to_compact_str());
                let span = call_expr.span;

                if let Some(file_extension) = file_extension {
                    let ext_str = file_extension.as_str();
                    let should_flag = match require_extension {
                        Some(FileExtensionConfig::Always) => {
                            config.is_never(ext_str) || !config.is_always(ext_str)
                        }
                        Some(FileExtensionConfig::Never) => !config.is_always(ext_str),
                        _ => config.is_never(ext_str),
                    };

                    if should_flag {
                        ctx.diagnostic(extension_should_not_be_included_in_diagnostic(
                            span,
                            &file_extension,
                            true,
                        ));
                    }
                } else if matches!(require_extension, Some(FileExtensionConfig::Always)) {
                    ctx.diagnostic(extension_missing_diagnostic(span, true));
                } else if matches!(require_extension, Some(FileExtensionConfig::IgnorePackages)) {
                    // When using ignorePackages, check if per-extension configs override it
                    // If js/ts configs are set to Never (not IgnorePackages), then missing extensions are OK
                    // Otherwise, require extension for non-packages
                    if matches!(config.js, FileExtensionConfig::IgnorePackages)
                        || matches!(config.ts, FileExtensionConfig::IgnorePackages)
                    {
                        ctx.diagnostic(extension_missing_diagnostic(span, true));
                    }
                }
            }
        }
    }
}
/// Determines if an import specifier is a package import (not relative or path alias)
fn is_package_import(module_name: &str) -> bool {
    // Relative imports start with ./ or ../
    if module_name.starts_with('.') {
        return false;
    }

    // Absolute paths start with /
    if module_name.starts_with('/') {
        return false;
    }

    // Check for path aliases with single-char prefix followed by /
    // Examples: @/, ~/, #/
    if module_name.len() >= 2 {
        let chars: Vec<char> = module_name.chars().collect();
        if chars.len() >= 2 && chars[1] == '/' {
            // Single character before slash - this is a path alias
            return false;
        }
    }

    // Scoped packages: @org/pkg where org has length >= 2
    // This handles @babel/core, @types/node, etc.
    if module_name.starts_with('@') {
        // Must have a slash to be a valid scoped package
        return module_name.contains('/');
    }

    // Bare imports without slash are packages: 'lodash', 'react'
    // Bare imports with slash are package subpaths: 'lodash/fp', 'react/dom'
    true
}

fn get_file_extension_from_module_name(module_name: &CompactStr) -> Option<CompactStr> {
    if let Some((_, extension)) =
        module_name.split('?').next().unwrap_or(module_name).rsplit_once('.')
        && !extension.is_empty()
        && !extension.starts_with('/')
    {
        return Some(CompactStr::from(extension));
    }

    None
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        // Default config: no extension requirements
        (r#"import a from "@/a""#, None),
        (r#"import a from "a""#, None),
        (r#"import dot from "./file.with.dot""#, None),
        // 'always': require extensions for all imports
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
        // 'never': no extensions allowed
        (r#"import thing from "non-package";"#, Some(json!(["never"]))),
        // Package subpaths are treated as packages (correctly handled with improved detection)
        (r#"import thing from "non-package/test";"#, Some(json!(["always"]))),
        // 'ignorePackages': require extensions for relative imports, not for packages
        // Packages are detected using string heuristics (not filesystem resolution)
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
        // Package detection: Root packages are names, not files
        // Note: @name/pkg.js is treated as scoped package, not a file
        (
            r#"
                import lib from "pkg.js";
                import lib2 from "pgk/package";
                import lib3 from "@name/pkg.js";
            "#,
            Some(json!(["never"])),
        ),
        // Query strings: extensions are extracted before the '?' character
        // This behavior matches ESLint
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
        // TypeScript type imports: ignored by default (checkTypeImports: false)
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
        // TypeScript type imports with checkTypeImports: true - extensions are checked
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
        // Edge cases: empty imports and unicode
        (r"import''", None),
        (r"export *from 'íìc'", None),
        (
            r"import { Something } from './something.hooks'; import SomeComponent from './SomeComponent.vue';",
            Some(json!(["ignorePackages", { "js": "never", "ts": "never" }])),
        ),
        // Configuration inheritance: when first arg is 'ignorePackages', per-extension configs
        // inherit IgnorePackages unless explicitly overridden. When overridden to 'never',
        // missing extensions are OK for those file types.
        // This matches ESLint behavior. https://github.com/oxc-project/oxc/issues/12220
        (
            r#"
                import { A } from './something';
            "#,
            Some(
                json!(["ignorePackages", { "js": "never", "ts": "never", "jsx": "never", "tsx": "never"}]),
            ),
        ),
        // Path alias ~/: correctly identified as non-package
        // Note: @/ path alias detection was fixed in Phase 2
        // https://github.com/oxc-project/oxc/issues/12220
        (
            r#"
                import { D } from '~/common/something';
            "#,
            Some(
                json!(["ignorePackages", { "js": "never", "ts": "never", "jsx": "never", "tsx": "never"}]),
            ),
        ),
        // Scoped package subpaths should be treated as packages
        (
            r#"
                import { foo } from '@scope/package/deep/nested/path';
            "#,
            Some(json!(["ignorePackages"])),
        ),
        // Mixed configuration: relative with extension, package without, scoped package subpath
        (
            r#"
                import a from './relative.js';
                import b from 'package';
                import c from '@org/pkg/sub';
            "#,
            Some(json!(["ignorePackages", { "js": "always" }])),
        ),
        // Phase 2: @/ path alias detection fix - passes when ignorePackages: false
        // @/ is correctly identified as a path alias (not a scoped package), so with extensions it passes
        (
            r#"
                import foo from '@/components/Foo.js';
                import bar from '@/utils/bar.ts';
            "#,
            Some(json!(["always", { "ignorePackages": false }])),
        ),
        // Phase 2: Other single-char path aliases are also correctly detected
        (
            r#"
                import a from '~/config.js';
                import b from '#/internal.ts';
            "#,
            Some(json!(["always", { "ignorePackages": false }])),
        ),
        // Phase 2: Scoped packages are still correctly distinguished from path aliases
        (
            r#"
                import babel from '@babel/core';
                import types from '@types/node';
            "#,
            Some(json!(["ignorePackages"])),
        ),
    ];

    let fail = vec![
        // Default config: package subpaths with extensions should fail
        (r#"import a from "a/index.js""#, None),
        // 'always' config: missing extensions should fail
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
        (r#"import "./bar.coffee""#, Some(json!(["never", { "js": "always", "jsx": "always" }]))),
        // https://github.com/oxc-project/oxc/issues/12220
        (
            r#"
                import { B } from './something.ts';
            "#,
            Some(json!(["ignorePackages", { "js": "never", "ts": "never" }])),
        ),
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
        // Query strings: extension detected before '?' should fail
        (r#"import withExtension from "./foo.js?a=True";"#, Some(json!(["never"]))),
        (r#"import withoutExtension from "./foo?a=True.ext";"#, Some(json!(["always"]))),
        // Require statements: same rules apply as import statements
        (
            r#"
                const { foo } = require("./foo");
                export { foo };
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                const { foo } = require("./foo.js");
                export { foo };
            "#,
            Some(json!(["never"])),
        ),
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
        // Directory imports: '.' and '..' are treated as relative imports without extensions
        // These fail with 'ignorePackages' because they're not packages and lack extensions
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
        // Directory imports with 'always' should fail
        (
            r#"
                import x from '.';
            "#,
            Some(json!(["always"])),
        ),
        (
            r#"
                import y from '..';
            "#,
            Some(json!(["always"])),
        ),
        // Scoped package subpaths with 'never' should fail when they have extensions (ignorePackages: false)
        (
            r#"
                import { bar } from '@scope/pkg/file.js';
            "#,
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        (
            r#"
                import { baz } from '@org/lib/sub/index.ts';
            "#,
            Some(json!(["never", { "ignorePackages": false }])),
        ),
        // Mixed configuration: some should pass, some should fail
        (
            r#"
                import x from './foo';
                import y from './bar.ts';
            "#,
            Some(json!(["always", { "ts": "never" }])),
        ),
    ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
