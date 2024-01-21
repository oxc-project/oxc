#![allow(clippy::significant_drop_tightening)]
use std::collections::HashMap;

use dashmap::mapref::one::Ref;
use oxc_ast::{
    ast::{BindingPatternKind, Expression, IdentifierReference, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::{Atom, Span};
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-named-as-default-member): {1:?} also has a named export {2:?}")]
#[diagnostic(severity(warning), help("Check if you meant to write `import {{{2:}}} from {3:?}`"))]
struct NoNamedAsDefaultDiagnostic(#[label] pub Span, String, String, String);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-named-as-default-member.md>
#[derive(Debug, Default, Clone)]
pub struct NoNamedAsDefault;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports use of an exported name as a property on the default export.
    ///
    /// ### Example
    ///
    /// ```javascript
    /// // ./bar.js
    /// export function bar() { return null }
    /// export default () => { return 1 }
    ///
    /// // ./foo.js
    /// import bar from './bar'
    /// const bar = foo.bar // trying to access named export via default
    /// ```
    NoNamedAsDefault,
    nursery
);
fn get_symbol_id_from_ident(
    ctx: &LintContext<'_>,
    ident: &IdentifierReference,
) -> Option<SymbolId> {
    let reference_id = ident.reference_id.get().unwrap();
    let reference = &ctx.symbols().references[reference_id];
    reference.symbol_id()
}

impl Rule for NoNamedAsDefault {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(import_span) = import_entry.import_name else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };

            if !remote_module_record_ref.exported_bindings.is_empty() {}
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import baz, {a} from "./named-exports""#,
        r#"import baz from "./named-exports"; const jjj = bar.jjj"#,
        r#"import {a} from "./named-exports"; const baz = a.baz"#,
        r#"import baz from "./default_export_default_property"; const d = baz.default;"#,
        r#"import baz, {foo} from "./named-and-default-export"; const d = baz.default;"#,
        r"import baz from './named-exports';
        {
            const baz = {};
            const a = baz.a;
        }",
    ];

    let fail = vec![
        r#"import baz from "./named-exports"; const a = baz.a;"#,
        r#"import baz from "./named-exports"; const a = baz["a"];"#,
        r#"import baz from "./named-exports"; baz.a();"#,
        r"import baz from './named-exports';
        {
            const a = baz.a;
        }",
        r#"import baz, { bar } from "./named-exports"; const {a} = baz"#,
        r#"import baz from "./named-and-default-export"; const {foo: _foo} = baz"#,
    ];

    Tester::new(NoNamedAsDefault::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
