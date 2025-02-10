use std::sync::Arc;

use oxc_ast::{
    ast::{BindingPatternKind, Expression, IdentifierReference, MemberExpression},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::SymbolId;
use oxc_span::Span;
use rustc_hash::FxHashMap;

use crate::{context::LintContext, module_record::ImportImportName, rule::Rule};

fn no_named_as_default_member_dignostic(
    span: Span,
    module_name: &str,
    export_name: &str,
    suggested_module_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("{module_name:?} also has a named export {export_name:?}"))
        .with_help(format!("Check if you meant to write `import {{ {export_name} }} from {suggested_module_name:?}`"))
        .with_label(span)
}

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/no-named-as-default-member.md>
#[derive(Debug, Default, Clone)]
pub struct NoNamedAsDefaultMember;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Reports the use of an exported name (named export) as a property on the
    /// default export. This occurs when trying to access a named export through
    /// the default export, which is incorrect.
    ///
    /// ### Why is this bad?
    ///
    /// Accessing a named export via the default export is incorrect and will not
    /// work as expected. Named exports should be imported directly, while default
    /// exports are accessed without properties. This mistake can lead to runtime
    /// errors or undefined behavior.
    ///
    /// ### Examples
    ///
    /// Given
    /// ```javascript
    /// // ./bar.js
    /// export function bar() { return null }
    /// export default () => { return 1 }
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // ./foo.js
    /// import foo from './bar'
    /// const bar = foo.bar; // Incorrect: trying to access named export via default
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // ./foo.js
    /// import { bar } from './bar'; // Correct: accessing named export directly
    /// ```
    NoNamedAsDefaultMember,
    import,
    suspicious
);

fn get_symbol_id_from_ident(
    ctx: &LintContext<'_>,
    ident: &IdentifierReference,
) -> Option<SymbolId> {
    ctx.symbols().get_reference(ident.reference_id()).symbol_id()
}

impl Rule for NoNamedAsDefaultMember {
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.module_record();

        let mut has_members_map = FxHashMap::default();
        for import_entry in &module_record.import_entries {
            let ImportImportName::Default(_) = import_entry.import_name else {
                continue;
            };

            let specifier = import_entry.module_request.name();
            let remote_module_record = module_record.loaded_modules.read().unwrap();
            let Some(remote_module_record) = remote_module_record.get(specifier) else {
                continue;
            };

            if remote_module_record.exported_bindings.is_empty() {
                continue;
            }

            let Some(symbol_id) = ctx.scopes().get_root_binding(import_entry.local_name.name())
            else {
                return;
            };

            has_members_map.insert(
                symbol_id,
                (Arc::clone(remote_module_record), import_entry.module_request.clone()),
            );
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
                            Some(&it.1)
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
                ctx.diagnostic(no_named_as_default_member_dignostic(
                    match member_expr {
                        MemberExpression::ComputedMemberExpression(it) => it.span,
                        MemberExpression::StaticMemberExpression(it) => it.span,
                        MemberExpression::PrivateFieldExpression(it) => it.span,
                    },
                    &ident.name,
                    prop_str,
                    module_name.name(),
                ));
            };
        };

        for item in ctx.semantic().nodes() {
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
                            ctx.diagnostic(no_named_as_default_member_dignostic(
                                decl.span,
                                &ident.name,
                                &name,
                                module_name.name(),
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

    Tester::new(NoNamedAsDefaultMember::NAME, NoNamedAsDefaultMember::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
