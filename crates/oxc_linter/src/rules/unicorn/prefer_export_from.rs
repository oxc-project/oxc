use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, ExportDefaultDeclarationKind, ModuleExportName, Statement,
        VariableDeclarationKind,
    },
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{NodeId, ReferenceId, SymbolId};
use oxc_span::{GetSpan, Span};
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn prefer_export_from_diagnostic(span: Span, exported: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Use `export…from` to re-export `{exported}`.")).with_label(span)
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default, rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferExportFromOptions {
    ignore_used_variables: bool,
}

#[derive(Debug, Default, Clone)]
pub struct PreferExportFrom(PreferExportFromOptions);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer `export…from` when re-exporting imported bindings.
    ///
    /// ### Why is this bad?
    ///
    /// `export … from` is shorter and makes re-exports clearer than importing
    /// and then exporting in separate statements.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// import foo from "foo";
    /// export { foo };
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// export { default as foo } from "foo";
    /// ```
    PreferExportFrom,
    unicorn,
    style,
    config = PreferExportFromOptions
);

impl Rule for PreferExportFrom {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value::<DefaultRuleConfig<PreferExportFromOptions>>(value)
            .map(DefaultRuleConfig::into_inner)
            .map(Self)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let import_bindings = collect_import_bindings(ctx);
        if import_bindings.is_empty() {
            return;
        }

        let skipped_import_declarations = if self.0.ignore_used_variables {
            collect_skipped_import_declarations(&import_bindings, ctx)
        } else {
            FxHashSet::default()
        };

        for export_usage in collect_export_usages(ctx) {
            let Some(import_binding) = import_bindings.get(&export_usage.symbol_id) else {
                continue;
            };
            if skipped_import_declarations.contains(&import_binding.declaration_span) {
                continue;
            }
            if import_binding.import_kind == ImportKind::Namespace && export_usage.is_default_export
            {
                continue;
            }

            ctx.diagnostic(prefer_export_from_diagnostic(export_usage.span, &export_usage.text));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImportKind {
    Default,
    Namespace,
    Named,
}

#[derive(Debug, Clone, Copy)]
struct ImportBinding {
    symbol_id: SymbolId,
    import_kind: ImportKind,
    declaration_span: Span,
}

#[derive(Debug, Clone)]
struct ExportUsage {
    symbol_id: SymbolId,
    span: Span,
    text: String,
    is_default_export: bool,
}

fn collect_import_bindings(ctx: &LintContext<'_>) -> FxHashMap<SymbolId, ImportBinding> {
    let mut local_name_counts = FxHashMap::<String, usize>::default();
    for statement in &ctx.nodes().program().body {
        let Statement::ImportDeclaration(import_declaration) = statement else {
            continue;
        };
        let Some(specifiers) = &import_declaration.specifiers else {
            continue;
        };

        for specifier in specifiers {
            let local_name = match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => {
                    specifier.local.name.as_str()
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                    specifier.local.name.as_str()
                }
                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                    specifier.local.name.as_str()
                }
            };
            *local_name_counts.entry(local_name.to_string()).or_default() += 1;
        }
    }

    let mut bindings = FxHashMap::<SymbolId, ImportBinding>::default();

    for statement in &ctx.nodes().program().body {
        let Statement::ImportDeclaration(import_declaration) = statement else {
            continue;
        };
        let Some(specifiers) = &import_declaration.specifiers else {
            continue;
        };
        if specifiers.is_empty() {
            continue;
        }

        let mut declaration_bindings = Vec::with_capacity(specifiers.len());
        let mut skip_declaration = false;

        for specifier in specifiers {
            let (local_name, symbol_id, import_kind) = match specifier {
                oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    specifier.local.symbol_id.get(),
                    ImportKind::Named,
                ),
                oxc_ast::ast::ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    specifier.local.symbol_id.get(),
                    ImportKind::Default,
                ),
                oxc_ast::ast::ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => (
                    specifier.local.name.as_str(),
                    specifier.local.symbol_id.get(),
                    ImportKind::Namespace,
                ),
            };

            if local_name_counts.get(local_name).copied().unwrap_or_default() > 1 {
                skip_declaration = true;
                break;
            }

