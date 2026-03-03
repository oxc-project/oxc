use oxc_ast::{
    AstKind,
    ast::{BindingPattern, ObjectPattern},
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_module_graph::NormalModule;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_export(span: Span, specifier_name: &str, namespace_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{specifier_name:?} not found in imported namespace {namespace_name:?}."
    ))
    .with_label(span)
}

fn no_export_in_deeply_imported_namespace(
    span: Span,
    specifier_name: &str,
    namespace_name: &str,
) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "{specifier_name:?} not found in deeply imported namespace {namespace_name:?}."
    ))
    .with_label(span)
}

fn computed_reference(span: Span, namespace_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!(
        "Unable to validate computed reference to imported namespace {namespace_name:?}."
    ))
    .with_label(span)
}

fn assignment(span: Span, namespace_name: &str) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Assignment to member of namespace {namespace_name:?}.'"))
        .with_label(span)
}

// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/namespace.md>
#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Namespace {
    /// Whether to allow computed references to an imported namespace.
    allow_computed: bool,
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Enforces names exist at the time they are dereferenced, when imported as
    /// a full namespace (i.e. `import * as foo from './foo'; foo.bar();` will
    /// report if bar is not exported by `./foo.`).  Will report at the import
    /// declaration if there are no exported names found.  Also, will report for
    /// computed references (i.e. `foo["bar"]()`).  Reports on assignment to a
    /// member of an imported namespace.
    ///
    /// ### Why is this bad?
    ///
    /// Dereferencing a name that does not exist can lead to runtime errors and
    /// unexpected behavior in your code. It makes the code less reliable and
    /// harder to maintain, as it may not be clear which names are valid. This
    /// rule helps ensure that all referenced names are defined, improving
    /// the clarity and robustness of your code.
    ///
    /// ### Examples
    ///
    /// Given
    /// ```javascript
    /// // ./foo.js
    /// export const bar = "I'm bar";
    /// ```
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// // ./qux.js
    /// import * as foo from './foo';
    /// foo.notExported(); // Error: notExported is not exported
    ///
    /// // Assignment to a member of an imported namespace
    /// foo.bar = "new value"; // Error: bar cannot be reassigned
    ///
    /// // Computed reference to a non-existent export
    /// const method = "notExported";
    /// foo[method](); // Error: notExported does not exist
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// // ./baz.js
    /// import * as foo from './foo';
    /// console.log(foo.bar); // Valid: bar is exported
    ///
    /// // Computed reference
    /// const method = "bar";
    /// foo[method](); // Valid: method refers to an exported function
    /// ```
    Namespace,
    import,
    correctness,
    config = Namespace,
);

impl Rule for Namespace {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext<'_>) {
        let Some(module) = ctx.current_module() else {
            return;
        };
        let Some(graph) = ctx.module_graph() else {
            return;
        };

        if !module.has_module_syntax {
            return;
        }

