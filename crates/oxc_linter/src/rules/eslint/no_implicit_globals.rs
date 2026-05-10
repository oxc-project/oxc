use javascript_globals::{GLOBALS, GLOBALS_BUILTIN};
use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};
use oxc_syntax::{reference::Reference, symbol::SymbolFlags};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    config::GlobalValue,
    context::LintContext,
    rule::{DefaultRuleConfig, Rule},
};

fn global_non_lexical_binding_diagnostic(kind: &'static str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected {kind} declaration in the global scope."))
        .with_help(
            "Wrap it in an IIFE for a local variable, or assign it as a global property for a global variable.",
        )
        .with_label(span)
}

fn global_lexical_binding_diagnostic(kind: &'static str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Unexpected {kind} declaration in the global scope."))
        .with_help("Wrap it in a block or in an IIFE.")
        .with_label(span)
}

fn global_variable_leak_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Global variable leak.")
        .with_help("Declare the variable if it is intended to be local.")
        .with_label(span)
}

fn assignment_to_readonly_global_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected assignment to read-only global variable.").with_label(span)
}

fn redeclaration_of_readonly_global_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Unexpected redeclaration of read-only global variable.").with_label(span)
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
struct NoImplicitGlobalsConfig {
    lexical_bindings: bool,
}

#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
pub struct NoImplicitGlobals(NoImplicitGlobalsConfig);

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallows declarations in the global scope, global variable leaks, and
    /// writes or redeclarations of read-only globals.
    ///
    /// ### Why is this bad?
    ///
    /// Browser scripts share a global scope. Top-level `var` and `function`
    /// declarations, and assignments to undeclared variables in sloppy mode,
    /// create globals that can collide with other scripts.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// var foo = 1;
    /// function bar() {}
    /// baz = 1;
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// window.foo = 1;
    /// (function() {
    ///   var bar = 1;
    /// }());
    /// ```
    NoImplicitGlobals,
    eslint,
    restriction,
    config = NoImplicitGlobals,
    version = "next",
);

impl Rule for NoImplicitGlobals {
    fn from_configuration(value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
        serde_json::from_value::<DefaultRuleConfig<Self>>(value).map(DefaultRuleConfig::into_inner)
    }

    fn run_once(&self, ctx: &LintContext) {
        let scoping = ctx.scoping();
        let root_scope_id = scoping.root_scope_id();
        let check_global_declarations =
            !ctx.source_type().is_module() && !ctx.source_type().is_commonjs();

        if check_global_declarations {
            for symbol_id in scoping.iter_bindings_in(root_scope_id) {
                let name = scoping.symbol_name(symbol_id);

                let global_value = global_variable_value(name, ctx);
                if global_value == Some(GlobalValue::Writable) {
                    continue;
                }

                let declarations = scoping.symbol_redeclarations(symbol_id);
                if declarations.is_empty() {
                    check_declaration(
                        scoping.symbol_flags(symbol_id),
                        scoping.symbol_span(symbol_id),
                        global_value,
                        self.0.lexical_bindings,
                        ctx,
                    );
                } else {
                    for redeclaration in declarations {
                        check_declaration(
                            redeclaration.flags,
                            redeclaration.span,
                            global_value,
                            self.0.lexical_bindings,
                            ctx,
                        );
                    }
                }
            }
        }

        for (name, reference_id_list) in scoping.root_unresolved_references() {
            let global_value = global_variable_value(name, ctx);

            if global_value == Some(GlobalValue::Writable) {
                continue;
            }

            for &reference_id in reference_id_list {
                let reference = scoping.get_reference(reference_id);
                if reference.is_type() || !reference.flags().is_write_only() {
                    continue;
                }

                if global_value == Some(GlobalValue::Readonly) {
                    ctx.diagnostic(assignment_to_readonly_global_diagnostic(assignment_span(
                        reference, ctx,
                    )));
                } else if !scoping.scope_flags(reference.scope_id()).is_strict_mode() {
                    ctx.diagnostic(global_variable_leak_diagnostic(assignment_span(
                        reference, ctx,
                    )));
                }
            }
        }
    }
}