            let Some(symbol_id) = symbol_id else {
                skip_declaration = true;
                break;
            };

            declaration_bindings.push((symbol_id, import_kind));
        }

        if skip_declaration {
            continue;
        }

        for (symbol_id, import_kind) in declaration_bindings {
            bindings.insert(
                symbol_id,
                ImportBinding { symbol_id, import_kind, declaration_span: import_declaration.span },
            );
        }
    }

    bindings
}

fn collect_skipped_import_declarations(
    import_bindings: &FxHashMap<SymbolId, ImportBinding>,
    ctx: &LintContext<'_>,
) -> FxHashSet<Span> {
    let mut skipped = FxHashSet::default();

    for binding in import_bindings.values() {
        if ctx
            .scoping()
            .get_resolved_references(binding.symbol_id)
            .any(|reference| !is_reexport_reference(binding.import_kind, reference.node_id(), ctx))
        {
            skipped.insert(binding.declaration_span);
        }
    }

    skipped
}

fn is_reexport_reference(
    import_kind: ImportKind,
    reference_node_id: NodeId,
    ctx: &LintContext<'_>,
) -> bool {
    let node = ctx.nodes().get_node(reference_node_id);
    let AstKind::IdentifierReference(_) = node.kind() else {
        return false;
    };

    let parent = ctx.nodes().parent_node(node.id());
    match parent.kind() {
        AstKind::ExportDefaultDeclaration(_) => import_kind != ImportKind::Namespace,
        AstKind::ExportSpecifier(specifier) => {
            !(import_kind == ImportKind::Namespace && is_default_export_name(&specifier.exported))
        }
        AstKind::VariableDeclarator(_) => {
            exported_usage_from_exported_const_assignment(reference_node_id, ctx).is_some()
        }
        _ => false,
    }
}

fn collect_export_usages(ctx: &LintContext<'_>) -> Vec<ExportUsage> {
    let mut usages = Vec::new();

    for statement in &ctx.nodes().program().body {
        match statement {
            Statement::ExportDefaultDeclaration(export_default) => {
                let ExportDefaultDeclarationKind::Identifier(identifier_reference) =
                    &export_default.declaration
                else {
                    continue;
                };

                let Some(symbol_id) =
                    symbol_id_of_reference(identifier_reference.reference_id(), ctx)
                else {
                    continue;
                };

                usages.push(ExportUsage {
                    symbol_id,
                    span: export_default.span,
                    text: "default".to_string(),
                    is_default_export: true,
                });
            }
            Statement::ExportNamedDeclaration(export_named) => {
                if export_named.source.is_some() {
                    continue;
                }

                if export_named.declaration.is_none() {
                    for specifier in &export_named.specifiers {
                        let ModuleExportName::IdentifierReference(local_reference) =
                            &specifier.local
                        else {
                            continue;
                        };

                        let Some(symbol_id) =
                            symbol_id_of_reference(local_reference.reference_id(), ctx)
                        else {
                            continue;
                        };

                        usages.push(ExportUsage {
                            symbol_id,
                            span: specifier.span,
                            text: specifier
                                .exported
                                .span()
                                .source_text(ctx.source_text())
                                .to_string(),
                            is_default_export: is_default_export_name(&specifier.exported),
                        });
                    }
                }

                if let Some(declaration) = &export_named.declaration
                    && let oxc_ast::ast::Declaration::VariableDeclaration(variable_declaration) =
                        declaration
                    && variable_declaration.kind == VariableDeclarationKind::Const
                    && variable_declaration.declarations.len() == 1
                    && let Some(export_usage) =
                        exported_usage_from_exported_const_assignment_in_declaration(
                            export_named.span,
                            &variable_declaration.declarations[0],
                            ctx,
                        )
                {
                    usages.push(export_usage);
                }
            }
            _ => {}
        }
    }

    usages
}

