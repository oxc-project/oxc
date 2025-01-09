use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    module_record::{ExportImportName, ImportImportName},
    rule::Rule,
};

fn named_diagnostic(imported_name: &str, module_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("named import {imported_name:?} not found"))
        .with_help(format!("does {module_name:?} have the export {imported_name:?}?"))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/named.md>
#[derive(Debug, Default, Clone)]
pub struct Named;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Verifies that all named imports are part of the set of named exports in
    /// the referenced module.
    ///
    /// For `export`, verifies that all named exports exist in the referenced
    /// module.
    ///
    /// Note: for packages, the plugin will find exported names from
    /// `jsnext:main` (deprecated) or `module`, if present in `package.json`.
    /// Redux's npm module includes this key, and thereby is lintable, for
    /// example.
    ///
    /// A module path that is ignored or not unambiguously an ES module will not
    /// be reported when imported. Note that type imports and exports, as used
    /// by Flow, are always ignored.
    ///
    /// ### Why is this bad?
    ///
    /// Importing or exporting names that do not exist in the referenced module
    /// can lead to runtime errors and confusion. It may suggest that certain
    /// functionality is available when it is not, making the code harder to
    /// maintain and understand. This rule helps ensure that your code
    /// accurately reflects the available exports, improving reliability.
    ///
    /// ### Examples
    ///
    /// Given
    /// ```js
    /// // ./foo.js
    /// export const foo = "I'm so foo";
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// // ./baz.js
    /// import { notFoo } from './foo'
    ///
    /// // ES7 proposal
    /// export { notFoo as defNotBar } from './foo'
    ///
    /// // will follow 'jsnext:main', if available
    /// import { dontCreateStore } from 'redux'
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// // ./bar.js
    /// import { foo } from './foo'
    ///
    /// // ES7 proposal
    /// export { foo as bar } from './foo'
    ///
    /// // node_modules without jsnext:main are not analyzed by default
    /// // (import/ignore setting)
    /// import { SomeNonsenseThatDoesntExist } from 'react'
    /// ```
    Named,
    import,
    nursery // There are race conditions in the runtime which may cause the module to
            // not find any exports from `exported_bindings_from_star_export`.
);

impl Rule for Named {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let semantic = ctx.semantic();

        // This rule is disabled in the typescript config.
        if semantic.source_type().is_typescript() {
            return;
        }

        let module_record = ctx.module_record();

        let loaded_modules = module_record.loaded_modules.read().unwrap();
        for import_entry in &module_record.import_entries {
            // Get named import
            let ImportImportName::Name(import_name) = &import_entry.import_name else {
                continue;
            };
            let specifier = import_entry.module_request.name();
            // Get remote module record
            let Some(remote_module_record) = loaded_modules.get(specifier) else {
                continue;
            };
            if !remote_module_record.has_module_syntax {
                continue;
            }
            let import_span = import_name.span();
            let name = import_name.name();
            // Check `import { default as foo } from 'bar'`
            if name == "default" && remote_module_record.export_default.is_some() {
                continue;
            }
            // Check remote bindings
            if remote_module_record.exported_bindings.contains_key(name) {
                continue;
            }
            // check re-export
            if remote_module_record
                .exported_bindings_from_star_export()
                .iter()
                .any(|(_, value)| value.contains(&import_name.name))
            {
                continue;
            }

            ctx.diagnostic(named_diagnostic(name, specifier, import_span));
        }