        for import in module.named_imports.values() {
            let is_namespace = import.imported_name.as_str() == "*";
            if !is_namespace {
                // Also handle the case where a named import refers to a
                // re-exported namespace (e.g. `import { b } from './a'` where
                // b.js does `export * as b from './c'`).
                let Some(record) = module.import_records.get(import.record_idx.index()) else {
                    continue;
                };
                let Some(target_idx) = record.resolved_module else {
                    continue;
                };
                let Some(loaded) = ctx.resolve_module(target_idx) else {
                    continue;
                };
                let Some(ns_source) =
                    get_namespace_module_request(import.imported_name.as_str(), loaded, ctx)
                else {
                    continue;
                };
                let Some(ns_module) = ns_source else {
                    continue;
                };
                if !ns_module.has_module_syntax {
                    continue;
                }

                // Look up the local binding by name: graph stores the local
                // name, then find the matching SymbolId in scoping.
                let local_name = graph.symbol_name(import.local_symbol);
                let Some(symbol_id) = ctx.scoping().get_root_binding(local_name.into()) else {
                    continue;
                };

                let source_str = record.specifier.to_string();
                let local_name = local_name.to_string();
                ctx.scoping().get_resolved_references(symbol_id).for_each(|reference| {
                    let parent = ctx.nodes().parent_node(reference.node_id());
                    let name = local_name.as_str();

                    match parent.kind() {
                        member if member.is_member_expression_kind() => {
                            let parent_kind = ctx.nodes().parent_kind(parent.id());
                            let is_assignment = match parent_kind {
                                AstKind::AssignmentExpression(assign_expr) => {
                                    assign_expr.left.span() == parent.span()
                                }
                                _ => false,
                            };
                            if is_assignment
                                || matches!(parent_kind, AstKind::IdentifierReference(_))
                            {
                                ctx.diagnostic(assignment(member.span(), name));
                            }

                            if !self.allow_computed
                                && matches!(member, AstKind::ComputedMemberExpression(_))
                            {
                                return ctx.diagnostic(computed_reference(member.span(), name));
                            }

                            check_deep_namespace_for_node(
                                parent,
                                &source_str,
                                vec![name.to_string()].as_slice(),
                                ns_module,
                                ctx,
                            );
                        }
                        AstKind::JSXMemberExpression(expr) => {
                            check_binding_exported_mg(
                                &expr.property.name,
                                || no_export(expr.property.span, &expr.property.name, &source_str),
                                ns_module,
                                ctx,
                            );
                        }
                        AstKind::VariableDeclarator(decl) => {
                            let BindingPattern::ObjectPattern(pattern) = &decl.id else {
                                return;
                            };
                            check_deep_namespace_for_object_pattern(
                                pattern,
                                &source_str,
                                &[name.to_string()],
                                ns_module,
                                ctx,
                            );
                        }
                        _ => {}
                    }
                });

                continue;
            }

            // Namespace import: `import * as ns from './foo'`
            let Some(record) = module.import_records.get(import.record_idx.index()) else {
                continue;
            };
            let Some(target_idx) = record.resolved_module else {
                continue;
            };
            let Some(remote) = ctx.resolve_module(target_idx) else {
                continue;
            };
            if !remote.has_module_syntax {
                continue;
            }

            let source = record.specifier.to_string();

            // Look up the local binding by name: graph stores the local
            // name, then find the matching SymbolId in scoping.
            let local_name_str = graph.symbol_name(import.local_symbol);
            let Some(symbol_id) = ctx.scoping().get_root_binding(local_name_str.into()) else {
                continue;
            };
            let local_name = local_name_str.to_string();
            ctx.scoping().get_resolved_references(symbol_id).for_each(|reference| {
                let parent = ctx.nodes().parent_node(reference.node_id());
                let name = local_name.as_str();

                match parent.kind() {
                    member if member.is_member_expression_kind() => {
                        let parent_kind = ctx.nodes().parent_kind(parent.id());
                        let is_assignment = match parent_kind {
                            AstKind::AssignmentExpression(assign_expr) => {
                                assign_expr.left.span() == parent.span()
                            }
                            _ => false,
                        };
                        if is_assignment || matches!(parent_kind, AstKind::IdentifierReference(_)) {
                            ctx.diagnostic(assignment(member.span(), name));
                        }

                        if !self.allow_computed
                            && matches!(member, AstKind::ComputedMemberExpression(_))
                        {
                            return ctx.diagnostic(computed_reference(member.span(), name));
                        }

                        check_deep_namespace_for_node(
                            parent,
                            &source,
                            vec![name.to_string()].as_slice(),
                            remote,
                            ctx,
                        );
                    }
                    AstKind::JSXMemberExpression(expr) => {
                        check_binding_exported_mg(
                            &expr.property.name,
                            || no_export(expr.property.span, &expr.property.name, &source),
                            remote,
                            ctx,
                        );
                    }
                    AstKind::VariableDeclarator(decl) => {
                        let BindingPattern::ObjectPattern(pattern) = &decl.id else {
                            return;
                        };

                        check_deep_namespace_for_object_pattern(
                            pattern,
                            &source,
                            &[name.to_string()],
                            remote,
                            ctx,
                        );
                    }
                    _ => {}
                }
            });
        }
    }
}