fn global_variable_value(name: &str, ctx: &LintContext) -> Option<GlobalValue> {
    if let Some(value) = ctx.globals().get(name) {
        return Some(*value);
    }

    if GLOBALS_BUILTIN.contains_key(name) {
        return Some(GlobalValue::Readonly);
    }

    for env in ctx.env().iter() {
        if let Some(env) = GLOBALS.get(env)
            && let Some(value) = env.get(name)
        {
            return Some(GlobalValue::from(*value));
        }
    }

    None
}

fn check_declaration(
    flags: SymbolFlags,
    span: Span,
    global_value: Option<GlobalValue>,
    check_lexical_bindings: bool,
    ctx: &LintContext,
) {
    let Some(kind) = declaration_kind(flags, check_lexical_bindings) else {
        return;
    };

    if global_value == Some(GlobalValue::Readonly) {
        ctx.diagnostic(redeclaration_of_readonly_global_diagnostic(span));
        return;
    }

    let diagnostic = match kind {
        DeclarationKind::Function | DeclarationKind::Var => {
            global_non_lexical_binding_diagnostic(kind.as_message_kind(), span)
        }
        DeclarationKind::Class | DeclarationKind::Let | DeclarationKind::Const => {
            global_lexical_binding_diagnostic(kind.as_message_kind(), span)
        }
    };

    ctx.diagnostic(diagnostic);
}

#[derive(Debug, Clone, Copy)]
enum DeclarationKind {
    Var,
    Function,
    Let,
    Const,
    Class,
}

impl DeclarationKind {
    fn as_message_kind(self) -> &'static str {
        match self {
            Self::Var => "'var'",
            Self::Function => "function",
            Self::Let => "'let'",
            Self::Const => "'const'",
            Self::Class => "class",
        }
    }
}

fn declaration_kind(flags: SymbolFlags, check_lexical_bindings: bool) -> Option<DeclarationKind> {
    if flags.is_function() {
        return Some(DeclarationKind::Function);
    }

    if flags.contains(SymbolFlags::FunctionScopedVariable) {
        return Some(DeclarationKind::Var);
    }

    if check_lexical_bindings {
        if flags.is_class() {
            return Some(DeclarationKind::Class);
        }
        if flags.contains(SymbolFlags::BlockScopedVariable) {
            return Some(if flags.is_const_variable() {
                DeclarationKind::Const
            } else {
                DeclarationKind::Let
            });
        }
    }

    None
}