        let loaded_modules = module_record.loaded_modules.read().unwrap();
        for export_entry in &module_record.indirect_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let ExportImportName::Name(import_name) = &export_entry.import_name else {
                continue;
            };
            let specifier = module_request.name();
            // Get remote module record
            let Some(remote_module_record) = loaded_modules.get(specifier) else {
                continue;
            };
            if !remote_module_record.has_module_syntax {
                continue;
            }
            // Check remote bindings
            let name = import_name.name();
            // `export { default as foo } from './source'` <> `export default xxx`
            if name == "default" && remote_module_record.export_default.is_some() {
                continue;
            }
            if remote_module_record.exported_bindings.contains_key(name) {
                continue;
            }
            ctx.diagnostic(named_diagnostic(name, specifier, import_name.span()));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import './malformed.js'",
        "import { foo } from './bar'",
        "import { foo } from './empty-module'",
        "import bar from './bar.js'",
        "import bar, { foo } from './bar.js'",
        "import {a, b, d} from './named-exports'",
        "import {ExportedClass} from './named-exports'",
        "import { destructingAssign } from './named-exports'",
        "import { destructingRenamedAssign } from './named-exports'",
        "import { ActionTypes } from './qc'",
        "import {a, b, c, d} from './re-export'",
        // "import {RuleTester} from './re-export-node_modules'",
        "import { jsxFoo } from './jsx/AnotherComponent'",
        "import {a, b, d} from './common'; // eslint-disable-line named",
        "import { foo, bar } from './re-export-names'",
        // TODO: module.exports
        // "import {a, b, c} from './re-export-common-star'",
        // "import { foo, bar } from './common'",
        // ignore core modules by default
        "import { foo } from 'crypto'",
        // "import { zoob } from 'a'",
        "import { someThing } from './test-module'",
        "export { foo } from './bar'",
        "export { foo as bar } from './bar'",
        "export { foo } from './does-not-exist'",
        // No longer valid syntax
        // "export bar, { foo } from './bar'",
        // "import { foo, bar } from './named-trampoline'",
        "let foo; export { foo as bar }",
        "import { destructuredProp } from './named-exports'",
        "import { arrayKeyProp } from './named-exports'",
        "import { deepProp } from './named-exports'",
        "import { deepSparseElement } from './named-exports'",
        // Flow not supported
        // "import type { MissingType } from './flowtypes'",
        // "import typeof { MissingType } from './flowtypes'",
        // "import type { MyOpaqueType } from './flowtypes'",
        // "import typeof { MyOpaqueType } from './flowtypes'",
        // "import { type MyOpaqueType, MyClass } from './flowtypes'",
        // "import { typeof MyOpaqueType, MyClass } from './flowtypes'",
        // "import typeof MissingType from './flowtypes'",
        // "import typeof * as MissingType from './flowtypes'",
        // "export type { MissingType } from './flowtypes'",
        // "export type { MyOpaqueType } from './flowtypes'",
        "/*jsnext*/ import { createStore } from 'redux'",
        "/*jsnext*/ import { createStore } from 'redux'",
        "import { foo } from 'es6-module'",
        "import { me, soGreat } from './narcissist'",
        "import { foo, bar, baz } from './re-export-default'",
        "import { common } from './re-export-default'",
        // "import {a, b, d} from './common'",
        // settings: { 'import/ignore': ['bar'] },
        // "import { baz } from './bar'",
        "import { common } from './re-export-default'",
        // "const { destructuredProp } = require('./named-exports')",
        // "let { arrayKeyProp } = require('./named-exports')",
        // "const { deepProp } = require('./named-exports')",
        // "const { foo, bar } = require('./re-export-names')",
        // "const { baz } = require('./bar')",
        // "const { baz } = require('./bar')",
        // "const { default: defExport } = require('./bar')",
        // "import { ExtfieldModel, Extfield2Model } from './models';",       filename: testFilePath('./export-star/downstream.js'),
        // "const { something } = require('./dynamic-import-in-commonjs')",
        // "import { something } from './dynamic-import-in-commonjs'",
        "import { 'foo' as foo } from './bar'",
        "import { 'foo' as foo } from './empty-module'",
        // export all
        "import { foo } from './export-all'",
        // TypeScript export assignment
        "import x from './typescript-export-assign-object'",
        "export { default as foo } from './typescript-export-default'",
        "import { default as foo } from './typescript-export-default'",
    ];

    let fail = vec![
        "import { somethingElse } from './test-module'",
        "import { baz } from './bar'",
        "import { baz, bop } from './bar'",
        "import {a, b, c} from './named-exports'",
        "import { a } from './default-export'",
        "import { ActionTypes1 } from './qc'",
        "import {a, b, c, d, e} from './re-export'",
        "import { a } from './re-export-names'",
        "export { bar } from './bar'",
        "export bar2, { bar } from './bar'",
        // old babel parser
        // "import { foo, bar, baz } from './named-trampoline'",
        // "import { baz } from './broken-trampoline'",
        // cjs
        // "const { baz } = require('./bar')",
        // "let { baz } = require('./bar')",
        // "const { baz: bar, bop } = require('./bar'), { a } = require('./re-export-names')",
        // "const { default: defExport } = require('./named-exports')",
        // flow
        // "import  { type MyOpaqueType, MyMissingClass } from './flowtypes'",
        // jsnext
        // "/*jsnext*/ import { createSnorlax } from 'redux'",
        "import { baz } from 'es6-module'",
        "import { foo, bar, bap } from './re-export-default'",
        "import { default as barDefault } from './re-export'",
        // export all
        "import { bar } from './export-all'",
        // TypeScript
        // Export assignment cannot be used when targeting ECMAScript modules. Consider using 'export default' or another module format instead.
        "import { NotExported } from './typescript-export-assign-object'",
        "import { FooBar } from './typescript-export-assign-object'",
    ];

    Tester::new(Named::NAME, Named::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