/// If the name is a namespace object in imported module, return the resolved
/// `NormalModule` for that namespace. Returns `Some(None)` when the name
/// is found as a namespace re-export but the target cannot be resolved.
fn get_namespace_module_request<'a>(
    name: &str,
    module: &'a NormalModule,
    ctx: &'a LintContext<'_>,
) -> Option<Option<&'a NormalModule>> {
    // Check indirect export entries for `export * as name from '...'`.
    for entry in &module.indirect_export_entries {
        if entry.imported_name.as_str() == "*" && entry.exported_name.as_str() == name {
            let target = entry.resolved_module.and_then(|idx| ctx.resolve_module(idx));
            return Some(target);
        }
    }

    let graph = ctx.module_graph()?;

    // Check if name is imported as a namespace in this module and then
    // re-exported (via indirect_export_entries or named_exports).
    for import in module.named_imports.values() {
        if import.imported_name.as_str() == "*" {
            // Check if re-exported via indirect_export_entries.
            for entry in &module.indirect_export_entries {
                if entry.exported_name.as_str() == name {
                    let target = entry.resolved_module.and_then(|idx| ctx.resolve_module(idx));
                    return Some(target);
                }
            }

            // Check if re-exported via named_exports (e.g., `import * as b from './b'; export { b }`).
            let import_local_name = graph.symbol_name(import.local_symbol);
            if module.named_exports.contains_key(name) && import_local_name == name {
                let record = module.import_records.get(import.record_idx.index())?;
                let target = record.resolved_module.and_then(|idx| ctx.resolve_module(idx));
                return Some(target);
            }
        }
    }

    None
}

fn check_deep_namespace_for_node<'a>(
    node: &AstNode,
    source: &str,
    namespaces: &[String],
    module: &'a NormalModule,
    ctx: &'a LintContext<'_>,
) -> Option<()> {
    let (span, name) = match node.kind() {
        AstKind::StaticMemberExpression(mem_expr) => mem_expr.static_property_info(),
        AstKind::ComputedMemberExpression(computed_expr) => computed_expr.static_property_info()?,
        _ => return None,
    };

    if let Some(ns_target) = get_namespace_module_request(name, module, ctx) {
        if let Some(ns_module) = ns_target {
            let parent_node = ctx.nodes().parent_node(node.id());
            let mut namespaces = namespaces.to_owned();
            namespaces.push(name.into());
            check_deep_namespace_for_node(parent_node, source, &namespaces, ns_module, ctx);
        }
    } else {
        check_binding_exported_mg(
            name,
            || {
                if namespaces.len() > 1 {
                    no_export_in_deeply_imported_namespace(span, name, &namespaces.join("."))
                } else {
                    no_export(span, name, source)
                }
            },
            module,
            ctx,
        );
    }

    None
}

fn check_deep_namespace_for_object_pattern<'a>(
    pattern: &ObjectPattern,
    source: &str,
    namespaces: &[String],
    module: &'a NormalModule,
    ctx: &'a LintContext<'_>,
) {
    for property in &pattern.properties {
        let Some(name) = property.key.name() else {
            continue;
        };

        if let BindingPattern::ObjectPattern(pattern) = &property.value
            && let Some(ns_target) = get_namespace_module_request(&name, module, ctx)
            && let Some(ns_module) = ns_target
        {
            let mut next_namespaces = namespaces.to_owned();
            next_namespaces.push(name.to_string());

            check_deep_namespace_for_object_pattern(
                pattern,
                source,
                next_namespaces.as_slice(),
                ns_module,
                ctx,
            );
            continue;
        }

        check_binding_exported_mg(
            &name,
            || {
                if namespaces.len() > 1 {
                    no_export_in_deeply_imported_namespace(
                        property.key.span(),
                        &name,
                        &namespaces.join("."),
                    )
                } else {
                    no_export(property.key.span(), &name, source)
                }
            },
            module,
            ctx,
        );
    }
}

fn check_binding_exported_mg(
    name: &str,
    get_diagnostic: impl FnOnce() -> OxcDiagnostic,
    module: &NormalModule,
    ctx: &LintContext<'_>,
) {
    if module.named_exports.contains_key(name)
        || module.resolved_exports.contains_key(name)
        || module.indirect_export_entries.iter().any(|e| e.exported_name.as_str() == name)
    {
        return;
    }
    let _ = ctx; // suppress unused warning
    ctx.diagnostic(get_diagnostic());
}

#[test]
fn test() {
    use serde_json::json;

    use crate::tester::Tester;

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
        // Issue: <https://github.com/oxc-project/oxc/issues/7696>
        (r"import * as acorn from 'acorn'; acorn.parse()", None),
        // https://github.com/oxc-project/oxc/issues/10318
        (
            r#"import * as lib from "./typescript-export-enum"; export const bar = lib.Foo.BAR;"#,
            None,
        ),
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

    Tester::new(Namespace::NAME, Namespace::PLUGIN, pass, fail)
        .change_rule_path("index.js")
        .with_import_plugin(true)
        .test_and_snapshot();
}
