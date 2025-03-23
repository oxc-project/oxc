use javascript_globals::GLOBALS;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;
use oxc_syntax::symbol::SymbolId;

use crate::{context::LintContext, rule::Rule};

fn no_redeclare_diagnostic(name: &str, decl_span: Span, re_decl_span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already defined.")).with_labels([
        decl_span.label(format!("'{name}' is already defined.")),
        re_decl_span.label("It can not be redeclare here."),
    ])
}

fn no_redeclare_as_builtin_in_diagnostic(name: &str, span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("'{name}' is already defined as a built-in global variable."))
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct NoRedeclare {
    built_in_globals: bool,
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
    ///
    /// ### Options
    ///
    /// #### builtinGlobals
    ///
    /// `{ type: bool, default: false }`
    ///
    /// When set `true`, it flags redeclaring built-in globals (e.g., `let Object = 1;`).
    NoRedeclare,
    eslint,
    pedantic
);

impl Rule for NoRedeclare {
    fn from_configuration(value: serde_json::Value) -> Self {
        let built_in_globals = value
            .get(0)
            .and_then(|config| config.get("builtinGlobals"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Self { built_in_globals }
    }

    fn run_on_symbol(&self, symbol_id: SymbolId, ctx: &LintContext) {
        let name = ctx.scoping().symbol_name(symbol_id);
        let is_builtin = self.built_in_globals
            && (GLOBALS["builtin"].contains_key(name) || ctx.globals().is_enabled(name));

        let decl_span = ctx.scoping().symbol_span(symbol_id);

        if is_builtin {
            ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(name, decl_span));
        }

        for span in ctx.scoping().symbol_redeclarations(symbol_id) {
            if is_builtin {
                ctx.diagnostic(no_redeclare_as_builtin_in_diagnostic(name, *span));
            } else {
                ctx.diagnostic(no_redeclare_diagnostic(name, decl_span, *span));
            }
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

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
        (
            "var Object = 0; var Object = 0; var globalThis = 0;",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
        ),
        (
            "var a; var {a = 0, b: Object = 0} = {};",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
        ),
        (
            "var a; var {a = 0, b: globalThis = 0} = {};",
            Some(serde_json::json!([{ "builtinGlobals": true }])),
        ),
        ("function f() { var a; var a; }", None),
        ("function f(a, b = 1) { var a; var b;}", None),
        ("function f() { var a; if (test) { var a; } }", None),
        ("for (var a, a;;);", None),
    ];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, pass, fail).test_and_snapshot();

    let fail = vec![(
        "var foo;",
        Some(serde_json::json!([{ "builtinGlobals": true }])),
        Some(serde_json::json!({ "globals": { "foo": false }})),
    )];

    Tester::new(NoRedeclare::NAME, NoRedeclare::PLUGIN, vec![], fail).test();
}