fn exported_usage_from_exported_const_assignment(
    reference_node_id: NodeId,
    ctx: &LintContext<'_>,
) -> Option<ExportUsage> {
    let reference_node = ctx.nodes().get_node(reference_node_id);
    let AstKind::IdentifierReference(reference_ident) = reference_node.kind() else {
        return None;
    };

    let variable_declarator_node = ctx.nodes().parent_node(reference_node.id());
    let AstKind::VariableDeclarator(variable_declarator) = variable_declarator_node.kind() else {
        return None;
    };

    let variable_declaration_node = ctx.nodes().parent_node(variable_declarator_node.id());
    let AstKind::VariableDeclaration(variable_declaration) = variable_declaration_node.kind()
    else {
        return None;
    };
    if variable_declaration.kind != VariableDeclarationKind::Const
        || variable_declaration.declarations.len() != 1
    {
        return None;
    }

    let export_named_node = ctx.nodes().parent_node(variable_declaration_node.id());
    let AstKind::ExportNamedDeclaration(export_named_declaration) = export_named_node.kind() else {
        return None;
    };
    if export_named_declaration.source.is_some()
        || !matches!(
            export_named_declaration.declaration.as_ref(),
            Some(oxc_ast::ast::Declaration::VariableDeclaration(_))
        )
    {
        return None;
    }

    let imported_symbol_id = symbol_id_of_reference(reference_ident.reference_id(), ctx)?;

    exported_usage_from_exported_const_assignment_in_declaration(
        export_named_declaration.span,
        variable_declarator,
        ctx,
    )
    .filter(|usage| usage.symbol_id == imported_symbol_id)
}

fn exported_usage_from_exported_const_assignment_in_declaration(
    export_span: Span,
    variable_declarator: &oxc_ast::ast::VariableDeclarator<'_>,
    ctx: &LintContext<'_>,
) -> Option<ExportUsage> {
    let init = variable_declarator.init.as_ref()?;
    let oxc_ast::ast::Expression::Identifier(identifier_reference) = init.get_inner_expression()
    else {
        return None;
    };

    let imported_symbol_id = symbol_id_of_reference(identifier_reference.reference_id(), ctx)?;

    if variable_declarator.type_annotation.is_some() {
        return None;
    }

    let BindingPattern::BindingIdentifier(binding_identifier) = &variable_declarator.id else {
        return None;
    };

    let exported_symbol_id = binding_identifier.symbol_id.get()?;
    if ctx.scoping().get_resolved_references(exported_symbol_id).next().is_some() {
        return None;
    }

    Some(ExportUsage {
        symbol_id: imported_symbol_id,
        span: export_span,
        text: binding_identifier.name.as_str().to_string(),
        is_default_export: false,
    })
}

fn symbol_id_of_reference(reference_id: ReferenceId, ctx: &LintContext<'_>) -> Option<SymbolId> {
    ctx.scoping().get_reference(reference_id).symbol_id()
}

