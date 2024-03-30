use std::sync::Arc;

use oxc_ast::{
    ast::{BindingPatternKind, ObjectPattern},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ModuleRecord};
use oxc_span::{CompactStr, GetSpan, Span};
use oxc_syntax::module_record::{ExportExportName, ExportImportName, ImportImportName};

use crate::{context::LintContext, rule::Rule};

#[derive(Debug, Error, Diagnostic)]
enum NamespaceDiagnostic {
    #[error("eslint-plugin-import(namespace): {1:?} not found in imported namespace {2:?}.")]
    #[diagnostic(severity(warning))]
    NoExport(#[label] Span, CompactStr, String),
    #[error(
        "eslint-plugin-import(namespace): {1:?} not found in deeply imported namespace {2:?}."
    )]
    #[diagnostic(severity(warning))]
    NoExportInDeeplyImportedNamespace(#[label] Span, CompactStr, String),
    #[error("eslint-plugin-import(namespace): Unable to validate computed reference to imported namespace {1:?}
    .")]
    #[diagnostic(severity(warning))]
    ComputedReference(#[label] Span, CompactStr),
    #[error("eslint-plugin-import(namespace): Assignment to member of namespace {1:?}.'")]
    #[diagnostic(severity(warning))]
    Assignment(#[label] Span, CompactStr),
}

/// <https://github.com/import-js/eslint-plugin-import/blob/main/docs/rules/namespace.md>
#[derive(Debug, Default, Clone)]
pub struct Namespace {
    allow_computed: bool,
}

declare_oxc_lint!(
    /// ### What it does
    /// Enforces names exist at the time they are dereferenced, when imported as a full namespace (i.e. import * as foo from './foo'; foo.bar(); will report if bar is not exported by ./foo.).
    /// Will report at the import declaration if there are no exported names found.
    /// Also, will report for computed references (i.e. foo["bar"]()).
    /// Reports on assignment to a member of an imported namespace.
    Namespace,
    nursery
);

impl Rule for Namespace {
    fn from_configuration(value: serde_json::Value) -> Self {
        let obj = value.get(0);
        Self {
            allow_computed: obj
                .and_then(|v| v.get("allowComputed"))
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }
    fn run_once(&self, ctx: &LintContext<'_>) {
        let module_record = ctx.semantic().module_record();
        module_record.import_entries.iter().for_each(|entry| {
            let (source, module) = match &entry.import_name {
                ImportImportName::NamespaceObject => {
                    let source = entry.module_request.name();
                    if let Some(module) = module_record.loaded_modules.get(source) {
                        (source.to_string(), Arc::clone(module.value()))
                    } else {
                        return;
                    }
                }
                ImportImportName::Name(name) => {
                    let Some(loaded_module) =
                        module_record.loaded_modules.get(entry.module_request.name())
                    else {
                        return;
                    };
                    let Some(source) = get_module_request_name(name.name(), &loaded_module) else {
                        return;
                    };

                    let Some(loaded_module) =
                        &loaded_module.loaded_modules.get(&CompactStr::from(source.clone()))
                    else {
                        return;
                    };

                    (source, Arc::clone(loaded_module.value()))
                }
                ImportImportName::Default(_) => {
                    // TODO: Hard to confirm if it's a namespace object
                    return;
                }
            };

            if module.not_esm {
                return;
            }

            let Some(symbol_id) =
                ctx.semantic().symbols().get_symbol_id_from_span(&entry.local_name.span())
            else {
                return;
            };

            ctx.symbols().get_resolved_references(symbol_id).for_each(|reference| {
                if let Some(node) = ctx.nodes().parent_node(reference.node_id()) {
                    let name = entry.local_name.name();

                    match node.kind() {
                        AstKind::MemberExpression(member) => {
                            if matches!(
                                ctx.nodes().parent_kind(node.id()),
                                Some(AstKind::SimpleAssignmentTarget(_))
                            ) {
                                ctx.diagnostic(NamespaceDiagnostic::Assignment(
                                    member.span(),
                                    name.clone(),
                                ));
                            };

                            if !self.allow_computed && member.is_computed() {
                                return ctx.diagnostic(NamespaceDiagnostic::ComputedReference(
                                    member.span(),
                                    name.clone(),
                                ));
                            }

                            check_deep_namespace_for_node(
                                node,
                                &source,
                                vec![entry.local_name.name().clone()].as_slice(),
                                &module,
                                ctx,
                            );
                        }
                        AstKind::JSXMemberExpressionObject(_) => {
                            if let Some(AstKind::JSXMemberExpression(expr)) =
                                ctx.nodes().parent_kind(node.id())
                            {
                                check_binding_exported(
                                    &expr.property.name,
                                    || {
                                        NamespaceDiagnostic::NoExport(
                                            expr.property.span,
                                            expr.property.name.to_compact_str(),
                                            source.clone(),
                                        )
                                    },
                                    &module,
                                    ctx,
                                );
                            }
                        }
                        AstKind::VariableDeclarator(decl) => {
                            let BindingPatternKind::ObjectPattern(pattern) = &decl.id.kind else {
                                return;
                            };

                            check_deep_namespace_for_object_pattern(
                                pattern,
                                &source,
                                vec![entry.local_name.name().clone()].as_slice(),
                                &module,
                                ctx,
                            );
                        }
                        _ => {}
                    }
                }
            });
        });
    }
}

/// If the name is a namespace object in imported module, return the module request name.
///
/// For example
/// ```ts
/// // ./a.js
/// import { b } from './b';
///
/// // ./b.js
/// export * as b from './c';
/// ```
/// b is a namespace in b.js so the return value is Some("./c")
///
fn get_module_request_name(name: &str, module_record: &ModuleRecord) -> Option<String> {
    if let Some(entry) =
        module_record.indirect_export_entries.iter().find(|e| match &e.import_name {
            ExportImportName::All => {
                if let ExportExportName::Name(name_span) = &e.export_name {
                    return name_span.name().as_str() == name;
                }

                false
            }
            ExportImportName::Name(name_span) => {
                return name_span.name().as_str() == name
                    && module_record.import_entries.iter().any(|entry| {
                        entry.local_name.name().as_str() == name
                            && entry.import_name.is_namespace_object()
                    })
            }
            _ => false,
        })
    {
        return entry.module_request.as_ref().map(|name| name.name().to_string());
    };

    return module_record
        .import_entries
        .iter()
        .find(|entry| {
            entry.local_name.name().as_str() == name && entry.import_name.is_namespace_object()
        })
        .map(|entry| entry.module_request.name().to_string());
}

fn check_deep_namespace_for_node(
    node: &AstNode,
    source: &str,
    namespaces: &[CompactStr],
    module: &Arc<ModuleRecord>,
    ctx: &LintContext<'_>,
) {
    if let AstKind::MemberExpression(expr) = node.kind() {
        let Some((span, name)) = expr.static_property_info() else {
            return;
        };

        if let Some(module_source) = get_module_request_name(name, module) {
            let Some(parent_node) = ctx.nodes().parent_node(node.id()) else {
                return;
            };

            let mut namespaces = namespaces.to_owned();
            namespaces.push(name.into());
            check_deep_namespace_for_node(
                parent_node,
                source,
                namespaces.as_slice(),
                module.loaded_modules.get(&CompactStr::from(module_source)).unwrap().value(),
                ctx,
            );
        } else {
            check_binding_exported(
                name,
                || {
                    if namespaces.len() > 1 {
                        NamespaceDiagnostic::NoExportInDeeplyImportedNamespace(
                            span,
                            name.into(),
                            namespaces.join("."),
                        )
                    } else {
                        NamespaceDiagnostic::NoExport(span, name.into(), source.to_string())
                    }
                },
                module,
                ctx,
            );
        }
    }
}

fn check_deep_namespace_for_object_pattern(
    pattern: &ObjectPattern,
    source: &str,
    namespaces: &[CompactStr],
    module: &Arc<ModuleRecord>,
    ctx: &LintContext<'_>,
) {
    for property in &pattern.properties {
        let Some(name) = property.key.name() else {
            continue;
        };

        if let BindingPatternKind::ObjectPattern(pattern) = &property.value.kind {
            if let Some(module_source) = get_module_request_name(&name, module) {
                let mut next_namespaces = namespaces.to_owned();
                next_namespaces.push(name.clone());
                check_deep_namespace_for_object_pattern(
                    pattern,
                    source,
                    next_namespaces.as_slice(),
                    module.loaded_modules.get(&CompactStr::from(module_source)).unwrap().value(),
                    ctx,
                );
                continue;
            }
        }

        check_binding_exported(
            &name,
            || {
                if namespaces.len() > 1 {
                    NamespaceDiagnostic::NoExportInDeeplyImportedNamespace(
                        property.key.span(),
                        name.clone(),
                        namespaces.join("."),
                    )
                } else {
                    NamespaceDiagnostic::NoExport(
                        property.key.span(),
                        name.clone(),
                        source.to_string(),
                    )
                }
            },
            module,
            ctx,
        );
    }
}

fn check_binding_exported(
    name: &str,
    get_diagnostic: impl FnOnce() -> NamespaceDiagnostic,
    module: &ModuleRecord,
    ctx: &LintContext<'_>,
) {
    if module.exported_bindings.contains_key(name)
        || (name == "default" && module.export_default.is_some())
        || module
            .exported_bindings_from_star_export
            .iter()
            .any(|entry| entry.value().contains(&CompactStr::from(name)))
    {
        return;
    }
    ctx.diagnostic(get_diagnostic());
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        (r#"import "./malformed.js""#, None),
        (r"import * as foo from './empty-folder';", None),
        (r#"import * as names from "./named-exports"; console.log((names.b).c);"#, None),
        (r#"import * as names from "./named-exports"; console.log(names.a);"#, None),
        (r#"import * as names from "./re-export-names"; console.log(names.foo);"#, None),
        (r"import * as elements from './jsx';", None),
        (
            r#"import * as foo from "./jsx/re-export.js";
        console.log(foo.jsxFoo);"#,
            None,
        ),
        (
            r#"import * as foo from "./jsx/bar/index.js";
        console.log(foo.Baz1);
        console.log(foo.Baz2);
        console.log(foo.Qux1);
        console.log(foo.Qux2);"#,
            None,
        ),
        (r"import * as foo from './common';", None),
        (r#"import * as names from "./named-exports"; const { a } = names"#, None),
        (r#"import * as names from "./named-exports"; const { d: c } = names"#, None),
        (
            r#"import * as names from "./named-exports";
        const { c } = foo,
        { length } = "names",
        alt = names;"#,
            None,
        ),
        (
            r#"import * as names from "./named-exports"; const { ExportedClass: { length } } = names"#,
            None,
        ),
        (
            r#"import * as names from "./named-exports"; function b(names) { const { c } = names }"#,
            None,
        ),
        (
            r#"import * as names from "./named-exports"; function b() { let names = null; const { c } = names }"#,
            None,
        ),
        (
            r#"import * as names from "./named-exports"; const x = function names() { const { c } = names }"#,
            None,
        ),
        (r#"export * as names from "./named-exports""#, None),
        // r#"export * as names from "./does-not-exist""#,
        // r#"import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Users)"#,
        (
            r#"function x() { console.log((names.b).c); } import * as names from "./named-exports";"#,
            None,
        ),
        (r"import * as names from './default-export';", None),
        (r"import * as names from './default-export'; console.log(names.default)", None),
        (r#"export * as names from "./default-export""#, None),
        // r#"export defport, * as names from "./default-export""#,
        (
            r"import * as names from './named-exports'; console.log(names['a']);",
            Some(json!([{ "allowComputed": true }])),
        ),
        (r"import * as names from './named-exports'; const {a, b, ...rest} = names;", None),
        (r"import * as ns from './re-export-common'; const {foo} = ns;", None),
        (r#"import * as Names from "./named-exports"; const Foo = <Names.a/>"#, None),
        // r#"import * as foo from "./typescript-declare-nested"
        // foo.bar.MyFunction()"#,
        // r#"import { foobar } from "./typescript-declare-interface""#,
        // r#"export * from "typescript/lib/typescript.d""#,
        // r#"export = function name() {}"#,
        // r#"for (let { foo, bar } of baz) {}"#,
        // r#"for (let [ foo, bar ] of baz) {}"#,
        // r#"const { x, y } = bar"#,
        // r#"const { x, y, ...z } = bar"#,
        // r#"let x; export { x }"#,
        // r#"let x; export { x as y }"#,
        // r#"export const x = null"#,
        // r#"export var x = null"#,
        // r#"export let x = null"#,
        // r#"export default x"#,
        // r#"export default class x {}"#,
        // r#"import json from "./data.json""#,
        // r#"import foo from "./foobar.json";"#,
        // r#"import foo from "./foobar";"#,
        // r#"import { foo } from "./issue-370-commonjs-namespace/bar""#,
        // r#"export * from "./issue-370-commonjs-namespace/bar""#,
        // r#"import * as a from "./commonjs-namespace/a"; a.b"#,
        // r#"import { foo } from "./ignore.invalid.extension""#,
        // r#"import * as color from './color';
        // export const getBackgroundFromColor = (color) => color.bg;
        // export const getExampleColor = () => color.example"#,
        // r#"import * as middle from './middle';

        // console.log(middle.myName);"#,
        // r#"import * as names from './default-export-string';"#,
        // r#"import * as names from './default-export-string'; console.log(names.default)"#,
        // r#"import * as names from './default-export-namespace-string';"#,
        // r#"import * as names from './default-export-namespace-string'; console.log(names.default)"#,
        (r#"import { "b" as b } from "./deep/a"; console.log(b.c.d.e)"#, None),
        (r#"import { "b" as b } from "./deep/a"; var {c:{d:{e}}} = b"#, None),
        (r#"import * as a from "./deep/a"; console.log(a.b.c.d.e)"#, None),
        (r#"import { b } from "./deep/a"; console.log(b.c.d.e)"#, None),
        (r#"import * as a from "./deep/a"; console.log(a.b.c.d.e.f)"#, None),
        (r#"import * as a from "./deep/a"; var {b:{c:{d:{e}}}} = a"#, None),
        (r#"import { b } from "./deep/a"; var {c:{d:{e}}} = b"#, None),
        (r#"import * as a from "./deep-es7/a"; console.log(a.b.c.d.e)"#, None),
        (r#"import { b } from "./deep-es7/a"; console.log(b.c.d.e)"#, None),
        (r#"import * as a from "./deep-es7/a"; console.log(a.b.c.d.e.f)"#, None),
        (r#"import * as a from "./deep-es7/a"; var {b:{c:{d:{e}}}} = a"#, None),
        (r#"import { b } from "./deep-es7/a"; var {c:{d:{e}}} = b"#, None),
        (r"import { a } from './oxc/indirect-export'; console.log(a.nothing)", None),
    ];

    let fail = vec![
        (r"import * as names from './named-exports'; console.log(names.c)", None),
        (r"import * as names from './named-exports'; console.log(names['a']);", None),
        (r"import * as foo from './bar'; foo.foo = 'y';", None),
        (r"import * as foo from './bar'; foo.x = 'y';", None),
        (r#"import * as names from "./named-exports"; const { c } = names"#, None),
        (r#"import * as names from "./named-exports"; function b() { const { c } = names }"#, None),
        (r#"import * as names from "./named-exports"; const { c: d } = names"#, None),
        (r#"import * as names from "./named-exports"; const { c: { d } } = names"#, None),
        // r#"import * as Endpoints from "./issue-195/Endpoints"; console.log(Endpoints.Foo)"#,
        // r#"import * as namespace from './malformed.js';"#,
        // TODO: Hard to confirm if it's a namespace object
        // r"import b from './deep/default'; console.log(b.e)",
        (r"console.log(names.c); import * as names from './named-exports';", None),
        (r"function x() { console.log(names.c) } import * as names from './named-exports';", None),
        (r#"import * as ree from "./re-export"; console.log(ree.default)"#, None),
        (r#"import * as Names from "./named-exports"; const Foo = <Names.e/>"#, None),
        (r#"import { "b" as b } from "./deep/a"; console.log(b.e)"#, None),
        (r#"import { "b" as b } from "./deep/a"; console.log(b.c.e)"#, None),
        (r#"import * as a from "./deep/a"; console.log(a.b.e)"#, None),
        (r#"import { b } from "./deep/a"; console.log(b.e)"#, None),
        (r#"import * as a from "./deep/a"; console.log(a.b.c.e)"#, None),
        (r#"import { b } from "./deep/a"; console.log(b.c.e)"#, None),
        (r#"import * as a from "./deep-es7/a"; console.log(a.b.e)"#, None),
        (r#"import { b } from "./deep-es7/a"; console.log(b.e)"#, None),
        (r#"import * as a from "./deep-es7/a"; console.log(a.b.c.e)"#, None),
        (r#"import { b } from "./deep-es7/a"; console.log(b.c.e)"#, None),
        (r#"import * as a from "./deep-es7/a"; var {b:{ e }} = a"#, None),
        (r#"import * as a from "./deep-es7/a"; var {b:{c:{ e }, e: { c }}} = a"#, None),
    ];

    Tester::new(Namespace::NAME, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
