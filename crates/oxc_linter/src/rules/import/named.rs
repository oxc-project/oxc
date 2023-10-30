use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};
use oxc_syntax::module_record::{ExportImportName, ImportImportName};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(named): named import {0:?} not found")]
#[diagnostic(severity(warning), help("does {1:?} have the export {0:?}?"))]
struct NamedDiagnostic(Atom, Atom, #[label] pub Span);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/named.md>
#[derive(Debug, Default, Clone)]
pub struct Named;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// ### Why is this bad?
    ///
    /// ### Example
    /// ```javascript
    /// ```
    Named,
    nursery
);

impl Rule for Named {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let semantic = ctx.semantic();

        // This rule is disabled in the typescript config.
        if semantic.source_type().is_typescript() {
            return;
        }

        let module_record = semantic.module_record();

        for import_entry in &module_record.import_entries {
            // Get named import
            let ImportImportName::Name(import_name) = &import_entry.import_name else {
                continue;
            };
            let specifier = import_entry.module_request.name();
            // Get remote module record
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };
            let remote_module_record = remote_module_record_ref.value();
            // Check remote bindings
            if remote_module_record.exported_bindings.contains_key(import_name.name()) {
                continue;
            }
            ctx.diagnostic(NamedDiagnostic(
                import_name.name().clone(),
                specifier.clone(),
                import_name.span(),
            ));
        }

        for export_entry in &module_record.indirect_export_entries {
            let Some(module_request) = &export_entry.module_request else {
                continue;
            };
            let ExportImportName::Name(import_name) = &export_entry.import_name else {
                continue;
            };
            let specifier = module_request.name();
            // Get remote module record
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };
            let remote_module_record = remote_module_record_ref.value();
            // Check remote bindings
            if remote_module_record.exported_bindings.contains_key(import_name.name()) {
                continue;
            }
            ctx.diagnostic(NamedDiagnostic(
                import_name.name().clone(),
                specifier.clone(),
                import_name.span(),
            ));
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
        // TODO: export *
        // "import {a, b, c, d} from './re-export'",
        // "import {a, b, c} from './re-export-common-star'",
        // "import {RuleTester} from './re-export-node_modules'",
        // "import { jsxFoo } from './jsx/AnotherComponent'",
        "import {a, b, d} from './common'; // eslint-disable-line named",
        "import { foo, bar } from './re-export-names'",
        // TODO: module.exports
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
        // "import { common } from './re-export-default'",
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
        // "import { foo } from './export-all'",
        // TypeScript export assignment
        "import x from './typescript-export-assign-object'",
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
        "const { baz } = require('./bar')",
        "let { baz } = require('./bar')",
        "const { baz: bar, bop } = require('./bar'), { a } = require('./re-export-names')",
        "const { default: defExport } = require('./named-exports')",
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

    Tester::new_without_config(Named::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
