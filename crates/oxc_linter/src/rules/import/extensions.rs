use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{AstNode, context::LintContext, rule::Rule};

fn extensions_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong")
        .with_help("Should be a command-like statement that tells the user how to fix the issue")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct Extensions;

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

impl Rule for Extensions {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        println!("running lint rule...");

        // the default.rs rule has good reference points for reading file names in an import.
        if let Some(extension) = ctx.file_path().extension() {
            println!("file extension: {extension:?}");
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::{json, Value};

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
        (
            r#"
                import bar from "./bar";
                import barjson from "./bar.json";
                import barhbs from "./bar.hbs";
            "#,
            Some(json!(["always", { "js": "never", "jsx": "never"}])),
        ),
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

    let fail: Vec<(&str, Option<Value>)> = vec![];

    // let fail: Vec<(&str, Option<Value>)> = vec![
    //     (r#"import a from "a/index.js""#, None),
    //     (r#"import dot from "./file.with.dot""#, Some(json!(["always"]))),
    //     (
    //         r#"
    //             import a from "a/index.js";
    //             import packageConfig from "./package";
    //         "#,
    //         Some(json!([{ "json": "always", "js": "never"}])),
    //     ),
    //     (
    //         r#"
    //             import lib from "./bar.js";
    //             import component from "./bar.jsx";
    //             import data from "./bar.json";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import lib from "./bar.js";
    //             import component from "./bar.jsx";
    //             import data from "./bar.json";
    //         "#,
    //         Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
    //     ),
    //     (
    //         r#"
    //             import component from "./bar.jsx";
    //             import data from "./bar.json";
    //         "#,
    //         Some(json!([{ "json": "always", "js": "never", "jsx": "never" }])),
    //     ),
    //     (r#"import "./bar.coffee""#, Some(json!(["never", { "js": "always", "jsx": "always" }]))),
    //     (
    //         r#"
    //             import barjs from "./bar.js";
    //             import barjson from "./bar.json";
    //             import barnone from "./bar";
    //         "#,
    //         Some(json!(["always", { "json": "always", "js": "never", "jsx": "never" }])),
    //     ),
    //     (
    //         r#"
    //             import barjs from ".";
    //             import barjs2 from "..";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             import barjs from "./bar.js";
    //             import barjson from "./bar.json";
    //             import barnone from "./bar";
    //         "#,
    //         Some(json!(["never", { "json": "always", "js": "never", "jsx": "never" }])),
    //     ),
    //     (
    //         r#"
    //             import thing from "./fake-file.js";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import thing from "non-package/test";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             import thing from "@name/pkg/test";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             import thing from "@name/pkg/test.js";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import foo from './foo.js';
    //             import bar from './bar.json';
    //             import Component from './Component';
    //             import baz from 'foo/baz';
    //             import baw from '@scoped/baw/import';
    //             import chart from '@/configs/chart';
    //             import express from 'express';
    //         "#,
    //         Some(json!(["always", { "ignorePackages": true }])),
    //     ),
    //     (
    //         r#"
    //             import foo from './foo.js';
    //             import bar from './bar.json';
    //             import Component from './Component';
    //             import baz from 'foo/baz';
    //             import baw from '@scoped/baw/import';
    //             import chart from '@/configs/chart';
    //             import express from 'express';
    //         "#,
    //         Some(json!(["ignorePackages"])),
    //     ),
    //     (
    //         r#"
    //             import foo from './foo.js';
    //             import bar from './bar.json';
    //             import Component from './Component.jsx';
    //             import express from 'express';
    //         "#,
    //         Some(json!(["never", { "ignorePackages": true }])),
    //     ),
    //     (
    //         r#"
    //             import foo from './foo.js';
    //             import bar from './bar.json';
    //             import Component from './Component.jsx';
    //         "#,
    //         Some(json!(["always", { "pattern": { "jsx": "never" } }])),
    //     ),
    //     // Exports
    //     (
    //         r#"
    //             export { foo } from "./foo";
    //             let bar; export { bar };
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             export { foo } from "./foo.js";
    //             let bar; export { bar };
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     // Query strings
    //     (r#"import withExtension from "./foo.js?a=True";"#, Some(json!(["never"]))),
    //     (r#"import withoutExtension from "./foo?a=True.ext";"#, Some(json!(["always"]))),
    //     // Require
    //     (
    //         r#"
    //             const { foo } = require("./foo");
    //             export { foo };
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             const { foo } = require("./foo".js);
    //             export { foo };
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import foo from "@/ImNotAScopedModule";
    //             import chart from "@/configs/chart";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     // Export { } from
    //     (
    //         r#"
    //             export { foo } from "./foo";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             export { foo } from "./foo.js";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     // Export * from
    //     (
    //         r#"
    //             export * from "./foo";
    //         "#,
    //         Some(json!(["always"])),
    //     ),
    //     (
    //         r#"
    //             export * from "./foo.js";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import foo from "@/ImNotAScopedModule.js";
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     (
    //         r#"
    //             import _ from 'lodash';
    //             import m from '@test-scope/some-module/index.js';
    //             import bar from './bar';
    //         "#,
    //         Some(json!(["never"])),
    //     ),
    //     // Relative imports
    //     (
    //         r#"
    //             import * as test from ".";
    //         "#,
    //         Some(json!(["ignorePackages"])),
    //     ),
    //     (
    //         r#"
    //             import * as test from "..";
    //         "#,
    //         Some(json!(["ignorePackages"])),
    //     ),
    //     // Type imports
    //     (
    //         r#"
    //             import T from "./typescript-declare";
    //         "#,
    //         Some(
    //             json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
    //         ),
    //     ),
    //     (
    //         r#"
    //             export { MyType } from "./typescript-declare";
    //         "#,
    //         Some(
    //             json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]),
    //         ),
    //     ),
    //     (
    //         r#"
    //             import type T from "./typescript-declare";
    //         "#,
    //         Some(
    //             json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
    //         ),
    //     ),
    //     (
    //         r#"
    //             export type { MyType } from "./typescript-declare";
    //         "#,
    //         Some(
    //             json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never", "checkTypeImports": true }]),
    //         ),
    //     ),
    //     (
    //         r#"
    //             import type { MyType } from "./typescript-declare";
    //         "#,
    //         Some(json!(["always", { "checkTypeImports": true }])),
    //     ),
    //     (
    //         r#"
    //             export type { MyType } from "./typescript-declare";
    //         "#,
    //         Some(json!(["always", { "checkTypeImports": true }])),
    //     ),
    // ];

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
