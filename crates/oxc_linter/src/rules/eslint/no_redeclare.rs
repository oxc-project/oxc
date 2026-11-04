use itertools::Itertools;
use javascript_globals::GLOBALS_BUILTIN;

use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolFlags;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn no_redeclare_diagnostic(name: &str, decl_span: Span, re_decl_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already defined."))
        .with_help("Use a different variable name or remove the duplicate declaration.")
        .with_labels([
            decl_span.label(format!("'{name}' is already defined.")),
            re_decl_span.label("It can not be redeclared here."),
        ])
}

fn no_redeclare_as_builtin_in_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already defined as a built-in global variable."))
        .with_help("Use a different variable name to avoid shadowing the built-in global.")
        .with_label(span)
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct NoRedeclare {
    /// When set `true`, it flags redeclaring built-in globals (e.g., `let Object = 1;`).
    builtin_globals: bool,
    /// When `true`, automatically merged declarations like namespace, class, etc. will be
    /// ignored when checking for redeclarations.
    ignore_declaration_merge: bool,
}

impl Default for NoRedeclare {
    fn default() -> Self {
        Self { builtin_globals: true, ignore_declaration_merge: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// This rule disallows redeclaring variables within the same scope, ensuring that each variable
    /// is declared only once. It helps avoid confusion and unintended behavior in code.
    ///
    /// ### Why is this bad?
    ///
    /// Redeclaring variables in the same scope can lead to unexpected behavior, overwriting existing values,
    /// and making the code harder to understand and maintain.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```javascript
    /// var a = 3;
    /// var a = 10;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```javascript
    /// var a = 3;
    /// a = 10;
    /// ```
    NoRedeclare,
    eslint,
    pedantic,
    recommended,
    config = NoRedeclare,
    version = "0.0.13",
    short_description = "Disallow variable redeclaration.",
);

impl Rule for NoRedeclare {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        DefaultRuleConfig::<Self>::from_value(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        // ES modules run in their own scope, so a top-level `Object` or `self` shadows nothing.
        // Only the globals half of the rule is skipped for them -- redeclaring a local binding is
        // still a redeclaration.
        let builtin_globals = if self.builtin_globals && !ctx.source_type().is_module() {
            Some(&GLOBALS_BUILTIN)
        } else {
            None
        };

        for symbol_id in ctx.scoping().symbol_ids() {
            let name = ctx.scoping().symbol_name(symbol_id);
            let decl_span = ctx.scoping().symbol_span(symbol_id);
            let is_builtin = builtin_globals.is_some_and(|builtin_globals| {
                builtin_globals.contains_key(name) || ctx.globals().is_enabled(name)
            });

            if is_builtin {
                ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(name, decl_span));
            }

            if ctx.source_type().is_typescript() {
                let redeclarations = ctx.scoping().symbol_redeclarations(symbol_id);
                if is_builtin {
                    for rd in redeclarations {
                        if rd.span != decl_span {
                            ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(name, rd.span));
                        }
                    }
                    continue;
                }

                let iter = redeclarations.iter().filter(|rd| {
                    if rd.flags.is_function() {
                        let node = ctx.nodes().get_node(rd.declaration);
                        if let Some(func) = node.kind().as_function() {
                            return !func.is_ts_declare_function();
                        }
                    }
                    true
                });

                let report_flags = if self.ignore_declaration_merge {
                    const IGNORED_MERGE_FLAGS: SymbolFlags = SymbolFlags::Ambient
                        .union(SymbolFlags::AsyncOrGeneratorFunction)
                        .union(SymbolFlags::NamespaceModule)
                        .union(SymbolFlags::ValueModule);
                    let flags = iter.clone().map(|rd| rd.flags).collect::<SymbolFlags>()
                        - IGNORED_MERGE_FLAGS;
                    if (SymbolFlags::Class | SymbolFlags::Interface).contains(flags) {
                        Some(SymbolFlags::Class)
                    } else if SymbolFlags::Function.contains(flags) {
                        Some(SymbolFlags::Function)
                    } else if SymbolFlags::Enum.contains(flags) {
                        Some(SymbolFlags::Enum)
                    } else {
                        None
                    }
                } else {
                    None
                };
                for (prev, next) in iter
                    .filter(|rd| report_flags.is_none_or(|flags| rd.flags.intersects(flags)))
                    .tuple_windows::<(_, _)>()
                {
                    ctx.diagnostic(no_redeclare_diagnostic(name, prev.span, next.span));
                }

                continue;
            }

            for windows in ctx.scoping().symbol_redeclarations(symbol_id).windows(2) {
                if is_builtin {
                    ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(name, windows[1].span));
                } else {
                    ctx.diagnostic(no_redeclare_diagnostic(name, windows[0].span, windows[1].span));
                }
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let defaults = NoRedeclare::default();
    assert!(defaults.builtin_globals);
    assert!(defaults.ignore_declaration_merge);

    let pass = vec![
        ("var a = 3; var b = function() { var a = 10; };", None),
        ("var a = 3; a = 10;", None),
        ("if (true) {\n    let b = 2;\n} else {    \nlet b = 3;\n}", None),
        ("var a; class C { static { var a; } }", None),
        ("class C { static { var a; } } var a; ", None),
        ("function a(){} class C { static { var a; } }", None),
        ("var a; class C { static { function a(){} } }", None),
        ("class C { static { var a; } static { var a; } }", None),
        ("class C { static { function a(){} } static { function a(){} } }", None),
        ("class C { static { var a; { function a(){} } } }", None),
        ("class C { static { function a(){}; { function a(){} } } }", None),
        ("class C { static { var a; { let a; } } }", None),
        ("class C { static { let a; { let a; } } }", None),
        ("class C { static { { let a; } { let a; } } }", None),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var Object = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var top = 0;", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var self = 1", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        ("var globalThis = foo", Some(serde_json::json!([{ "builtinGlobals": false }]))),
        // Issue: <https://github.com/oxc-project/oxc/issues/10396>
        ("export function foo(): void; export function foo() { }", None),
        ("function foo(arg: string): void; function foo(arg: number): any {}", None),
        ("function a(): string; function a(): number; function a() {}", None),
        (
            "function a(): string; function a(): number; function a() {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        ("interface A {} interface A {}", None),
        ("interface A {} interface A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("interface A {} class A {}", None),
        ("interface A {} class A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("class A {} namespace A {}", None),
        ("class A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("interface A {} class A {} namespace A {}", None),
        (
            "interface A {} class A {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": true }])),
        ),
        ("enum A {} namespace A {}", None),
        ("enum A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("function A() {} namespace A {}", None),
        ("function A() {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("namespace A {} namespace A {}", None),
        ("namespace A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("interface A {} namespace A {}", None),
        ("interface A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("namespace A { export const a = 1; } namespace A { export const b = 2; }", None),
        (
            "namespace A { export const a = 1; } namespace A { export const b = 2; }",
            Some(json!([{ "ignoreDeclarationMerge": true }])),
        ),
        ("namespace A {} class A {} interface A {}", None),
        (
            "namespace A {} class A {} interface A {}",
            Some(json!([{ "ignoreDeclarationMerge": true }])),
        ),
        ("namespace A {} function A() {}", None),
        ("namespace A {} function A() {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("namespace A {} enum A {}", None),
        ("namespace A {} enum A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("declare class A {} interface A {}", None),
        ("declare class A {} interface A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("async function A() {} namespace A {}", None),
        ("async function A() {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("function A(): string; function A(): number; function A() {} namespace A {}", None),
        (
            "function A(): string; function A(): number; function A() {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": true }])),
        ),
    ];

    let fail = vec![
        ("switch(foo) { case a: var b = 3;\ncase b: var b = 4}", None),
        ("var a = 3; var a = 10;", None),
        ("var a = {}; var a = [];", None),
        ("var a; function a() {}", None),
        ("function a() {} function a() {}", None),
        ("var a = function() { }; var a = function() { }", None),
        ("var a = function() { }; var a = new Date();", None),
        ("var a = 3; var a = 10; var a = 15;", None),
        ("var a; var a;", None),
        ("export var a; var a;", None),
        ("class C { static { var a; var a; } }", None),
        ("class C { static { var a; { var a; } } }", None),
        ("class C { static { { var a; } var a; } }", None),
        ("class C { static { { var a; } { var a; } } }", None),
        ("var Object = 0; var Object = 0; var globalThis = 0;", None),
        ("var a; var {a = 0, b: Object = 0} = {};", None),
        ("var a; var {a = 0, b: globalThis = 0} = {};", None),
        ("function f() { var a; var a; }", None),
        ("function f(a, b = 1) { var a; var b;}", None),
        ("function f() { var a; if (test) { var a; } }", None),
        ("for (var a, a;;);", None),
        // Issue: <https://github.com/oxc-project/oxc/issues/10396>
        ("export function undefined(): void; export function undefined() { }", None),
        ("type foo = 1; export function foo(): void; export function foo() { }", None),
        ("class A {} class A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        (
            "function A() {} function A() {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": true }])),
        ),
        ("enum A {} namespace A {} enum A {}", Some(json!([{ "ignoreDeclarationMerge": true }]))),
        ("function A() {} class A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        (
            "function A() {} class A {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        ("type something = string; const something = 2;", None),
        ("interface A {} const A = 2;", None),
        ("interface A {} class A {} const A = 2;", None),
        ("function A() {} class A {} namespace A {}", None),
        ("enum A {} interface A {} namespace A {}", None),
        ("var Object; namespace Object {}", None),
        ("class A {} interface A {} class A {} namespace A {}", None),
        ("function A(): void; function A() {} namespace A {} function A() {}", None),
        ("interface A {} interface A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("interface A {} class A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("class A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        (
            "interface A {} class A {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        ("enum A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("function A() {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("namespace A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("interface A {} namespace A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        (
            "namespace A { export const a = 1; } namespace A { export const b = 2; }",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        (
            "namespace A {} class A {} interface A {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        ("namespace A {} function A() {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("namespace A {} enum A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        ("declare class A {} interface A {}", Some(json!([{ "ignoreDeclarationMerge": false }]))),
        (
            "async function A() {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
        (
            "function A(): string; function A(): number; function A() {} namespace A {}",
            Some(json!([{ "ignoreDeclarationMerge": false }])),
        ),
    ];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, fail)
        .change_rule_path_extension(".cts")
        .test_and_snapshot();

    let fail = vec![("var foo;", None, Some(serde_json::json!({ "globals": { "foo": false }})))];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, vec![], fail)
        .change_rule_path_extension(".cts")
        .test();

    let pass = vec![(
        "import { performance } from 'node:perf_hooks'; (() => { performance })",
        None,
        Some(serde_json::json!({ "globals": { "performance": "readonly" }})),
    )];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, vec![])
        .change_rule_path_extension(".ts")
        .test();

    // ES modules have their own scope, so shadowing a global is fine there -- but a redeclaration
    // inside one is still a redeclaration. `.ts` is parsed as `ModuleKind::Unambiguous`, so these
    // cases carry ESM syntax to actually resolve as modules.
    // Issue: <https://github.com/oxc-project/oxc/issues/25339>
    let pass = vec![
        ("export {}; var Object = 0;", None),
        ("export {}; var globalThis = 0;", None),
        ("export {}; var a = 3; a = 10;", None),
        ("export {}; function foo(arg: string): void; function foo(arg: number): any {}", None),
    ];

    let fail = vec![
        ("export {}; var a = 3; var a = 10;", None),
        ("export {}; function f() { var a; var a; }", None),
        ("type foo = 1; export function foo(): void; export function foo() { }", None),
        ("export {}; var Object = 0; var Object = 0;", None),
    ];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, fail)
        .change_rule_path_extension(".ts")
        .with_snapshot_suffix("es-modules")
        .test_and_snapshot();
}