fn assignment_span(reference: &Reference, ctx: &LintContext) -> Span {
    let reference_node = ctx.nodes().get_node(reference.node_id());
    let mut node = reference_node;

    loop {
        let parent = ctx.nodes().parent_node(node.id());
        match parent.kind() {
            AstKind::AssignmentExpression(_)
            | AstKind::ForInStatement(_)
            | AstKind::ForOfStatement(_) => return parent.kind().span(),
            _ if parent.id() == node.id() => return reference_node.kind().span(),
            _ => node = parent,
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        ("window.foo = 1;", None),             // { "globals": globals.browser },
        ("window.foo = function() {};", None), // { "globals": globals.browser },
        ("window.foo = function foo() {};", None), // { "globals": globals.browser },
        ("window.foo = function bar() {};", None), // { "globals": globals.browser },
        ("window.foo = function*() {};", None), // { "ecmaVersion": 2015, "globals": globals.browser, },
        ("window.foo = function *foo() {};", None), // { "ecmaVersion": 2015, "globals": globals.browser, },
        ("window.foo = async function() {};", None), // { "ecmaVersion": 2017, "globals": globals.browser, },
        ("window.foo = async function foo() {};", None), // { "ecmaVersion": 2017, "globals": globals.browser, },
        ("window.foo = async function*() {};", None), // { "ecmaVersion": 2018, "globals": globals.browser, },
        ("window.foo = async function *foo() {};", None), // { "ecmaVersion": 2018, "globals": globals.browser, },
        ("window.foo = class {};", None), // { "ecmaVersion": 2015, "globals": globals.browser, },
        ("window.foo = class foo {};", None), // { "ecmaVersion": 2015, "globals": globals.browser, },
        ("window.foo = class bar {};", None), // { "ecmaVersion": 2015, "globals": globals.browser, },
        ("self.foo = 1;", None),              // { "globals": globals.browser },
        ("self.foo = function() {};", None),  // { "globals": globals.browser },
        ("this.foo = 1;", None),
        ("this.foo = function() {};", None),
        ("this.foo = function bar() {};", None),
        ("typeof function() {}", None),
        ("typeof function foo() {}", None),
        ("(function() {}) + (function foo() {})", None),
        ("typeof function *foo() {}", None), // { "ecmaVersion": 2015 },
        ("typeof async function foo() {}", None), // { "ecmaVersion": 2017 },
        ("typeof async function *foo() {}", None), // { "ecmaVersion": 2018 },
        ("(function() { var foo = 1; })();", None),
        ("(function() { function foo() {} })();", None),
        ("(function() { function *foo() {} })();", None), // { "ecmaVersion": 2015 },
        ("(function() { async function foo() {} })();", None), // { "ecmaVersion": 2017 },
        ("(function() { async function *foo() {} })();", None), // { "ecmaVersion": 2018 },
        (
            "window.foo = (function() { var bar; function foo () {}; return function bar() {} })();",
            None,
        ), // { "globals": globals.browser },
        ("const foo = 1; let bar; class Baz {}", None),   // { "ecmaVersion": 2015 },
        (
            "const foo = 1; let bar; class Baz {}",
            Some(serde_json::json!([{ "lexicalBindings": false }])),
        ), // { "ecmaVersion": 2015 },
        ("const Array = 1; let Object; class Math {}", None), // { "ecmaVersion": 2015 },
        ("typeof class {}", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("typeof class foo {}", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        (
            "{ const foo = 1; let bar; class Baz {} }",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "(function() { const foo = 1; let bar; class Baz {} })();",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
        ), // { "ecmaVersion": 2015 },
        (
            "window.foo = (function() { const bar = 1; let baz; class Quux {} return function () {} })();",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
        ), // { "ecmaVersion": 2015 },
        ("const foo = 1; let bar; class Baz {}", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("const foo = 1; let bar; class Baz {}", None), // { "ecmaVersion": 2015, "sourceType": "commonjs", },
        ("const foo = 1; let bar; class Baz {}", None), // { "ecmaVersion": 2015, "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("const foo = 1;", None),                       // { "ecmaVersion": 2015 },
        ("let foo = 1;", None),                         // { "ecmaVersion": 2015 },
        ("let foo = function() {};", None),             // { "ecmaVersion": 2015 },
        ("const foo = function() {};", None),           // { "ecmaVersion": 2015 },
        ("class Foo {}", None),                         // { "ecmaVersion": 2015 },
        ("(function() { let foo = 1; })();", None),     // { "ecmaVersion": 2015 },
        ("(function() { const foo = 1; })();", None),   // { "ecmaVersion": 2015 },
        ("let foo = 1;", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("const foo = 1;", None), // { "ecmaVersion": 2015, "sourceType": "module" },
        ("let foo = 1;", None), // { "ecmaVersion": 2015, "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("const foo = 1;", None), // { "ecmaVersion": 2015, "parserOptions": { "ecmaFeatures": { "globalReturn": true } }, },
        ("foo", None),
        ("foo + bar", None),
        ("foo(bar)", None),
        ("foo++", None),
        ("--foo", None),
        ("foo += 1", None),
        ("foo ||= 1", None), // { "ecmaVersion": 2021 },
        ("'use strict';foo = 1;", None),
        ("(function() {'use strict'; foo = 1; })();", None),
        ("{ class Foo { constructor() { bar = 1; } baz() { bar = 1; } } }", None), // { "ecmaVersion": 2015 },
        ("Foo.bar = 1;", None),
        ("Utils.foo = 1;", None),
        ("Utils.foo = function() {};", None),
        ("window.foo = 1;", None),
        ("window.foo = function() {};", None),
        ("window.foo = function foo() {};", None),
        ("self.foo = 1;", None),
        ("self.foo = function() {};", None),
        ("++foo", None),
        ("foo--", None),
        ("window.foo = function bar() { bar = 1; };", None), // { "globals": globals.browser },
        ("window.foo = function bar(baz) { baz = 1; };", None), // { "globals": globals.browser },
        ("window.foo = function bar() { var baz; function quux() { quux = 1; } };", None), // { "globals": globals.browser },
        ("Array.from = 1;", None),
        ("Object['assign'] = 1;", None),
        ("foo.bar = 1;", None),
    ];

    let fail = vec![
        ("var foo = 1;", None),
        ("function foo() {}", None),
        ("function *foo() {}", None),       // { "ecmaVersion": 2015 },
        ("async function foo() {}", None),  // { "ecmaVersion": 2017 },
        ("async function *foo() {}", None), // { "ecmaVersion": 2018 },
        ("var foo = function() {};", None),
        ("var foo = function foo() {};", None),
        ("var foo = function*() {};", None), // { "ecmaVersion": 2015 },
        ("var foo = function *foo() {};", None), // { "ecmaVersion": 2015 },
        ("var foo = 1, bar = 2;", None),
        ("const a = 1;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("let a;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("let a = 1;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("class A {}", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("const a = 1; const b = 2;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("const a = 1, b = 2;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("let a, b = 1;", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("const a = 1; let b; class C {}", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        ("const [a, b, ...c] = [];", Some(serde_json::json!([{ "lexicalBindings": true }]))), // { "ecmaVersion": 2015 },
        (
            "let { a, foo: b, bar: { c } } = {};",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
        ), // { "ecmaVersion": 2015 },
        ("foo = 1", None),
        ("foo = function() {};", None),
        ("foo = function*() {};", None), // { "ecmaVersion": 2015 },
        ("window.foo = function() { bar = 1; }", None),
        ("(function() {}(foo = 1));", None),
        ("for (foo in {});", None),
        ("for (foo of []);", None), // { "ecmaVersion": 2015 },
        ("window.foo = { bar() { foo = 1 } }", None), // { "ecmaVersion": 2015 },
        ("foo = 1, bar = 2;", None),
        ("foo = bar = 1", None),
        ("foo = 1; var bar;", None),
        ("var foo = bar = 1;", None),
        ("[foo, bar] = [];", None), // { "ecmaVersion": 2015 },
        ("Array = 1", None),
        ("window = 1;", None), // { "globals": globals.browser },
        ("var Array = 1", None),
        ("var Array = 1; Array = 2;", None),
    ];

    Tester::new(NoImplicitGlobals::NAME, NoImplicitGlobals::PLUGIN, pass, fail).test_and_snapshot();
}

#[test]
fn test_configured_globals() {
    use crate::tester::Tester;

    let writable_globals = Some(serde_json::json!({
        "globals": {
            "foo": "writable",
        },
    }));
    let readonly_globals = Some(serde_json::json!({
        "globals": {
            "foo": "readonly",
        },
    }));
    let disabled_globals = Some(serde_json::json!({
        "globals": {
            "foo": "off",
        },
    }));

    let pass = vec![
        ("foo = 1;", None, writable_globals.clone()),
        ("var foo = 1;", None, writable_globals.clone()),
        (
            "const foo = 1;",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
            writable_globals,
        ),
        ("foo.bar = 1;", None, readonly_globals.clone()),
    ];

    let fail = vec![
        ("foo = 1;", None, readonly_globals.clone()),
        ("for (foo in {});", None, readonly_globals.clone()),
        ("var foo = 1;", None, readonly_globals.clone()),
        ("function foo() {}", None, readonly_globals.clone()),
        (
            "const foo = 1;",
            Some(serde_json::json!([{ "lexicalBindings": true }])),
            readonly_globals,
        ),
        ("foo = 1;", None, disabled_globals),
    ];

    Tester::new(NoImplicitGlobals::NAME, NoImplicitGlobals::PLUGIN, pass, fail)
        .with_snapshot_suffix("configured-globals")
        .test_and_snapshot();
}
