use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{Span, VALID_EXTENSIONS};

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn default_diagnostic(imported_name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("No default export found in imported module {imported_name:?}"))
        .with_help(format!("does {imported_name:?} have the default export?"))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/default.md>
#[derive(Debug, Default, Clone)]
pub struct Default;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// If a default import is requested, this rule will report if there is no
    /// default export in the imported module.
    ///
    /// ### Why is this bad?
    ///
    /// Using a default import when there is no default export can lead to
    /// confusion and runtime errors. It can make the code harder to understand
    /// and maintain, as it may suggest that a module has a default export
    /// when it does not, leading to unexpected behavior.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // ./bar.js
    /// export function bar() { return null }
    ///
    /// // ./foo.js
    /// import bar from './bar' // no default export found in ./bar
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // ./bar.js
    /// export default function bar() { return null }
    ///
    /// // ./foo.js
    /// import { bar } from './bar' // correct usage of named import
    /// ```
    Default,
    import,
    correctness
);

impl Rule for Default {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();
        let loaded_modules = module_record.loaded_modules.read().unwrap();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(default_span) = import_entry.import_name else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record) = loaded_modules.get(specifier) else {
                continue;
            };
            if !remote_module_record.has_module_syntax {
                continue;
            }
            if !remote_module_record
                .resolved_absolute_path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| VALID_EXTENSIONS.contains(&ext))
            {
                continue;
            }
            if remote_module_record.export_default.is_none()
                && !remote_module_record.exported_bindings.contains_key("default")
            {
                ctx.diagnostic(default_diagnostic(specifier, default_span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "import './malformed.js'",
        r#"import foo from "./empty-folder""#,
        r#"import { foo } from "./default-export""#,
        r#"import foo from "./default-export""#,
        r#"import foo from "./mixed-exports""#,
        r#"import bar from "./default-export""#,
        r#"import CoolClass from "./default-class""#,
        r#"import bar, { baz } from "./default-export""#,
        r#"import crypto from "crypto""#,
        r#"import common from "./common""#,
        // No longer valid syntax
        // r#"export bar from "./bar""#,
        // r#"export bar, { foo } from "./bar""#,
        // r#"export bar, * as names from "./bar""#,
        // r#"export bar from './default-export-from-ignored.js'"#,
        r#"export { default as bar } from "./bar""#,
        r#"export { default as bar, foo } from "./bar""#,
        r#"export {a} from "./named-exports""#,
        r#"import twofer from "./trampoline""#,
        r#"import MyCoolComponent from "./jsx/MyCoolComponent.jsx""#,
        r#"import foo from "./named-default-export""#,
        r#"import connectedApp from "./redux""#,
        r"import Foo from './jsx/FooES7.js'",
        r"import bar from './default-export-from.js'",
        r"import bar from './default-export-from-named.js'",
        r"import bar from './default-export-from-ignored.js'",
        r#"export { "default" as bar } from "./bar""#,
        r#"import foo from "./jsx/MyUncoolComponent.jsx""#,
        r#"import foobar from "./typescript-default""#,
        r#"import foobar from "./typescript-export-assign-default""#,
        r#"import foobar from "./typescript-export-assign-mixed""#,
        r#"import Foo from "./typescript-export-as-default-namespace""#,
        r#"import Foo from "./typescript-export-react-test-renderer""#,
        r#"import Foo from "./typescript-extended-config""#,
        // TODO: `export =`
        // r#"import foobar from "./typescript-export-assign-function""#,
        // r#"import foobar from "./typescript-export-assign-property""#,
        // r#"import foobar from "./typescript-export-assign-default-reexport""#,
        // r#"import React from "./typescript-export-assign-default-namespace"#,
        r#"import Foo from "./vue/main.vue""#,
    ];

    let fail = vec![
        r#"import baz from "./named-exports""#,
        r#"export baz from "./named-exports""#,
        r#"export baz, { bar } from "./named-exports""#,
        r#"export baz, * as names from "./named-exports""#,
        // broken-trampoline.js contains no longer valid syntax
        // r#"import twofer from "./broken-trampoline""#,
        r#"import barDefault from "./re-export""#,
        // Ignore Case Sensitivity
        // r#"import bar from "./Named-Exports""#,
        r#"import foobar from "./typescript""#,
        // TODO: `export =`
        // r#"import React from "./typescript-export-assign-default-namespace""#,

        // TODO: resolve index.d.ts
        // r#"import FooBar from "./typescript-export-as-default-namespace""#,
        // r#"import Foo from "./typescript-export-as-default-namespace""#,
    ];

    Tester::new(Default::NAME, Default::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
