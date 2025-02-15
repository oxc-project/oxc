use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    AstNode,
};

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
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    // const ruleTester = new RuleTester();
    // const ruleTesterWithTypeScriptImports = new RuleTester({
    //   settings: {
    //     'import/resolver': {
    //       typescript: {
    //         alwaysTryTypes: true,
    //       },
    //     },
    //   },
    // });
    let pass: Vec<(&str, Option<serde_json::Value>)> = vec![
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
            Some(json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never"}]))
        ),
        (
            r#"export type { MyType } from "./typescript-declare";"#,
            Some(json!(["always", { "ts": "never", "tsx": "never", "js": "never", "jsx": "never" }]))
        )
    ];

    let fail: Vec<(&str, Option<serde_json::Value>)> = vec![
        (
            r#"import a from "a/index.js""#,
            None
        ),
        (
            r#"import dot from "./file.with.dot""#,
            Some(json!(["always"]))
        ),
        (
            r#"
                import a from "a/index.js";
                import packageConfig from "./package";
            "#,
            Some(json!([{ "json": "always", "js": "never"}]))
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!(["never"]))
        ),
        (
            r#"
                import lib from "./bar.js";
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }]))
        ),
        (
            r#"
                import component from "./bar.jsx";
                import data from "./bar.json";
            "#,
            Some(json!([{ "json": "always", "js": "never", "jsx": "never" }]))
        ),
        (
            r#"import "./bar.coffee""#,
            Some(json!(["never", { "js": "always", "jsx": "always" }]))
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
            Some(json!(["always"]))
        ),
        (
            r#"
                import barjs from "./bar.js";
                import barjson from "./bar.json";
                import barnone from "./bar";
            "#,
            Some(json!(["never", { "json": "always", "js": "never", "jsx": "never" }]))
        ),
    ];


    // ruleTester.run('extensions', rule, {
    //   invalid: [
    //     // extension resolve order (#583/#965)

    //     // unresolved (#271/#295)
    //     test({
    //       code: 'import thing from "./fake-file.js"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./fake-file.js"',
    //           line: 1,
    //           column: 19,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: 'import thing from "non-package/test"',
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "non-package/test"',
    //           line: 1,
    //           column: 19,
    //         },
    //       ],
    //     }),

    //     test({
    //       code: 'import thing from "@name/pkg/test"',
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "@name/pkg/test"',
    //           line: 1,
    //           column: 19,
    //         },
    //       ],
    //     }),

    //     test({
    //       code: 'import thing from "@name/pkg/test.js"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "@name/pkg/test.js"',
    //           line: 1,
    //           column: 19,
    //         },
    //       ],
    //     }),

    //     test({
    //       code: `
    //         import foo from './foo.js'
    //         import bar from './bar.json'
    //         import Component from './Component'
    //         import baz from 'foo/baz'
    //         import baw from '@scoped/baw/import'
    //         import chart from '@/configs/chart'
    //         import express from 'express'
    //       `,
    //       options: ['always', { ignorePackages: true }],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./Component"',
    //           line: 4,
    //           column: 31,
    //         },
    //         {
    //           message: 'Missing file extension for "@/configs/chart"',
    //           line: 7,
    //           column: 27,
    //         },
    //       ],
    //     }),

    //     test({
    //       code: `
    //         import foo from './foo.js'
    //         import bar from './bar.json'
    //         import Component from './Component'
    //         import baz from 'foo/baz'
    //         import baw from '@scoped/baw/import'
    //         import chart from '@/configs/chart'
    //         import express from 'express'
    //       `,
    //       options: ['ignorePackages'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./Component"',
    //           line: 4,
    //           column: 31,
    //         },
    //         {
    //           message: 'Missing file extension for "@/configs/chart"',
    //           line: 7,
    //           column: 27,
    //         },
    //       ],
    //     }),

    //     test({
    //       code: `
    //         import foo from './foo.js'
    //         import bar from './bar.json'
    //         import Component from './Component.jsx'
    //         import express from 'express'
    //       `,
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js"',
    //           line: 2,
    //           column: 25,
    //         }, {
    //           message: 'Unexpected use of file extension "jsx" for "./Component.jsx"',
    //           line: 4,
    //           column: 31,
    //         },
    //       ],
    //       options: ['never', { ignorePackages: true }],
    //     }),

    //     test({
    //       code: `
    //         import foo from './foo.js'
    //         import bar from './bar.json'
    //         import Component from './Component.jsx'
    //       `,
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "jsx" for "./Component.jsx"',
    //           line: 4,
    //           column: 31,
    //         },
    //       ],
    //       options: ['always', { pattern: { jsx: 'never' } }],
    //     }),

    //     // export (#964)
    //     test({
    //       code: [
    //         'export { foo } from "./foo"',
    //         'let bar; export { bar }',
    //       ].join('\n'),
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./foo"',
    //           line: 1,
    //           column: 21,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: [
    //         'export { foo } from "./foo.js"',
    //         'let bar; export { bar }',
    //       ].join('\n'),
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js"',
    //           line: 1,
    //           column: 21,
    //         },
    //       ],
    //     }),

    //     // Query strings.
    //     test({
    //       code: 'import withExtension from "./foo.js?a=True"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js?a=True"',
    //           line: 1,
    //           column: 27,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: 'import withoutExtension from "./foo?a=True.ext"',
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./foo?a=True.ext"',
    //           line: 1,
    //           column: 30,
    //         },
    //       ],
    //     }),
    //     // require (#1230)
    //     test({
    //       code: [
    //         'const { foo } = require("./foo")',
    //         'export { foo }',
    //       ].join('\n'),
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./foo"',
    //           line: 1,
    //           column: 25,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: [
    //         'const { foo } = require("./foo.js")',
    //         'export { foo }',
    //       ].join('\n'),
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js"',
    //           line: 1,
    //           column: 25,
    //         },
    //       ],
    //     }),

    //     // export { } from
    //     test({
    //       code: 'export { foo } from "./foo"',
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./foo"',
    //           line: 1,
    //           column: 21,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: `
    //         import foo from "@/ImNotAScopedModule";
    //         import chart from '@/configs/chart';
    //       `,
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "@/ImNotAScopedModule"',
    //           line: 2,
    //         },
    //         {
    //           message: 'Missing file extension for "@/configs/chart"',
    //           line: 3,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: 'export { foo } from "./foo.js"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js"',
    //           line: 1,
    //           column: 21,
    //         },
    //       ],
    //     }),

    //     // export * from
    //     test({
    //       code: 'export * from "./foo"',
    //       options: ['always'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "./foo"',
    //           line: 1,
    //           column: 15,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: 'export * from "./foo.js"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "./foo.js"',
    //           line: 1,
    //           column: 15,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: 'import foo from "@/ImNotAScopedModule.js"',
    //       options: ['never'],
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "@/ImNotAScopedModule.js"',
    //           line: 1,
    //         },
    //       ],
    //     }),
    //     test({
    //       code: `
    //         import _ from 'lodash';
    //         import m from '@test-scope/some-module/index.js';

    //         import bar from './bar';
    //       `,
    //       options: ['never'],
    //       settings: {
    //         'import/resolver': 'webpack',
    //         'import/external-module-folders': ['node_modules', 'symlinked-module'],
    //       },
    //       errors: [
    //         {
    //           message: 'Unexpected use of file extension "js" for "@test-scope/some-module/index.js"',
    //           line: 3,
    //         },
    //       ],
    //     }),

    //     // TODO: properly ignore packages resolved via relative imports
    //     test({
    //       code: [
    //         'import * as test from "."',
    //       ].join('\n'),
    //       filename: testFilePath('./internal-modules/test.js'),
    //       options: ['ignorePackages'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for "."',
    //           line: 1,
    //         },
    //       ],
    //     }),
    //     // TODO: properly ignore packages resolved via relative imports
    //     test({
    //       code: [
    //         'import * as test from ".."',
    //       ].join('\n'),
    //       filename: testFilePath('./internal-modules/plugins/plugin.js'),
    //       options: ['ignorePackages'],
    //       errors: [
    //         {
    //           message: 'Missing file extension for ".."',
    //           line: 1,
    //         },
    //       ],
    //     }),
    //   ],
    // });

    // describe('TypeScript', () => {
    //   getTSParsers()
    //     // Type-only imports were added in TypeScript ESTree 2.23.0
    //     .filter((parser) => parser !== parsers.TS_OLD)
    //     .forEach((parser) => {
    //       ruleTester.run(`${parser}: extensions ignore type-only`, rule, {
    //         invalid: [
    //           test({
    //             code: 'import T from "./typescript-declare";',
    //             errors: ['Missing file extension for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { ts: 'never', tsx: 'never', js: 'never', jsx: 'never' },
    //             ],
    //             parser,
    //           }),
    //           test({
    //             code: 'export { MyType } from "./typescript-declare";',
    //             errors: ['Missing file extension for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { ts: 'never', tsx: 'never', js: 'never', jsx: 'never' },
    //             ],
    //             parser,
    //           }),
    //           test({
    //             code: 'import type T from "./typescript-declare";',
    //             errors: ['Missing file extension for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { ts: 'never', tsx: 'never', js: 'never', jsx: 'never', checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //           test({
    //             code: 'export type { MyType } from "./typescript-declare";',
    //             errors: ['Missing file extension for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { ts: 'never', tsx: 'never', js: 'never', jsx: 'never', checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //         ],
    //       });
    //       ruleTesterWithTypeScriptImports.run(`${parser}: (with TS resolver) extensions are enforced for type imports/export when checkTypeImports is set`, rule, {
    //         valid: [
    //           test({
    //             code: 'import type { MyType } from "./typescript-declare.ts";',
    //             options: [
    //               'always',
    //               { checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //           test({
    //             code: 'export type { MyType } from "./typescript-declare.ts";',
    //             options: [
    //               'always',
    //               { checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //         ],
    //         invalid: [
    //           test({
    //             code: 'import type { MyType } from "./typescript-declare";',
    //             errors: ['Missing file extension "ts" for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //           test({
    //             code: 'export type { MyType } from "./typescript-declare";',
    //             errors: ['Missing file extension "ts" for "./typescript-declare"'],
    //             options: [
    //               'always',
    //               { checkTypeImports: true },
    //             ],
    //             parser,
    //           }),
    //         ],
    //       });
    //     });
    // });

    Tester::new(Extensions::NAME, Extensions::PLUGIN, pass, fail).test_and_snapshot();
}
