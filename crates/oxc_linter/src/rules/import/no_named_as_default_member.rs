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
use oxc_span::{CompactStr, Span};
use oxc_syntax::module_record::ImportImportName;

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-import(no-named-as-default-member): {1:?} also has a named export {2:?}")]
#[diagnostic(severity(warning), help("Check if you meant to write `import {{{2:}}} from {3:?}`"))]
struct NoNamedAsDefaultMemberDignostic(#[label] pub Span, String, String, String);

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/no-named-as-default-member.md>
#[derive(Debug, Default, Clone)]
pub struct NoNamedAsDefaultMember;

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
    NoNamedAsDefaultMember,
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

impl Rule for NoNamedAsDefaultMember {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();

        let mut has_members_map: HashMap<SymbolId, (Ref<'_, CompactStr, _, _>, CompactStr)> =
            HashMap::default();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(_) = import_entry.import_name else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let Some(remote_module_record_ref) = module_record.loaded_modules.get(specifier) else {
                continue;
            };

            if !remote_module_record_ref.exported_bindings.is_empty() {
                has_members_map.insert(
                    ctx.symbols().get_symbol_id_from_span(&import_entry.local_name.span()).unwrap(),
                    (remote_module_record_ref, import_entry.module_request.name().clone()),
                );
            }
        }

        if has_members_map.is_empty() {
            return;
        };
        let get_external_module_name_if_has_entry =
            |ident: &IdentifierReference, entry_name: &str| {
                get_symbol_id_from_ident(ctx, ident)
                    .and_then(|symbol_id| has_members_map.get(&symbol_id))
                    .and_then(|it| {
                        if it.0.exported_bindings.contains_key(entry_name) {
                            Some(it.1.to_string())
                        } else {
                            None
                        }
                    })
            };

        let process_member_expr = |member_expr: &MemberExpression| {
            let Expression::Identifier(ident) = member_expr.object() else {
                return;
            };
            let Some(prop_str) = member_expr.static_property_name() else {
                return;
            };
            if let Some(module_name) = get_external_module_name_if_has_entry(ident, prop_str) {
                ctx.diagnostic(NoNamedAsDefaultMemberDignostic(
                    match member_expr {
                        MemberExpression::ComputedMemberExpression(it) => it.span,
                        MemberExpression::StaticMemberExpression(it) => it.span,
                        MemberExpression::PrivateFieldExpression(it) => it.span,
                    },
                    ident.name.to_string(),
                    prop_str.to_string(),
                    module_name,
                ));
            };
        };

        for item in ctx.semantic().nodes().iter() {
            match item.kind() {
                AstKind::MemberExpression(member_expr) => process_member_expr(member_expr),
                AstKind::VariableDeclarator(decl) => {
                    let Some(Expression::Identifier(ident)) = &decl.init else {
                        continue;
                    };
                    let BindingPatternKind::ObjectPattern(object_pattern) = &decl.id.kind else {
                        continue;
                    };

                    for prop in &*object_pattern.properties {
                        let Some(name) = prop.key.static_name() else {
                            continue;
                        };
                        if let Some(module_name) =
                            get_external_module_name_if_has_entry(ident, &name)
                        {
                            ctx.diagnostic(NoNamedAsDefaultMemberDignostic(
                                decl.span,
                                ident.name.to_string(),
                                name.to_string(),
                                module_name,
                            ));
                        }
                    }
                }
                _ => {}
            }
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
        r"import baz from './named-exports';
        {
            const baz = {};
            const a = baz.looooooooooooooooooooooooong;
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

    Tester::new(NoNamedAsDefaultMember::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
