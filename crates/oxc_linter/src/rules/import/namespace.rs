use std::sync::Arc;

use oxc_ast::{
    ast::{BindingPatternKind, ObjectPattern},
    AstKind,
};
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{GetSpan, Span};

use crate::{
    context::LintContext,
    module_record::{ExportExportName, ExportImportName, ImportImportName, ModuleRecord},
    rule::Rule,
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

/// <https://github.com/import-js/eslint-plugin-import/blob/v2.29.1/docs/rules/namespace.md>
#[derive(Debug, Default, Clone)]
pub struct Namespace {
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
    correctness
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
        let module_record = ctx.module_record();

        if !module_record.has_module_syntax {
            return;
        }

        let loaded_modules = module_record.loaded_modules.read().unwrap();

        for entry in &module_record.import_entries {
            let (source, module) = match &entry.import_name {
                ImportImportName::NamespaceObject => {
                    let source = entry.module_request.name();
                    if let Some(module) = loaded_modules.get(source) {
                        (source.to_string(), Arc::clone(module))
                    } else {
                        return;
                    }
                }
                ImportImportName::Name(name) => {
                    let Some(loaded_module) = loaded_modules.get(entry.module_request.name())
                    else {
                        return;
                    };
                    let Some(source) = get_module_request_name(name.name(), loaded_module) else {
                        return;
                    };

                    let loaded_module = loaded_module.loaded_modules.read().unwrap();
                    let Some(loaded_module) = loaded_module.get(source.as_str()) else {
                        return;
                    };

                    (source, Arc::clone(loaded_module))
                }
                ImportImportName::Default(_) => {
                    // TODO: Hard to confirm if it's a namespace object
                    return;
                }
            };

            if !module.has_module_syntax {
                return;
            }

            let Some(symbol_id) = ctx.scopes().get_root_binding(entry.local_name.name()) else {
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
                                ctx.diagnostic(assignment(member.span(), name));
                            };

                            if !self.allow_computed && member.is_computed() {
                                return ctx.diagnostic(computed_reference(member.span(), name));
                            }

                            check_deep_namespace_for_node(
                                node,
                                &source,
                                vec![entry.local_name.name().to_string()].as_slice(),
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
                                    || no_export(expr.property.span, &expr.property.name, &source),
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
                                &[entry.local_name.name().to_string()],
                                &module,
                                ctx,
                            );
                        }
                        _ => {}
                    }
                }
            });
        }
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
                    return name_span.name() == name;
                }

                false
            }
            ExportImportName::Name(name_span) => {
                name_span.name() == name
                    && module_record.import_entries.iter().any(|entry| {
                        entry.local_name.name() == name && entry.import_name.is_namespace_object()
                    })
            }
            _ => false,
        })
    {
        return entry.module_request.as_ref().map(|name| name.name().to_string());
    };

    module_record
        .import_entries
        .iter()
        .find(|entry| entry.local_name.name() == name && entry.import_name.is_namespace_object())
        .map(|entry| entry.module_request.name().to_string())
}

fn check_deep_namespace_for_node(
    node: &AstNode,
    source: &str,
    namespaces: &[String],
    module: &Arc<ModuleRecord>,
    ctx: &LintContext<'_>,
) -> Option<()> {
    let expr = node.kind().as_member_expression()?;
    let (span, name) = expr.static_property_info()?;

    if let Some(module_source) = get_module_request_name(name, module) {
        let parent_node = ctx.nodes().parent_node(node.id())?;
        let loaded_modules = module.loaded_modules.read().unwrap();
        let module_record = loaded_modules.get(module_source.as_str())?;
        let mut namespaces = namespaces.to_owned();
        namespaces.push(name.into());
        check_deep_namespace_for_node(parent_node, source, &namespaces, module_record, ctx);
    } else {
        check_binding_exported(
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

fn check_deep_namespace_for_object_pattern(
    pattern: &ObjectPattern,
    source: &str,
    namespaces: &[String],
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
                next_namespaces.push(name.to_string());

                let loaded_modules = module.loaded_modules.read().unwrap();
                check_deep_namespace_for_object_pattern(
                    pattern,
                    source,
                    next_namespaces.as_slice(),
                    loaded_modules.get(module_source.as_str()).unwrap(),
                    ctx,
                );
                continue;
            }
        }

        check_binding_exported(
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

fn check_binding_exported(
    name: &str,
    get_diagnostic: impl FnOnce() -> OxcDiagnostic,
    module: &ModuleRecord,
    ctx: &LintContext<'_>,
) {
    if module.exported_bindings.contains_key(name)
        || (name == "default" && module.export_default.is_some())
        || module
            .exported_bindings_from_star_export()
            .iter()
            .any(|(_, value)| value.iter().any(|s| s.as_str() == name))
    {
        return;
    }
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