fn is_default_export_name(module_export_name: &ModuleExportName<'_>) -> bool {
    match module_export_name {
        ModuleExportName::IdentifierName(identifier_name) => identifier_name.name == "default",
        ModuleExportName::IdentifierReference(identifier_reference) => {
            identifier_reference.name == "default"
        }
        ModuleExportName::StringLiteral(string_literal) => string_literal.value == "default",
    }
}
#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r#"import "foo";"#,
        r#"import {} from "foo";"#,
        r#"import * as namespace from "foo";"#,
        r#"import defaultExport from "foo";"#,
        r#"import {named} from "foo";"#,
        "const named = import(foo);
            export {named};",
        r#"export * from "foo";"#,
        r#"export {default} from "foo";"#,
        r#"export {named} from "foo";"#,
        "const defaultExport = require('foo');
            export default defaultExport;",
        "import defaultExport from 'foo';
            export var variable = defaultExport;",
        "import defaultExport from 'foo';
            export let variable = defaultExport;",
        "import defaultExport from 'foo';
            export const variable = defaultExport;
            use(variable);",
        "import defaultExport from 'foo';
            export let variable = defaultExport;
            variable = 1;",
        "import * as namespace from 'foo';
            export default namespace;",
        "import * as namespace from 'foo';
            export {namespace as default};",
        "import defaultExport from 'foo';
            const variable = defaultExport;
            export {variable}",
        "import defaultExport from 'foo';
            export const {variable} = {variable: defaultExport};",
        "import {useDispatch as reduxUseDispatch} from 'react-redux'
            type MyDispatchType = Dispatch<MyActions>
            export const useDispatch: () => DispatchAllActions = reduxUseDispatch",
        r#"export type { bar, foo } from "foo";"#,
    ];

    let fail = vec![
        "import defaultExport from 'foo';
            export default defaultExport;",
        "import defaultExport from 'foo';
            export {defaultExport as default};",
        "import defaultExport from 'foo';
            export {defaultExport as named};",
        "import defaultExport from 'foo';
            export const variable = defaultExport;",
        "import {default as defaultExport} from 'foo';
            export default defaultExport;",
        "import {default as defaultExport} from 'foo';
            export {defaultExport as default};",
        "import {default as defaultExport} from 'foo';
            export {defaultExport as named};",
        "import defaultExport from 'foo';
            export const variable = defaultExport;",
        "import defaultExport from 'foo';
            defaultExport.bar = 1;
            export {defaultExport as named};
            export {defaultExport as default};
            export const variable = defaultExport;",
        "import {named} from 'foo';
            export default named;",
        "import {named} from 'foo';
            export {named as default};",
        "import {named} from 'foo';
            export {named as named};",
        "import {named} from 'foo';
            export {named as renamed};",
        "import {named} from 'foo';
            export const variable = named;",
        "import {named} from 'foo';
            named.bar = 1;
            export {named as named};
            export {named as default};
            export const variable = named;",
        "import * as namespace from 'foo';
            export {namespace as namespace};",
        "import * as namespace from 'foo';
            export {namespace as renamed};",
        "import * as namespace from 'foo';
            export const variable = namespace;",
        "import * as namespace from 'foo';
            namespace.bar = 1;
            export {namespace as named};
            export {namespace as default};
            export const variable = namespace;",
        "import {named1, named2} from 'foo';
            export {named1};",
        "import defaultExport, {named} from 'foo';
            export {defaultExport};",
        "import defaultExport, {named} from 'foo';
            export {named};",
        "import defaultExport, * as namespace from 'foo';
            export {defaultExport};",
        "import * as foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export {} from 'foo';",
        "import * as foo from 'foo';
            export {foo};
            export * from 'foo';",
        "import foo from 'foo';
            export {foo};
            export * as bar from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {bar} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {bar,} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export {} from 'foo';",
        "import foo from 'foo';
            export {foo};
            export * from 'foo';",
        "import {named1, named2} from 'foo';
            export {named1, named2};",
        "import {named} from 'foo';
            export {named as default, named};",
        "import {named, named as renamed} from 'foo';
            export {named, renamed};",
        "import defaultExport, {named1, named2} from 'foo';
            export {named1 as default};
            export {named2};
            export {defaultExport};",
        "import * as foo from 'foo';
            import * as bar from 'foo';
            export {foo, bar};",
        "import * as foo from 'foo';
            export {foo, foo as bar};",
        "import defaultExport from 'foo';
            export * from 'foo';
            export default defaultExport;",
        "import defaultExport from 'foo';
            export {named} from 'foo';
            export * from 'foo';
            export default defaultExport;",
        "import defaultExport from './foo.js';
            export {named} from './foo.js';
            export default defaultExport;",
        "import defaultExport from './foo.js';
            export {named} from './foo.js?query';
            export default defaultExport;",
        "import * as namespace from 'foo';
            export default namespace;
            export {namespace};",
        "import * as namespace from 'foo';
            export {namespace};
            export default namespace;",
        "import {'foo' as foo} from 'foo';
            export default foo;",
        "import {'foo' as foo} from 'foo';
            export {foo};",
        "import {'foo' as foo} from 'foo';
            export const bar = foo;",
        "import {'foo' as foo} from 'foo';
            export {foo as 'foo'};",
        r#"import {'foo' as foo} from 'foo';
            export {foo as "foo"};"#,
        r#"import {'qu\\\\u{20}x' as foo} from 'foo';
            export {foo as "qu x"};"#,
        r#"import {'qu\\\\nx' as foo} from 'foo';
            export {foo as "qu\\\\u000ax"};"#,
        "import {'default' as foo} from 'foo';
            export {foo};",
        "import {'default' as foo} from 'foo';
            export default foo;",
        "import {'*' as foo} from 'foo';
            export {foo};",
        r#"import { foo } from "foo";
            export { foo };
            export type { bar } from "foo";"#,
        r#"import { foo } from "foo";
            export { foo };
            export { type bar } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
        r#"import type { foo } from "foo";
            export type { foo };
            export type { bar } from "foo";"#,
        r#"import { foo } from 'foo';
            export { foo };
            export { baz } from "foo";
            export { type bar } from "foo";"#,
        r#"import type { foo } from "foo";
            export type { foo };
            export { type bar } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export type { bar } from "foo";
            export { baz } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { baz } from "foo";
            export type { bar } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { type bar } from "foo";
            export { baz } from "foo";"#,
        r#"import type { foo } from 'foo';
            export type { foo };
            export { baz } from "foo";
            export { type bar } from "foo";"#,
        "import { type foo } from 'foo';
            export type { foo };",
        "import { foo } from 'foo';
            export type { foo };",
        "import type { foo } from 'foo';
            export { type foo };",
        r#"import type foo from "foo";
            export default foo"#,
        "import {type foo} from 'foo';
            export {type foo as bar};",
        "import {type foo} from 'foo';
            export {type foo as bar};
            export {type bar} from 'foo';",
        "import {type foo as bar} from 'foo';
            export {type bar as baz};",
        "import {type foo as foo} from 'foo';
            export {type foo as bar};",
        "import {type foo as bar} from 'foo';
            export {type bar as bar};",
        "import {type foo as bar} from 'foo';
            export {type bar as foo};",
        "import json from './foo.json' assert { type: 'json' };
            export default json;",
        "import * as json from './foo.json' assert { type: 'json' };
            export {json};",
        "import {foo} from './foo.json' assert { type: 'unknown' };
            export {foo};
            export {bar} from './foo.json';",
        "import {foo} from './foo.json';
            export {foo};
            export {bar} from './foo.json' assert { type: 'unknown' };",
        "import json from './foo.json' with { type: 'json' };
            export default json;",
        "import * as json from './foo.json' with { type: 'json' };
            export {json};",
        "import {foo} from './foo.json' with { type: 'unknown' };
            export {foo};
            export {bar} from './foo.json';",
        "import {foo} from './foo.json';
            export {foo};
            export {bar} from './foo.json' with { type: 'unknown' };",
    ];

    Tester::new(PreferExportFrom::NAME, PreferExportFrom::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_ignore_used_variables_option() {
    use crate::tester::Tester;

    let pass = vec![
        (
            "import defaultExport from 'foo';
            use(defaultExport);
            export default defaultExport;",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport from 'foo';
            use(defaultExport);
            export {defaultExport};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {named} from 'foo';
            use(named);
            export {named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {named} from 'foo';
            use(named);
            export default named;",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import * as namespace from 'foo';
            use(namespace);
            export {namespace};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import * as namespace from 'foo';
            use(namespace);
            export default namespace;",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import * as namespace from 'foo';
            export {namespace as default};
            export {namespace as named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import * as namespace from 'foo';
            export default namespace;
            export {namespace as named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named} from 'foo';
            use(defaultExport);
            export {named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named} from 'foo';
            use(named);
            export {defaultExport};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {named1, named2} from 'foo';
            use(named1);
            export {named2};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named1, named2} from 'foo';
            use(defaultExport);
            export {named1, named2};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named1, named2} from 'foo';
            use(named1);
            export {defaultExport, named2};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
    ];

    let fail = vec![
        (
            "import defaultExport from 'foo';
            export {defaultExport as default};
            export {defaultExport as named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {named} from 'foo';
            export {named as default};
            export {named as named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {named} from 'foo';
            export default named;
            export {named as named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named} from 'foo';
            export default defaultExport;
            export {named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport, {named} from 'foo';
            export {defaultExport as default, named};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import defaultExport from 'foo';
            export const variable = defaultExport;",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
        (
            "import {notUsedNotExported, exported} from 'foo';
            export {exported};",
            Some(serde_json::json!([{ "ignoreUsedVariables": true }])),
        ),
    ];

    Tester::new(PreferExportFrom::NAME, PreferExportFrom::PLUGIN, pass, fail)
        .with_snapshot_suffix("ignore_used_variables")
        .test_and_snapshot();
}
